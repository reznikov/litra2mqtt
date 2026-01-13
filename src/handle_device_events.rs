use std::collections::HashMap;
use std::sync::Arc;

use litra::{DeviceError, DeviceHandle, Litra};
use log::{error, info};
use rumqttc::AsyncClient;
use tokio::sync::Mutex;

use crate::device_bridge::DeviceBridge;
use crate::device_hardware_event::DeviceHardwareEvent;
use crate::state::{DeviceState, DeviceStateManager};

pub async fn handle_device_events(
    client: &AsyncClient,
    devices_mutex: Arc<Mutex<HashMap<String, (DeviceHandle, DeviceBridge)>>>,
    instance_id: String,
) -> () {
    let litra = Litra::new().unwrap();
    let mut devices = devices_mutex.lock().await;

    litra.get_connected_devices().for_each(|device| {
        let device_handle = device.open(&litra).unwrap();
        let device_bridge = DeviceBridge::new(&device_handle);
        let device_id = device_bridge.device_id();

        info!("Found device: {}", device_id);
        devices.insert(device_id.clone(), (device.open(&litra).unwrap(), device_bridge));

        let device_client = client.clone();
        let instance_id_clone = instance_id.clone();

        tokio::spawn(async move {
            info!("Starting device event loop for {}", device_id);

            // Initialize state from device
            let initial_state = DeviceState::new(
                device_handle.is_on().unwrap_or(false),
                device_handle.brightness_in_lumen().unwrap_or(100),
                device_handle.temperature_in_kelvin().unwrap_or(3000),
                instance_id_clone.clone(),
            );

            let mut state_manager = DeviceStateManager::new(initial_state, instance_id_clone);

            // Publish initial state and discovery
            device_bridge
                .publish_availability(&device_client, true)
                .await;
            device_bridge
                .publish_discovery(
                    &device_client,
                    device_bridge.create_discovery_message(&device_handle),
                )
                .await;
            device_bridge
                .publish_state(&device_client, state_manager.current_state())
                .await;
            state_manager.mark_published();

            device_handle.hid_device().set_blocking_mode(false).unwrap();

            loop {
                let mut device_buffer: [u8; 6] = [0; 6];
                let hid_device = device_handle.hid_device();

                hid_device.set_blocking_mode(false).unwrap();

                let read_result = hid_device
                    .read(&mut device_buffer)
                    .map_err(DeviceError::from);

                if let Err(e) = read_result {
                    error!("Error reading from device {}: {}", device_id, e);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }

                if device_buffer[0] == 0x0 {
                    tokio::task::yield_now().await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }

                hid_device.set_blocking_mode(true).unwrap();

                match DeviceHardwareEvent::from_bytes(&device_buffer) {
                    DeviceHardwareEvent::Power(value) => {
                        info!("Device {} hardware power change: {}", device_id, value);
                        if let Some(new_state) = state_manager.update_from_hardware(Some(value), None, None) {
                            device_bridge.publish_state(&device_client, &new_state).await;
                            state_manager.mark_published();
                        }
                    }
                    DeviceHardwareEvent::PowerAck => {
                        info!("Device {} hardware power ack", device_id);
                        // Re-read actual state from device and update if different
                        if let Ok(actual_power) = device_handle.is_on() {
                            if let Some(new_state) = state_manager.update_from_hardware(Some(actual_power), None, None) {
                                device_bridge.publish_state(&device_client, &new_state).await;
                                state_manager.mark_published();
                            }
                        }
                    }
                    DeviceHardwareEvent::BrightnessInLumen(value) => {
                        info!("Device {} hardware brightness change: {}", device_id, value);
                        if let Some(new_state) = state_manager.update_from_hardware(None, Some(value), None) {
                            device_bridge.publish_state(&device_client, &new_state).await;
                            state_manager.mark_published();
                        }
                    }
                    DeviceHardwareEvent::BrightnessInLumenAck => {
                        info!("Device {} hardware brightness ack", device_id);
                    }
                    DeviceHardwareEvent::ColorTemperatureInKelvin(value) => {
                        info!("Device {} hardware temperature change: {}", device_id, value);
                        if let Some(new_state) = state_manager.update_from_hardware(None, None, Some(value)) {
                            device_bridge.publish_state(&device_client, &new_state).await;
                            state_manager.mark_published();
                        }
                    }
                    DeviceHardwareEvent::ColorTemperatureInKelvinAck => {
                        info!("Device {} hardware temperature ack", device_id);
                    }
                    DeviceHardwareEvent::Unknown(bytes) => {
                        info!("Device {} hardware unknown event: {:?}", device_id, bytes);
                    }
                }

                tokio::task::yield_now().await;
            }
        });
    });
}
