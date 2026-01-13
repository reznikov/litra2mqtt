pub const TEMPERATURE: &str = "temperature";
pub const BRIGHTNESS: &str = "brightness";
pub const POWER: &str = "state";
pub const AVAILABILITY: &str = "availability";

// Topic format: litra_bridge/<device_id>/<topic>
pub fn base_topic(device_id: &str) -> String {
    format!("litra_bridge/{}", device_id)
}

pub fn state_topic(device_id: &str) -> String {
    format!("{}/state", base_topic(device_id))
}

pub fn command_topic(device_id: &str) -> String {
    format!("{}/command", base_topic(device_id))
}

pub fn availability_topic(device_id: &str) -> String {
    format!("{}/availability", base_topic(device_id))
}

pub fn discovery_topic(device_id: &str) -> String {
    format!("homeassistant/light/{}/config", device_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_topic() {
        assert_eq!(base_topic("device123"), "litra_bridge/device123");
    }

    #[test]
    fn test_state_topic() {
        assert_eq!(state_topic("device123"), "litra_bridge/device123/state");
    }

    #[test]
    fn test_command_topic() {
        assert_eq!(command_topic("device123"), "litra_bridge/device123/command");
    }

    #[test]
    fn test_availability_topic() {
        assert_eq!(availability_topic("device123"), "litra_bridge/device123/availability");
    }

    #[test]
    fn test_discovery_topic() {
        assert_eq!(discovery_topic("device123"), "homeassistant/light/device123/config");
    }
}

