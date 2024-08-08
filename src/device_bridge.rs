use litra::DeviceHandle;
use rumqttc::{AsyncClient, QoS};
use serde_json::{json, Value};

use crate::device_handle::DeviceHandleWithMerids;
use crate::mqtt_topic;

#[derive(Copy, Clone)]
pub struct DeviceBridge {
    serial_number: &'static str,
    device_model: &'static str,
    device_model_key: &'static str,
}

impl DeviceBridge {
    pub fn new(device_handle: &DeviceHandle) -> DeviceBridge {
        let serial_number = Box::leak(device_handle.serial_number().unwrap().unwrap().into_boxed_str());
        let device_model = Box::leak(device_handle.device_type().to_string().into_boxed_str());
        let device_model_key = Box::leak(device_model.to_lowercase().replace(" ", "-").into_boxed_str());

        DeviceBridge {
            serial_number,
            device_model,
            device_model_key,
        }
    }

    pub async fn publish_discovery(&self, mqtt_client: &AsyncClient, discovery_message: Value) {
        mqtt_client.publish(
            format!(
                "homeassistant/light/logitech_{device_model}_{serial_number}/config",
                device_model = self.device_model_key,
                serial_number = self.serial_number
            ),
            QoS::AtLeastOnce,
            false,
            discovery_message.to_string(),
        ).await.unwrap();
    }

    pub async fn publish_availability(&self, mqtt_client: &AsyncClient, available: bool) {
        mqtt_client.publish(
            format!(
                "logitech/{device_model}/{serial_number}/{topic}",
                device_model = self.device_model_key,
                serial_number = self.serial_number,
                topic = mqtt_topic::AVAILABILITY
            ),
            QoS::AtLeastOnce,
            false,
            match available {
                true => "online",
                false => "offline",
            },
        ).await.unwrap();
    }

    pub async fn publish_state(&self, mqtt_client: &AsyncClient, state: bool) {
        mqtt_client.publish(
            format!(
                "logitech/{device_model}/{serial_number}/{topic}",
                device_model = self.device_model_key,
                serial_number = self.serial_number,
                topic = mqtt_topic::POWER
            ),
            QoS::AtLeastOnce,
            false,
            match state {
                true => "ON",
                false => "OFF",
            },
        ).await.unwrap();
    }

    pub async fn publish_brightness(&self, mqtt_client: &AsyncClient, brightness: u16) {
        mqtt_client.publish(
            format!(
                "logitech/{device_model}/{serial_number}/{topic}",
                device_model = self.device_model_key,
                serial_number = self.serial_number,
                topic = mqtt_topic::BRIGHTNESS
            ),
            QoS::AtLeastOnce,
            false,
            brightness.to_string(),
        ).await.unwrap();
    }

    pub async fn publish_color_temperature(&self, mqtt_client: &AsyncClient, color_temperature: u16) {
        mqtt_client.publish(
            format!(
                "logitech/{device_model}/{serial_number}/{topic}",
                device_model = self.device_model_key,
                serial_number = self.serial_number,
                topic = mqtt_topic::TEMPERATURE
            ),
            QoS::AtLeastOnce,
            false,
            color_temperature.to_string(),
        ).await.unwrap();
    }

    pub fn create_discovery_message(&self, device_handle: &DeviceHandle) -> Value {
        let serial_number = self.serial_number;
        let device_model = self.device_model;
        let device_model_key = self.device_model_key;

        let device_min = device_handle.minimum_brightness_in_lumen();
        let device_max = device_handle.maximum_brightness_in_lumen();

        return json!({
            "~": format!("logitech/{device_model_key}/{serial_number}"),
            "device_class": "light",
            "supported_color_modes": [
                "color_temp"
                // todo: add rgb for beam lx
            ],
            "unique_id": format!("logitech_{device_model}_{serial_number}", device_model = device_model.replace(" ", "_")).to_lowercase(),
            "device": {
                "name": format!("Logitech {device_model}"),
                "identifiers": serial_number,
                "manufacturer": "Logitech",
                "model": device_model,
                "serial_number": serial_number,
            },

            "availability_topic": format!("~/{}", mqtt_topic::AVAILABILITY),
            "state_topic": format!("~/{}", mqtt_topic::POWER),
            "command_topic": format!("~/{}/set", mqtt_topic::POWER),

            "brightness_scale": device_max - device_min,
            "brightness_state_topic": format!("~/{}", mqtt_topic::BRIGHTNESS),
            "brightness_value_template": format!("{{{{ value - {device_min} }}}}"),
            "brightness_command_topic": format!("~/{}/set", mqtt_topic::BRIGHTNESS),
            "brightness_command_template": format!("{{{{ value + {device_min} }}}}"),

            "min_mireds": device_handle.minimum_temperature_in_mireds(),
            "max_mireds": device_handle.maximum_temperature_in_mireds(),

            "color_temp_state_topic": format!("~/{}", mqtt_topic::TEMPERATURE),
            "color_temp_value_template": "{{ (1000000 / value) | int }}",
            "color_temp_command_topic": format!("~/{}/set", mqtt_topic::TEMPERATURE),

            // round value to closest 00 for color temp
            "color_temp_command_template": "{{ (1000000 / value / 100) | int * 100 }}"
        });
    }
}
