use std::collections::HashMap;
use std::error::Error;

use log::{debug, error, info};
use regex::Regex;
use rumqttc::{Event, Packet, Publish, QoS};

use crate::litra::device_handle::DeviceHandle;
use crate::litra::litra::Litra;
use crate::mqtt::device_bridge::DeviceBridge;
use crate::mqtt::util::create_async_client;

mod litra;
mod mqtt;

fn extract_serial_and_property(topic: &str) -> Option<(String, String)> {
    let re = Regex::new(r"logitech/litra-.*/([^/]+)/([^/]+)/set").unwrap();
    re.captures(topic).and_then(|cap| {
        let serial_number = cap.get(1)?.as_str().to_string();
        let state = cap.get(2)?.as_str().to_string();
        Some((serial_number, state))
    })
}

fn handle_message(message: Publish, litras: &HashMap<String, DeviceHandle>) {
    // logitech/litra-beam/2321FE9037D8/state/set
    // handle_mqtt_message(incoming).await;
    // incoming.topic
    if let Some((serial_number, property)) = extract_serial_and_property(&message.topic) {
        debug!("Serial number: {}, property: {}", serial_number, property);
        match litras.get(&serial_number) {
            Some(device_handle) => {
                let payload_result = std::str::from_utf8(&message.payload);

                match property.as_str() {
                    "state" => {
                        match payload_result {
                            Ok(payload) => {
                                match payload {
                                    "ON" | "OFF" => {
                                        debug!("Setting new power state: {}", payload);
                                        device_handle.set_on(payload == "ON").unwrap();
                                    }
                                    _ => error!("Invalid power state received: {}", payload),
                                }
                            }
                            Err(e) => error!("Failed to decode payload: {}", e),
                        }
                    }
                    "brightness" => {
                        match payload_result {
                            Ok(payload) => {
                                match payload.parse::<u16>() {
                                    Ok(brightness) => {
                                        info!("Setting new brightness: {}", brightness);
                                        let _ = device_handle.set_brightness_in_lumen(brightness);
                                    }
                                    _ => error!("Invalid brightness value received: {}", payload),
                                }
                            }
                            Err(e) => error!("Failed to decode payload: {}", e),
                        }
                    }
                    // "temperature" => {
                    //     let payload_result = std::str::from_utf8(&incoming.payload);
                    //     match payload_result {
                    //         Ok(payload) => {
                    //             match payload.parse::<f32>() {
                    //                 Ok(temperature) => {
                    //                     debug!("Setting new temperature: {}", temperature);
                    //                     device_handle.set_temperature(temperature).await;
                    //                 }
                    //                 _ => error!("Invalid temperature value received: {}", payload),
                    //             }
                    //         }
                    //         Err(e) => error!("Failed to decode payload: {}", e),
                    //     }
                    // }
                    _ => {
                        error!("Unknown property: {}", property);
                    }
                }
            }
            None => {
                error!("Device not found: {}", serial_number);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let (client, mut eventloop) = create_async_client().unwrap();
    let litra = Litra::new();

    if !litra.is_ok() {
        panic!("Unable to initialize Litra")
    }

    let litra = litra.unwrap();

    let mut litras = HashMap::new();

    litra.get_connected_devices().for_each(|device| {
        let device_handle = device.open(&litra).unwrap();
        litras.insert(device_handle.serial_number().unwrap(), device_handle);
    });

    litra.get_connected_devices().for_each(|device| {
        let device_handle = device.open(&litra).unwrap();
        let client = client.clone();
        let mut device_bridge = DeviceBridge::new(client, device_handle);

        tokio::spawn(async move {
            device_bridge.publish_discovery().await;
            device_bridge.publish_availability().await;

            loop {
                device_bridge.publish_state().await;
                device_bridge.publish_brightness().await;
                device_bridge.publish_color_temperature().await;

                tokio::task::yield_now().await;
            }
        });
    });

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(Packet::Publish(incoming)) => {
                        debug!("Received message: {:?}", incoming);
                        handle_message(incoming, &litras);
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

    client.subscribe("logitech/+/+/brightness/set", QoS::AtLeastOnce).await?;
    client.subscribe("logitech/+/+/temperature/set", QoS::AtLeastOnce).await?;
    client.subscribe("logitech/+/+/state/set", QoS::AtLeastOnce).await?;

    info!("Program is running. Press CTRL+C to exit.");

    tokio::signal::ctrl_c().await.expect("Failed to listen for CTRL+C");

    info!("Exiting program");
    Ok(())
}
