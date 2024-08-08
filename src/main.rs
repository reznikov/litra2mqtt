use std::collections::HashMap;
use std::error::Error;

use log::{debug, error, info};
use rumqttc::{Event, Packet, QoS};

use device_remote_event::DeviceRemoteEvent;
use util::create_async_client;

mod device_hardware_event;
mod device_handle;
mod device_bridge;
mod util;
mod device_remote_event;
mod handle_device_events;
mod mqtt_topic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let (client, mut eventloop) = create_async_client().unwrap();
    let mut devices = HashMap::new();

    handle_device_events::handle_device_events(&client, &mut devices);

    let event_client = client.clone();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(Packet::Publish(incoming)) => {
                        let serial = util::get_serial(&incoming.topic).unwrap();
                        let (device_handle, device_bridge) = devices.get(&serial).unwrap();

                        match DeviceRemoteEvent::from_mqtt_message(incoming) {
                            DeviceRemoteEvent::Power(value) => {
                                info!("received mqtt power {}", value);
                                match device_handle.set_on(value) {
                                    Ok(_) => {
                                        device_bridge.publish_state(&event_client, value).await;
                                    }
                                    Err(e) => {
                                        error!("device does not support power; {}", e);
                                    }
                                }
                            }
                            DeviceRemoteEvent::BrightnessInLumen(value) => {
                                info!("received mqtt brightness: {}", value);
                                match device_handle.set_brightness_in_lumen(value) {
                                    Ok(_) => {
                                        // device acts like it supports only stepped brightness
                                        device_bridge.publish_brightness(&event_client, value).await;
                                    }
                                    Err(e) => {
                                        error!("Error setting brightness; {}", e);
                                    }
                                }
                            }
                            DeviceRemoteEvent::ColorTemperatureInKelvin(value) => {
                                info!("received mqtt color temperature: {}", value);
                                match device_handle.set_temperature_in_kelvin(value) {
                                    Ok(_) => {
                                        device_bridge.publish_color_temperature(&event_client, value).await;
                                    }
                                    Err(e) => {
                                        error!("Error setting color temperature; {}", e);
                                    }
                                }
                            }
                            DeviceRemoteEvent::Unknown(bytes) => {
                                error!("received mqtt unknown: {:?}", bytes);
                            }
                        }
                    }
                    event => {
                        debug!("Received something else {:?}", event);
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                    break;
                }
            }

            tokio::task::yield_now().await;
        }
    });

    client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::POWER), QoS::AtLeastOnce).await?;
    client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::BRIGHTNESS), QoS::AtLeastOnce).await?;
    client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::TEMPERATURE), QoS::AtLeastOnce).await?;

    info!("Program is running. Press CTRL+C to exit.");

    tokio::signal::ctrl_c().await.expect("Failed to listen for CTRL+C");

    info!("Exiting program");
    Ok(())
}
