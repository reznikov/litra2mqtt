use std::collections::HashMap;
use std::sync::Arc;

use litra::{DeviceError, DeviceHandle, Litra};
use log::info;
use rumqttc::AsyncClient;
use tokio::sync::Mutex;

use crate::device_bridge::DeviceBridge;
use crate::device_hardware_event::DeviceHardwareEvent;

pub async fn handle_device_events(
    client: &AsyncClient,
    devices_mutex: Arc<Mutex<HashMap<String, (DeviceHandle, DeviceBridge)>>>,
) -> () {
    let litra = Litra::new().unwrap();
    let mut devices = devices_mutex.lock().await;

    litra.get_connected_devices().for_each(|device| {
        let device_handle = device.open(&litra).unwrap();
        let device_bridge = DeviceBridge::new(&device_handle);
        let serial_number = device_handle.serial_number().unwrap().unwrap();

        info!("Found device: {serial_number}");
        devices.insert(serial_number, (device.open(&litra).unwrap(), device_bridge));

        let device_client = client.clone();

        tokio::spawn(async move {
            info!("Starting device event loop");

            device_bridge
                .publish_state(&device_client, device_handle.is_on().unwrap())
                .await;
            device_bridge
                .publish_brightness(&device_client, device_handle.brightness_in_lumen().unwrap())
                .await;
            device_bridge
                .publish_color_temperature(
                    &device_client,
                    device_handle.temperature_in_kelvin().unwrap(),
                )
                .await;
            device_bridge
                .publish_availability(&device_client, true)
                .await;
            device_bridge
                .publish_discovery(
                    &device_client,
                    device_bridge.create_discovery_message(&device_handle),
                )
                .await;

            device_handle.hid_device().set_blocking_mode(false).unwrap();

            loop {
                let mut device_buffer: [u8; 6] = [0; 6];
                let hid_device = device_handle.hid_device();

                hid_device.set_blocking_mode(false).unwrap();

                let _ = hid_device
                    .read(&mut device_buffer)
                    .map_err(DeviceError::from);

                if device_buffer[0] == 0x0 {
                    tokio::task::yield_now().await;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }

                hid_device.set_blocking_mode(true).unwrap();

                match DeviceHardwareEvent::from_bytes(&device_buffer) {
                    DeviceHardwareEvent::Power(value) => {
                        info!("received hardware power {}", value);
                        device_bridge.publish_state(&device_client, value).await;
                    }
                    DeviceHardwareEvent::PowerAck => {
                        info!("received hardware power ack");
                        device_bridge
                            .publish_state(&device_client, device_handle.is_on().unwrap())
                            .await;
                    }
                    DeviceHardwareEvent::BrightnessInLumen(value) => {
                        info!("received hardware brightness: {}", value);
                        device_bridge
                            .publish_brightness(&device_client, value)
                            .await;
                    }
                    DeviceHardwareEvent::BrightnessInLumenAck => {
                        info!("received hardware brightness ack");
                    }
                    DeviceHardwareEvent::ColorTemperatureInKelvin(value) => {
                        info!("received hardware color temperature: {}", value);
                        device_bridge
                            .publish_color_temperature(&device_client, value)
                            .await;
                    }
                    DeviceHardwareEvent::ColorTemperatureInKelvinAck => {
                        info!("received hardware color temperature ack");
                    }
                    DeviceHardwareEvent::Unknown(bytes) => {
                        info!("received hardware unknown: {:?}", bytes);
                    }
                }

                tokio::task::yield_now().await;
            }
        });
    });
}
