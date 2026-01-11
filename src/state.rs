use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Represents the state of a device with revision tracking for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceState {
    pub power: bool,
    pub brightness: u16,
    pub temperature: u16,
    pub revision: u64,
    pub timestamp: u64,
    pub origin: String,
}

impl DeviceState {
    pub fn new(power: bool, brightness: u16, temperature: u16, origin: String) -> Self {
        Self {
            power,
            brightness,
            temperature,
            revision: 0,
            timestamp: Self::current_timestamp(),
            origin,
        }
    }

    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self.timestamp = Self::current_timestamp();
        self
    }

    pub fn increment_revision(mut self) -> Self {
        self.revision += 1;
        self.timestamp = Self::current_timestamp();
        self
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Determines if this state should take precedence over another state
    /// Uses revision number first, then timestamp as tie-breaker
    pub fn should_override(&self, other: &DeviceState) -> bool {
        if self.revision != other.revision {
            self.revision > other.revision
        } else {
            self.timestamp > other.timestamp
        }
    }
}

/// Manages state for a single device with loop prevention
pub struct DeviceStateManager {
    current_state: DeviceState,
    last_published_state: Option<DeviceState>,
    instance_id: String,
}

impl DeviceStateManager {
    pub fn new(initial_state: DeviceState, instance_id: String) -> Self {
        Self {
            current_state: initial_state,
            last_published_state: None,
            instance_id,
        }
    }

    /// Update state from a hardware event
    pub fn update_from_hardware(&mut self, power: Option<bool>, brightness: Option<u16>, temperature: Option<u16>) -> Option<DeviceState> {
        let mut changed = false;

        if let Some(p) = power {
            if self.current_state.power != p {
                self.current_state.power = p;
                changed = true;
            }
        }

        if let Some(b) = brightness {
            if self.current_state.brightness != b {
                self.current_state.brightness = b;
                changed = true;
            }
        }

        if let Some(t) = temperature {
            if self.current_state.temperature != t {
                self.current_state.temperature = t;
                changed = true;
            }
        }

        if changed {
            self.current_state = self.current_state.clone().increment_revision();
            self.current_state.origin = self.instance_id.clone();
            Some(self.current_state.clone())
        } else {
            None
        }
    }

    /// Update state from an MQTT command
    /// Returns Some(state) if the change should be applied, None if it should be ignored (loop prevention)
    pub fn update_from_mqtt(&mut self, incoming_state: DeviceState) -> Option<DeviceState> {
        // Loop prevention: ignore our own published messages
        if incoming_state.origin == self.instance_id {
            return None;
        }

        // Check if this is a newer state
        if !incoming_state.should_override(&self.current_state) {
            return None;
        }

        self.current_state = incoming_state.clone();
        Some(incoming_state)
    }

    /// Mark state as published to MQTT
    pub fn mark_published(&mut self) {
        self.last_published_state = Some(self.current_state.clone());
    }

    /// Check if current state needs to be published
    pub fn needs_publish(&self) -> bool {
        match &self.last_published_state {
            None => true,
            Some(last) => last != &self.current_state,
        }
    }

    pub fn current_state(&self) -> &DeviceState {
        &self.current_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_should_override_by_revision() {
        let state1 = DeviceState::new(true, 100, 3000, "instance1".to_string()).with_revision(1);
        let state2 = DeviceState::new(false, 200, 4000, "instance2".to_string()).with_revision(2);
        
        assert!(state2.should_override(&state1));
        assert!(!state1.should_override(&state2));
    }

    #[test]
    fn test_state_should_override_by_timestamp() {
        let mut state1 = DeviceState::new(true, 100, 3000, "instance1".to_string()).with_revision(1);
        state1.timestamp = 1000;
        
        let mut state2 = DeviceState::new(false, 200, 4000, "instance2".to_string()).with_revision(1);
        state2.timestamp = 2000;
        
        assert!(state2.should_override(&state1));
        assert!(!state1.should_override(&state2));
    }

    #[test]
    fn test_loop_prevention_same_origin() {
        let initial = DeviceState::new(false, 100, 3000, "instance1".to_string());
        let mut manager = DeviceStateManager::new(initial, "instance1".to_string());
        
        let incoming = DeviceState::new(true, 200, 4000, "instance1".to_string()).with_revision(5);
        
        // Should ignore messages from same origin
        assert!(manager.update_from_mqtt(incoming).is_none());
    }

    #[test]
    fn test_loop_prevention_different_origin() {
        let initial = DeviceState::new(false, 100, 3000, "instance1".to_string());
        let mut manager = DeviceStateManager::new(initial, "instance1".to_string());
        
        let incoming = DeviceState::new(true, 200, 4000, "instance2".to_string()).with_revision(5);
        
        // Should accept messages from different origin
        assert!(manager.update_from_mqtt(incoming).is_some());
    }

    #[test]
    fn test_hardware_update_increments_revision() {
        let initial = DeviceState::new(false, 100, 3000, "instance1".to_string());
        let mut manager = DeviceStateManager::new(initial, "instance1".to_string());
        
        let result = manager.update_from_hardware(Some(true), None, None);
        assert!(result.is_some());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.revision, 1);
        assert_eq!(new_state.power, true);
    }

    #[test]
    fn test_needs_publish() {
        let initial = DeviceState::new(false, 100, 3000, "instance1".to_string());
        let mut manager = DeviceStateManager::new(initial, "instance1".to_string());
        
        // Initially should need publish
        assert!(manager.needs_publish());
        
        manager.mark_published();
        
        // After marking published, should not need publish
        assert!(!manager.needs_publish());
        
        // After hardware change, should need publish again
        manager.update_from_hardware(Some(true), None, None);
        assert!(manager.needs_publish());
    }

    #[test]
    fn test_conflict_resolution_newer_revision_wins() {
        let initial = DeviceState::new(false, 100, 3000, "instance1".to_string()).with_revision(1);
        let mut manager = DeviceStateManager::new(initial, "instance1".to_string());
        
        // Older revision should be ignored
        let old_incoming = DeviceState::new(true, 200, 4000, "instance2".to_string()).with_revision(0);
        assert!(manager.update_from_mqtt(old_incoming).is_none());
        
        // Newer revision should be accepted
        let new_incoming = DeviceState::new(true, 200, 4000, "instance2".to_string()).with_revision(5);
        assert!(manager.update_from_mqtt(new_incoming).is_some());
    }
}
