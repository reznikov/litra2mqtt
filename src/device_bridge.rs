use litra::DeviceHandle;
use log::{error, info};
use rumqttc::{AsyncClient, QoS};
use serde_json::{json, Value};

use crate::device_handle::DeviceHandleWithMerids;
use crate::mqtt_topic;
use crate::state::DeviceState;

#[derive(Copy, Clone)]
pub struct DeviceBridge {
    serial_number: &'static str,
    device_model: &'static str,
    device_model_key: &'static str,
}

impl DeviceBridge {
    pub fn new(device_handle: &DeviceHandle) -> DeviceBridge {
        let serial_number = Box::leak(
            device_handle
                .serial_number()
                .unwrap()
                .unwrap()
                .into_boxed_str(),
        );
        let device_model = Box::leak(
            device_handle
                .device_type()
                .to_string()
                .into_boxed_str()
        );
        let device_model_key = Box::leak(
            device_model
                .to_lowercase()
                .replace(" ", "-")
                .into_boxed_str(),
        );

        DeviceBridge {
            serial_number,
            device_model,
            device_model_key,
        }
    }

    pub fn device_id(&self) -> String {
        format!("logitech_{}_{}", self.device_model_key, self.serial_number)
    }

    pub async fn publish_discovery(
        &self,
        mqtt_client: &AsyncClient,
        discovery_message: Value
    ) -> () {
        info!("Publishing discovery for device: {}", self.device_id());

        let topic = mqtt_topic::discovery_topic(&self.device_id());
        
        let result = mqtt_client
            .publish(
                topic,
                QoS::AtLeastOnce,
                true, // retain discovery messages
                discovery_message.to_string(),
            )
            .await;

        if let Err(error) = result {
            error!("Error publishing discovery: {error}");
        }
    }

    pub async fn publish_availability(
        &self,
        mqtt_client: &AsyncClient,
        available: bool
    ) -> () {
        info!("Publishing availability for device: {} - {}", self.device_id(), available);

        let topic = mqtt_topic::availability_topic(&self.device_id());
        
        let result = mqtt_client
            .publish(
                topic,
                QoS::AtLeastOnce,
                true, // retain availability
                match available {
                    true => "online",
                    false => "offline",
                },
            )
            .await;

        if let Err(error) = result {
            error!("Error publishing availability: {error}");
        }
    }

    pub async fn publish_state(
        &self,
        mqtt_client: &AsyncClient,
        state: &DeviceState
    ) -> () {
        info!("Publishing state for device: {}", self.device_id());

        let topic = mqtt_topic::state_topic(&self.device_id());
        
        let state_json = json!({
            "state": if state.power { "ON" } else { "OFF" },
            "brightness": state.brightness,
            "color_temp": (1000000.0 / state.temperature as f64) as u16, // Convert K to mireds
            "revision": state.revision,
            "timestamp": state.timestamp,
            "origin": state.origin,
        });

        let result = mqtt_client
            .publish(
                topic,
                QoS::AtLeastOnce,
                true, // retain state
                state_json.to_string(),
            ).await;

        if let Err(error) = result {
            error!("Error publishing state: {error}");
        }
    }

    pub fn create_discovery_message(&self, device_handle: &DeviceHandle) -> Value {
        let device_id = self.device_id();
        let serial_number = self.serial_number;
        let device_model = self.device_model;

        let device_min = device_handle.minimum_brightness_in_lumen();
        let device_max = device_handle.maximum_brightness_in_lumen();

        json!({
            "name": format!("Logitech {}", device_model),
            "unique_id": device_id,
            "object_id": device_id,
            "device_class": "light",
            "supported_color_modes": ["color_temp"],
            
            "device": {
                "name": format!("Logitech {}", device_model),
                "identifiers": [device_id.clone()],
                "manufacturer": "Logitech",
                "model": device_model,
                "serial_number": serial_number,
            },

            "availability_topic": mqtt_topic::availability_topic(&device_id),
            "state_topic": mqtt_topic::state_topic(&device_id),
            "command_topic": mqtt_topic::command_topic(&device_id),
            "state_value_template": "{{ value_json.state }}",
            
            "brightness_scale": device_max - device_min,
            "brightness_value_template": format!("{{{{ (value_json.brightness | int) - {} }}}}", device_min),
            "brightness_command_template": format!("{{{{ (value | int) + {} }}}}", device_min),

            "min_mireds": device_handle.minimum_temperature_in_mireds(),
            "max_mireds": device_handle.maximum_temperature_in_mireds(),
            "color_temp_value_template": "{{ value_json.color_temp | int }}",

            "json_attributes_topic": mqtt_topic::state_topic(&device_id),
            "json_attributes_template": "{{ {'revision': value_json.revision, 'timestamp': value_json.timestamp, 'origin': value_json.origin} | tojson }}",

            "schema": "json",
        })
    }
}
