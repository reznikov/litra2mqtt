use std::collections::HashMap;
use std::sync::Arc;

use log::{info};
use tokio::sync::Mutex;

use util::create_async_client;

mod device_bridge;
mod device_handle;
mod device_hardware_event;
mod device_remote_event;
mod handle_device_events;
mod handle_remote_events;
mod mqtt_topic;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let (client, eventloop) = create_async_client().unwrap();
    let devices = Arc::new(Mutex::new(HashMap::new()));

    handle_device_events::handle_device_events(&client, Arc::clone(&devices)).await;
    handle_remote_events::handle_remote_events(&client, eventloop, Arc::clone(&devices)).await;

    info!("Program is running. Press CTRL+C to exit.");

    tokio::signal::ctrl_c().await.expect("Failed to listen for CTRL+C");

    for (_, (_, device_bridge)) in devices.lock().await.iter() {
        device_bridge.publish_availability(&client, false).await;
    }

    info!("Exiting program");
    Ok(())
}
