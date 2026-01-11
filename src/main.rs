use std::collections::HashMap;
use std::sync::Arc;

use log::{info};
use rumqttc::QoS;
use tokio::sync::Mutex;

use util::create_async_client;

mod device_bridge;
mod device_handle;
mod device_hardware_event;
mod device_remote_event;
mod handle_device_events;
mod handle_remote_events;
mod mqtt_topic;
mod state;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let instance_id = util::get_instance_id();
    info!("Starting litra2mqtt bridge with instance_id: {}", instance_id);

    let (client, eventloop) = create_async_client().unwrap();
    let devices = Arc::new(Mutex::new(HashMap::new()));

    handle_device_events::handle_device_events(&client, Arc::clone(&devices), instance_id.clone()).await;
    handle_remote_events::handle_remote_events(&client, eventloop, Arc::clone(&devices)).await;

    info!("Program is running. Press CTRL+C to exit.");

    tokio::signal::ctrl_c().await.expect("Failed to listen for CTRL+C");

    info!("Shutting down - marking all devices as offline");
    for (_, (_, device_bridge)) in devices.lock().await.iter() {
        device_bridge.publish_availability(&client, false).await;
    }
    
    // Publish bridge status as offline
    let _ = client.publish("litra_bridge/status", QoS::AtLeastOnce, true, "offline").await;

    info!("Exiting program");
    Ok(())
}
