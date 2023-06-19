use std::{error::Error, time::Duration};

use btleplug::{
    api::{
        Central,
        Characteristic,
        Manager as _,
        Peripheral as _,
        ScanFilter,
        WriteType::WithResponse,
    },
    platform::{Adapter, Manager, Peripheral},
};
use clap::Parser;
use tokio::time;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// Fan speed in percentage
    #[arg(default_value = "10")]
    speed: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    let manager = Manager::new().await?;
    let adapter = get_first_adapter(&manager).await.unwrap();

    info!(
        "Using first adapter found: {}",
        adapter.adapter_info().await?
    );

    if let Err(why) = adapter.start_scan(ScanFilter::default()).await {
        error!("Can't scan BLE adapter: {why}");
        return Err(why.into());
    }

    loop {
        match find_adafruit_ble(&adapter).await {
            None => time::sleep(Duration::from_millis(10)).await,
            Some(p) => {
                info!("Found peripheral: {}", get_peripheral_local_name(&p).await);

                let char = match get_tx_characteristic(&p).await {
                    Ok(c) => {
                        info!("Found TX UART characteristic: {}", c.uuid);
                        c
                    }
                    Err(why) => {
                        error!("Can't get TX UART characteristic: {why}");
                        return Err(why);
                    }
                };

                let speed = convert_percentage_to_pwm_duty_cycle(args.speed);

                p.write(&char, speed.to_string().as_bytes(), WithResponse)
                    .await?;
                info!("Speed set: {}% ({}/255)", args.speed, speed);

                p.disconnect().await?;
                info!("Disconnected from peripheral");

                break;
            }
        }
    }

    Ok(())
}

async fn get_peripheral_local_name(peripheral: &Peripheral) -> String {
    let prop = peripheral.properties().await.unwrap();
    match prop {
        None => {
            info!("No properties found for peripheral");
            "Unknown".to_string()
        }
        Some(prop) => prop.local_name.unwrap_or("Unknown".to_string()),
    }
}

fn convert_percentage_to_pwm_duty_cycle(percentage: u8) -> u8 {
    let speed = if percentage > 100 { 100 } else { percentage };
    // convert 0-100 to 0-255
    (speed as f32 * 2.55) as u8
}

async fn get_first_adapter(manager: &Manager) -> Option<Adapter> {
    manager.adapters().await.unwrap().into_iter().next()
}

async fn find_adafruit_ble(adapter: &Adapter) -> Option<Peripheral> {
    for p in adapter.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name == "Adafruit Bluefruit LE")
        {
            return Some(p);
        }
    }
    None
}

async fn get_tx_characteristic(p: &Peripheral) -> Result<Characteristic, Box<dyn Error>> {
    // https://learn.adafruit.com/adafruit-feather-32u4-bluefruit-le/uart-service
    let uuid = Uuid::parse_str("6e400002-b5a3-f393-e0a9-e50e24dcca9e").unwrap();

    p.connect().await?;
    p.discover_services().await?;
    p.characteristics()
        .iter()
        .find(|c| c.uuid == uuid)
        .cloned()
        .ok_or("Could not find TX characteristic".into())
}
