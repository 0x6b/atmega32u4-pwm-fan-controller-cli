use std::{error::Error, time::Duration};

use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType},
    platform::{Adapter, Manager, Peripheral},
};
use structopt::StructOpt;
use tokio::time;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

// https://learn.adafruit.com/adafruit-feather-32u4-bluefruit-le/uart-service
const TX_UUID: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";

#[derive(Debug, StructOpt)]
#[structopt(name = "fanctl", about = "Control the fan speed")]
struct Args {
    /// Fan speed in percentage
    #[structopt(default_value = "10")]
    speed: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::from_args();
    let manager = Manager::new().await?; // always Ok
    let adapter = get_first_adapter(&manager).await.unwrap();

    info!("Using first adapter found: {}", adapter.adapter_info().await?); // always Ok

    if let Err(why) = adapter.start_scan(ScanFilter::default()).await {
        error!("Can't scan BLE adapter: {why}");
        return Err(why.into());
    }

    loop {
        match find_adafruit_ble(&adapter).await {
            None => time::sleep(Duration::from_millis(10)).await,
            Some(peripheral) => {
                let prop = &peripheral.properties().await?; // always Ok
                let prop = match prop {
                    None => {
                        info!("No properties found for peripheral");
                        continue;
                    }
                    Some(prop) => prop,
                };
                info!("Found peripheral: {}", prop.local_name.as_ref().unwrap_or(&"Unknown".to_string()));

                let characteristic = match get_tx_characteristic(&peripheral).await {
                    Ok(c) => c,
                    Err(why) => {
                        info!("Can't get TX UART characteristic: {why}");
                        return Err(why);
                    }
                };
                info!("Found TX UART characteristic: {}", characteristic.uuid);

                let speed = if args.speed > 100 {
                    100
                } else {
                    args.speed
                };

                // treat speed as percentage, convert it to 0-255 range
                let speed = (speed as f32 * 2.55) as u8;
                let speed = speed.to_string();

                peripheral
                    .write(&characteristic, speed.as_bytes(), WriteType::WithResponse)
                    .await?; // always Ok
                info!("Speed set: {}% ({}/255)", args.speed, speed);

                peripheral.disconnect().await?; // always Ok
                info!("Disconnected from peripheral");

                break;
            }
        }
    }

    Ok(())
}

async fn get_first_adapter(manager: &Manager) -> Option<Adapter> {
    manager.adapters().await.unwrap() // always Ok
        .into_iter().next()
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

async fn get_tx_characteristic(peripheral: &Peripheral) -> Result<Characteristic, Box<dyn Error>> {
    peripheral.connect().await?; // always Ok
    peripheral.discover_services().await?; // always Ok
    let uuid = Uuid::parse_str(TX_UUID).unwrap(); // always Ok
    for c in peripheral.characteristics().iter() {
        if c.uuid == uuid {
            return Ok(c.clone());
        }
    }
    Err("Could not find TX characteristic".into())
}
