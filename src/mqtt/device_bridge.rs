use rumqttc::{AsyncClient, QoS};
use serde_json::{json, Value};
use crate::litra::device_handle::DeviceHandle;

pub struct DeviceBridge {
    mqtt_client: AsyncClient,
    device_handle: DeviceHandle,

    pub power: Option<bool>,
    pub brightness: Option<u16>,
    pub color_temperature: Option<u16>,
}

impl DeviceBridge {
    pub fn new(
        mqtt_client: AsyncClient,
        device_handle: DeviceHandle,
    ) -> Self {
        Self {
            device_handle,
            mqtt_client,

            power: None,
            brightness: None,
            color_temperature: None,
        }
    }

    fn device_topic_path(&self) -> String {
        format!(
            "{}/{}",
            self.device_handle.device_type.to_string().to_lowercase().replace(" ", "-"),
            self.device_handle.serial_number().unwrap()
        )
    }

    fn device_root_topic(&self) -> String {
        format!("logitech/{}", self.device_topic_path())
    }

    pub async fn publish_state(&mut self) {
        let power = self.device_handle.is_on().unwrap();

        if self.power.is_some() && self.power.unwrap() == power {
            return;
        }

        self.power = Some(power);
        self
            .publish(
                format!("{}/state", self.device_root_topic()),
                match power {
                    true => "ON",
                    false => "OFF",
                }.to_string(),
            )
            .await;
        // self.mqtt_client
        //     .publish(
        //         format!("{}/state", self.device_root_topic()),
        //         QoS::AtLeastOnce,
        //         false,
        //         match power {
        //             true => "ON",
        //             false => "OFF",
        //         }.as_bytes().to_vec(),
        //     )
        //     .await
        //     .unwrap();
    }

    pub fn save_brightness(&mut self, brightness: u16) {
        self.brightness = Some(brightness)
    }
    pub async fn publish_brightness(&mut self) {
        let brightness = self.device_handle.brightness_in_lumen().unwrap();

        if self.brightness.is_some() && self.brightness.unwrap() == brightness {
            return;
        }

        self.save_brightness(brightness);
        self
            .publish(
                format!("{}/brightness", self.device_root_topic()),
                brightness.to_string(),
            )
            .await;
        // self.mqtt_client
        //     .publish(
        //         format!("{}/brightness", self.device_root_topic()),
        //         QoS::AtLeastOnce,
        //         false,
        //         brightness.to_string().as_bytes().to_vec(),
        //     )
        //     .await
        //     .unwrap();
    }

    pub async fn publish_color_temperature(&mut self) {
        let color_temperature = self.device_handle.temperature_in_kelvin().unwrap();

        if self.color_temperature.is_some() && self.color_temperature.unwrap() == color_temperature {
            return;
        }

        self.color_temperature = Some(color_temperature);
        self
            .publish(
                format!("{}/color_temp", self.device_root_topic()),
                color_temperature.to_string(),
            )
            .await;

        // self.mqtt_client
        //     .publish(
        //         format!("{}/color_temp", self.device_root_topic()),
        //         QoS::AtLeastOnce,
        //         false,
        //         color_temperature.to_string().as_bytes().to_vec(),
        //     )
        //     .await
        //     .unwrap();
    }

    pub async fn publish_discovery(&mut self) {
        // self
        //     .publish(
        //         format!("homeassistant/light/{}", self.device_root_topic()),
        //         self.to_discovery_json().to_string(),
        //     )
        //     .await;

        self.mqtt_client
            .publish(
                format!(
                    "homeassistant/light/logitech-{}/{}/config",
                    self.device_handle.device_type.to_string().to_lowercase().replace(" ", "-"),
                    self.device_handle.serial_number().unwrap()
                ),
                QoS::AtLeastOnce,
                true,
                self.to_discovery_json().to_string().as_bytes().to_vec(),
            )
            .await
            .unwrap();
    }

    pub async fn publish_availability(&mut self) {
        self
            .publish(
                format!("{}/status", self.device_root_topic()),
                "online".to_string(),
            )
            .await;

        // self.mqtt_client
        //     .publish(
        //         format!("{}/status", self.device_root_topic()),
        //         QoS::AtLeastOnce,
        //         false,
        //         "online".as_bytes().to_vec(),
        //     )
        //     .await
        //     .unwrap()
    }

    fn to_discovery_json(&self) -> Value {
        // rescaled_value = (value / 255.0) * (max - min) + min.

        return json!({
            "~": self.device_root_topic(),
            "device_class": "light",
            "supported_color_modes": [
                "color_temp"
                // todo: add rgb to beam lx
            ],
            "unique_id": self.device_handle.serial_number().unwrap(),
            "device": {
                "name": format!("Logitech {}", self.device_handle.device_type.to_string()),
                "identifiers": self.device_handle.serial_number().unwrap(),
                "manufacturer": "Logitech",
                "model": self.device_handle.device_type.to_string(),
                "serial_number": self.device_handle.serial_number().unwrap(),
            },

            "availability_topic": "~/status",
            "state_topic": "~/state",
            "command_topic": "~/state/set",


            // "brightness_scale": self.device_handle.maximum_brightness_in_lumen() - self.device_handle.minimum_brightness_in_lumen(),
            "brightness_scale": 255,
            // rescale brightness to 0-255
            "brightness_template": format!(
                "{{{{ (((value - {min}) / ({max} - {min})) * 255) | round(0) }}}}",
                min = self.device_handle.minimum_brightness_in_lumen(),
                max = self.device_handle.maximum_brightness_in_lumen(),
            ),
            "brightness_state_topic": "~/brightness",
            "brightness_command_topic": "~/brightness/set",
            "brightness_command_template": format!(
                "{{{{ (((value / 255 | float) * ({max} - {min})) + {min}) | round(0) }}}}",
                min = self.device_handle.minimum_brightness_in_lumen(),
                max = self.device_handle.maximum_brightness_in_lumen(),
            ),

            /*
            "min_mireds": self.device_handle.minimum_temperature_in_mireds(),
            "max_mireds": self.device_handle.maximum_temperature_in_mireds(),

            "color_temp_state_topic": "~/color_temp",
            "color_temp_template": "{{ (1000000 / value | float) | round(0) }}",

            "color_temp_command_topic": "~/color_temp/set",
            "color_temp_command_template": format!(
                "{{ [[(1000000 / value | float) | round(0), {min}] | max, {max}] | min }}",
                min = self.device_handle.minimum_temperature_in_kelvin(),
                max = self.device_handle.maximum_temperature_in_kelvin()
            )
             */
        });
    }

    async fn publish(&mut self, topic: String, payload: String) {
        self.mqtt_client
            .publish(
                topic,
                QoS::AtLeastOnce,
                false,
                payload.as_bytes().to_vec(),
            )
            .await
            .unwrap()
    }
}
