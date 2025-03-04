use std::collections::HashMap;
use std::sync::Arc;

use litra::DeviceHandle;
use log::{debug, error, info};
use rumqttc::{AsyncClient, Event, EventLoop, Packet, QoS};
use tokio::sync::Mutex;

use crate::device_bridge::DeviceBridge;
use crate::device_remote_event::DeviceRemoteEvent;
use crate::{mqtt_topic, util};

pub async fn handle_remote_events(
    client: &AsyncClient,
    mut eventloop: EventLoop,
    devices_mutex: Arc<Mutex<HashMap<String, (DeviceHandle, DeviceBridge)>>>,
) -> () {
    let event_client = client.clone();

    tokio::spawn(async move {
        let devices = devices_mutex.lock().await;

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
                                        device_bridge
                                            .publish_brightness(&event_client, value)
                                            .await;
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
                                        device_bridge
                                            .publish_color_temperature(&event_client, value)
                                            .await;
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
                },
                Err(e) => {
                    error!("Error: {}", e);
                    break;
                }
            }

            tokio::task::yield_now().await;
        }
    });

    if let Err(e) = client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::POWER), QoS::AtLeastOnce, ).await {
        error!("Error subscribing to power: {}", e);
    }
    if let Err(e) = client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::BRIGHTNESS), QoS::AtLeastOnce, ).await {
        error!("Error subscribing to brightness: {}", e);
    }
    if let Err(e) = client.subscribe(format!("logitech/+/+/{}/set", mqtt_topic::TEMPERATURE), QoS::AtLeastOnce, ).await {
        error!("Error subscribing to temperature: {}", e);
    }
}
