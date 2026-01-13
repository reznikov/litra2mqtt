use std::collections::HashMap;
use std::sync::Arc;

use litra::DeviceHandle;
use log::{debug, error, info};
use rumqttc::{AsyncClient, Event, EventLoop, Packet, QoS};
use tokio::sync::Mutex;

use crate::device_bridge::DeviceBridge;
use crate::device_remote_event::DeviceRemoteEvent;
use crate::util;

pub async fn handle_remote_events(
    client: &AsyncClient,
    mut eventloop: EventLoop,
    devices_mutex: Arc<Mutex<HashMap<String, (DeviceHandle, DeviceBridge)>>>,
) -> () {
    let event_client = client.clone();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(Packet::ConnAck(_)) => {
                        info!("MQTT Connected! Subscribing to command topics...");
                        
                        // Subscribe to all device command topics
                        if let Err(e) = event_client.subscribe("litra_bridge/+/command", QoS::AtLeastOnce).await {
                            error!("Error subscribing to command topics: {}", e);
                        }
                        
                        // Publish bridge status
                        if let Err(e) = event_client.publish(
                            "litra_bridge/status",
                            QoS::AtLeastOnce,
                            true,
                            "online"
                        ).await {
                            error!("Error publishing bridge status: {}", e);
                        }
                    }
                    Event::Incoming(Packet::Publish(incoming)) => {
                        let topic = incoming.topic.clone();
                        let device_id = match util::get_serial(&topic) {
                            Some(id) => id,
                            None => {
                                error!("Could not extract device_id from topic: {}", topic);
                                continue;
                            }
                        };

                        let devices = devices_mutex.lock().await;
                        let device_info = match devices.get(&device_id) {
                            Some(info) => info,
                            None => {
                                error!("Device not found: {}", device_id);
                                continue;
                            }
                        };
                        
                        let (device_handle, _device_bridge) = device_info;

                        // Get current state from device
                        let current_power = device_handle.is_on().unwrap_or(false);
                        let current_brightness = device_handle.brightness_in_lumen().unwrap_or(100);
                        let current_temp = device_handle.temperature_in_kelvin().unwrap_or(3000);
                        
                        let current_state = crate::state::DeviceState::new(
                            current_power,
                            current_brightness,
                            current_temp,
                            "mqtt_command".to_string(), // temporary origin for comparison
                        );

                        match DeviceRemoteEvent::from_mqtt_message(incoming, &current_state) {
                            DeviceRemoteEvent::StateUpdate(desired_state) => {
                                info!("Received MQTT command for device {}: {}", device_id, 
                                      format!("power={}, brightness={}, temp={}", 
                                              desired_state.power, desired_state.brightness, desired_state.temperature));
                                
                                // Apply changes to device
                                
                                if desired_state.power != current_power {
                                    match device_handle.set_on(desired_state.power) {
                                        Ok(_) => {
                                            info!("Set power to {} for device {}", desired_state.power, device_id);
                                        }
                                        Err(e) => {
                                            error!("Error setting power for device {}: {}", device_id, e);
                                        }
                                    }
                                }
                                
                                if desired_state.brightness != current_brightness {
                                    match device_handle.set_brightness_in_lumen(desired_state.brightness) {
                                        Ok(_) => {
                                            info!("Set brightness to {} for device {}", desired_state.brightness, device_id);
                                        }
                                        Err(e) => {
                                            error!("Error setting brightness for device {}: {}", device_id, e);
                                        }
                                    }
                                }
                                
                                if desired_state.temperature != current_temp {
                                    match device_handle.set_temperature_in_kelvin(desired_state.temperature) {
                                        Ok(_) => {
                                            info!("Set temperature to {} for device {}", desired_state.temperature, device_id);
                                        }
                                        Err(e) => {
                                            error!("Error setting temperature for device {}: {}", device_id, e);
                                        }
                                    }
                                }
                                
                                // Note: State will be published by hardware event handler
                                // This prevents duplicate publishes and maintains proper revision tracking
                            }
                            DeviceRemoteEvent::Unknown(msg) => {
                                error!("Unknown MQTT command for device {}: {}", device_id, msg);
                            }
                        }
                    }
                    event => {
                        debug!("Received MQTT event: {:?}", event);
                    }
                },
                Err(e) => {
                    error!("MQTT Error: {} - will attempt to reconnect", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }

            tokio::task::yield_now().await;
        }
    });
}
