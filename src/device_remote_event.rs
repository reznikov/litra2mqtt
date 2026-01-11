use std::fmt;

use rumqttc::Publish;
use serde::{Deserialize, Serialize};

use crate::state::DeviceState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub brightness: Option<u16>,
    #[serde(default)]
    pub color_temp: Option<u16>,
    #[serde(default)]
    pub revision: Option<u64>,
    #[serde(default)]
    pub timestamp: Option<u64>,
    #[serde(default)]
    pub origin: Option<String>,
}

pub enum DeviceRemoteEvent {
    StateUpdate(DeviceState),
    Unknown(String),
}

impl fmt::Display for DeviceRemoteEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceRemoteEvent::StateUpdate(state) => write!(
                f, 
                "State update: power={}, brightness={}, temp={}, rev={}",
                state.power, state.brightness, state.temperature, state.revision
            ),
            DeviceRemoteEvent::Unknown(message) => write!(f, "Unknown event: {:?}", message),
        }
    }
}

impl DeviceRemoteEvent {
    pub fn from_mqtt_message(message: Publish, current_state: &DeviceState) -> Self {
        let payload = std::str::from_utf8(&message.payload).unwrap();

        match serde_json::from_str::<CommandPayload>(payload) {
            Ok(cmd) => {
                // Start with current state
                let mut new_state = current_state.clone();
                
                // Update fields that are present in the command
                if let Some(state_str) = cmd.state {
                    new_state.power = state_str == "ON";
                }
                
                if let Some(brightness) = cmd.brightness {
                    new_state.brightness = brightness;
                }
                
                if let Some(color_temp) = cmd.color_temp {
                    // Convert mireds to Kelvin
                    // Validate color_temp to avoid division by zero
                    if color_temp > 0 {
                        new_state.temperature = (1000000.0 / color_temp as f64) as u16;
                    } else {
                        // Invalid color_temp, keep current value
                        // Log warning in real implementation
                    }
                }
                
                // Use provided revision/timestamp/origin or default
                if let Some(revision) = cmd.revision {
                    new_state.revision = revision;
                }
                
                if let Some(timestamp) = cmd.timestamp {
                    new_state.timestamp = timestamp;
                }
                
                if let Some(origin) = cmd.origin {
                    new_state.origin = origin;
                }
                
                DeviceRemoteEvent::StateUpdate(new_state)
            }
            Err(e) => {
                DeviceRemoteEvent::Unknown(format!("Failed to parse command: {} - payload: {}", e, payload))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_power_command() {
        let payload = r#"{"state": "ON"}"#;
        let current = DeviceState::new(false, 100, 3000, "test".to_string());
        
        let publish = Publish::new(
            "litra_bridge/device1/command",
            rumqttc::QoS::AtLeastOnce,
            payload.as_bytes(),
        );
        
        match DeviceRemoteEvent::from_mqtt_message(publish, &current) {
            DeviceRemoteEvent::StateUpdate(state) => {
                assert_eq!(state.power, true);
                assert_eq!(state.brightness, 100); // unchanged
            }
            _ => panic!("Expected StateUpdate"),
        }
    }

    #[test]
    fn test_parse_full_command() {
        let payload = r#"{"state": "ON", "brightness": 200, "color_temp": 250}"#;
        let current = DeviceState::new(false, 100, 3000, "test".to_string());
        
        let publish = Publish::new(
            "litra_bridge/device1/command",
            rumqttc::QoS::AtLeastOnce,
            payload.as_bytes(),
        );
        
        match DeviceRemoteEvent::from_mqtt_message(publish, &current) {
            DeviceRemoteEvent::StateUpdate(state) => {
                assert_eq!(state.power, true);
                assert_eq!(state.brightness, 200);
                assert_eq!(state.temperature, 4000); // 1000000 / 250 = 4000K
            }
            _ => panic!("Expected StateUpdate"),
        }
    }

    #[test]
    fn test_parse_zero_color_temp() {
        let payload = r#"{"color_temp": 0}"#;
        let current = DeviceState::new(false, 100, 3000, "test".to_string());
        
        let publish = Publish::new(
            "litra_bridge/device1/command",
            rumqttc::QoS::AtLeastOnce,
            payload.as_bytes(),
        );
        
        match DeviceRemoteEvent::from_mqtt_message(publish, &current) {
            DeviceRemoteEvent::StateUpdate(state) => {
                // Should keep current temperature when color_temp is 0
                assert_eq!(state.temperature, 3000);
            }
            _ => panic!("Expected StateUpdate"),
        }
    }
}
