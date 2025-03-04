use std::fmt;

use rumqttc::Publish;

use crate::{mqtt_topic, util};

pub enum DeviceRemoteEvent {
    Power(bool),
    BrightnessInLumen(u16),
    ColorTemperatureInKelvin(u16),
    Unknown(String),
    // todo: rgb?
}

impl fmt::Display for DeviceRemoteEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceRemoteEvent::Power(value) => write!(f, "Power {}", value),
            DeviceRemoteEvent::BrightnessInLumen(value) => write!(f, "Brightness {} lm", value),
            DeviceRemoteEvent::ColorTemperatureInKelvin(value) => {
                write!(f, "Color temperature {} K", value)
            }
            DeviceRemoteEvent::Unknown(message) => write!(f, "Unknown event: {:?}", message),
        }
    }
}

impl DeviceRemoteEvent {
    pub fn from_mqtt_message(message: Publish) -> Self {
        let payload = std::str::from_utf8(&message.payload).unwrap();

        if let Some(property) = util::get_property(&message.topic) {
            match String::as_str(&property) {
                mqtt_topic::POWER => DeviceRemoteEvent::power(&payload),
                mqtt_topic::BRIGHTNESS => DeviceRemoteEvent::brightness(&payload),
                mqtt_topic::TEMPERATURE => DeviceRemoteEvent::color_temperature(&payload),
                _ => DeviceRemoteEvent::Unknown(format!("Unknown property {}", property)),
            }
        } else {
            DeviceRemoteEvent::Unknown(format!("Unknown mqtt message {:?}", message))
        }
    }

    fn power(payload: &str) -> Self {
        match payload {
            "ON" => DeviceRemoteEvent::Power(true),
            "OFF" => DeviceRemoteEvent::Power(false),
            _ => DeviceRemoteEvent::Unknown(format!("Invalid power state {}", payload)),
        }
    }

    fn brightness(payload: &str) -> Self {
        match payload.parse::<u16>() {
            Ok(brightness) => DeviceRemoteEvent::BrightnessInLumen(brightness),
            _ => DeviceRemoteEvent::Unknown(format!("Invalid brightness {}", payload)),
        }
    }

    fn color_temperature(payload: &str) -> Self {
        match payload.parse::<u16>() {
            Ok(temperature) => DeviceRemoteEvent::ColorTemperatureInKelvin(temperature),
            _ => DeviceRemoteEvent::Unknown(format!("Invalid color temperature {}", payload)),
        }
    }
}
