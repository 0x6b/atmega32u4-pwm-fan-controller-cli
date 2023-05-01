use std::{error::Error, time::Duration};

use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType},
    platform::{Adapter, Manager, Peripheral},
};
use structopt::StructOpt;
use tokio::time;
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
    let args = Args::from_args();

    let manager = Manager::new().await?;
    let adapter = get_first_adapter(&manager).await.unwrap();
    println!("Using adapter: {:?}", adapter.adapter_info().await?);

    adapter
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");
    loop {
        match find_adafruit_ble(&adapter).await {
            None => time::sleep(Duration::from_millis(500)).await,
            Some(peripheral) => {
                println!(
                    "Found peripheral: {}",
                    peripheral.properties().await?.unwrap().local_name.unwrap()
                );

                let characteristic = get_tx_characteristic(&peripheral).await?;
                println!("Found TX UART characteristic: {}", characteristic.uuid);

                let speed = if args.speed > 100 {
                    100
                } else {
                    args.speed
                };
                // treat speed as percentage, convert it to 0-255 range
                let speed = (speed as f32 * 2.55) as u8;
                println!("Speed: {}", speed);

                let speed = speed.to_string();
                peripheral
                    .write(&characteristic, speed.as_bytes(), WriteType::WithResponse)
                    .await?;
                println!("Wrote {} ({}%) to the UART", speed, args.speed);

                peripheral.disconnect().await?;
                println!("Disconnected from peripheral");

                break;
            }
        }
    }

    Ok(())
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

async fn get_tx_characteristic(peripheral: &Peripheral) -> Result<Characteristic, Box<dyn Error>> {
    peripheral.connect().await?;
    peripheral.discover_services().await?;
    let uuid = Uuid::parse_str(TX_UUID).unwrap();
    for c in peripheral.characteristics().iter() {
        if c.uuid == uuid {
            return Ok(c.clone());
        }
    }
    Err("Could not find TX characteristic".into())
}
