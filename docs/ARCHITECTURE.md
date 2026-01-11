# Litra2MQTT Architecture

## System Overview

Litra2MQTT is a bidirectional bridge that synchronizes state between Logitech Litra USB devices and Home Assistant via MQTT. The architecture is designed for reliability, multi-device support, and conflict-free operation.

```
┌─────────────────┐         ┌──────────────────┐         ┌──────────────────┐
│  Litra Device   │◄───────►│  Litra2MQTT      │◄───────►│  MQTT Broker     │
│  (USB HID)      │  USB    │  Bridge          │  TCP    │  (Mosquitto)     │
└─────────────────┘         └──────────────────┘         └──────────────────┘
                                     │                             │
                                     │                             │
                                     └─────────────────────────────┼───────►
                                                                   │
                                                          ┌─────────▼─────────┐
                                                          │ Home Assistant    │
                                                          │ (MQTT Integration)│
                                                          └───────────────────┘
```

## Core Components

### 1. State Management (`state.rs`)

The state management module is the heart of the loop prevention and conflict resolution system.

#### DeviceState Structure

```rust
pub struct DeviceState {
    pub power: bool,
    pub brightness: u16,
    pub temperature: u16,
    pub revision: u64,        // Monotonic counter
    pub timestamp: u64,       // Milliseconds since epoch
    pub origin: String,       // Instance ID that created this state
}
```

#### DeviceStateManager

Manages state transitions and enforces business rules:

```rust
pub struct DeviceStateManager {
    current_state: DeviceState,
    last_published_state: Option<DeviceState>,
    instance_id: String,
}
```

**Key methods:**
- `update_from_hardware()`: Processes local device changes
- `update_from_mqtt()`: Processes remote MQTT commands
- `mark_published()`: Tracks what has been sent to MQTT
- `needs_publish()`: Determines if state needs republishing

### 2. Loop Prevention

Loop prevention prevents feedback loops where the bridge publishes a state change to MQTT, Home Assistant echoes it back, and the bridge re-publishes it infinitely.

#### Strategy: Instance-Based Origin Tracking

Each bridge instance has a unique `instance_id` (UUID). Every state change includes this ID in the `origin` field.

**Rules:**
1. When publishing state to MQTT, include the bridge's `instance_id` in `origin`
2. When receiving commands from MQTT:
   - If `origin` matches the bridge's `instance_id` → **IGNORE** (it's our own message)
   - If `origin` is different → **PROCESS** (it's from another source)

**Example Flow:**

```
1. User presses button on Litra device
   ├─ Bridge detects hardware event
   ├─ Creates state with origin="bridge-instance-abc"
   └─ Publishes to MQTT: litra_bridge/device1/state

2. Home Assistant receives state update
   └─ Updates internal state (doesn't send command back)

3. Bridge receives its own published message
   ├─ Checks origin field: "bridge-instance-abc"
   ├─ Matches own instance_id → IGNORE
   └─ Loop prevented ✓
```

**Alternative scenario - HA sends command:**

```
1. Home Assistant sends command
   └─ Publishes to: litra_bridge/device1/command
      (no origin or origin="home-assistant")

2. Bridge receives command
   ├─ Checks origin: different from own instance_id
   ├─ Applies to physical device
   └─ Hardware ACK triggers state publish with origin="bridge-instance-abc"

3. Bridge receives its own state publish
   └─ Ignores (same as scenario above)
```

### 3. Conflict Resolution

When both hardware and MQTT commands occur simultaneously, conflicts must be resolved deterministically.

#### Strategy: Revision + Timestamp Ordering

**Primary rule**: Higher revision number wins
**Tie-breaker**: Higher timestamp wins

```rust
pub fn should_override(&self, other: &DeviceState) -> bool {
    if self.revision != other.revision {
        self.revision > other.revision
    } else {
        self.timestamp > other.timestamp
    }
}
```

**Revision Management:**
- Hardware events increment revision by 1
- MQTT commands include their revision
- Each state change gets a new timestamp

**Example Conflict:**

```
Time T0: Device state = {power: false, revision: 5, timestamp: 1000}

Time T1: Hardware button pressed
  ├─ New state: {power: true, revision: 6, timestamp: 1001}
  └─ Published to MQTT

Time T2: MQTT command arrives (was sent at T0.5)
  └─ Command: {power: false, revision: 4, timestamp: 950}

Conflict Resolution:
  ├─ Current state revision: 6
  ├─ Incoming state revision: 4
  ├─ 6 > 4 → Current wins
  └─ MQTT command IGNORED ✓

Result: Device stays ON (hardware button press respected)
```

### 4. Event Flow

#### Hardware → MQTT Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Hardware Event Loop (per device)                            │
│                                                              │
│  1. Poll USB HID device (100ms interval)                   │
│  2. Parse hardware event                                    │
│  3. Update DeviceStateManager                               │
│  4. If state changed:                                       │
│     ├─ Increment revision                                   │
│     ├─ Set timestamp                                        │
│     ├─ Set origin = instance_id                            │
│     ├─ Publish to litra_bridge/<device_id>/state           │
│     └─ Mark as published                                    │
└─────────────────────────────────────────────────────────────┘
```

**Hardware Events:**
- Power button press: `0x11 0xFF 0x04 0x00 <state> 0x00`
- Brightness change: `0x11 0xFF 0x04 0x10 <hi> <lo>`
- Temperature change: `0x11 0xFF 0x04 0x20 <hi> <lo>`
- Acknowledgments: Various `0x1C`, `0x4C`, `0x9C` patterns

#### MQTT → Hardware Flow

```
┌─────────────────────────────────────────────────────────────┐
│ MQTT Event Loop                                             │
│                                                              │
│  1. Receive on litra_bridge/<device_id>/command            │
│  2. Parse JSON command                                      │
│  3. Extract device_id from topic                           │
│  4. Get current device state                                │
│  5. Check origin field (loop prevention)                    │
│  6. If origin != instance_id:                              │
│     ├─ Apply power change (if different)                   │
│     ├─ Apply brightness change (if different)              │
│     ├─ Apply temperature change (if different)             │
│     └─ Hardware ACK triggers state publish                 │
└─────────────────────────────────────────────────────────────┘
```

### 5. MQTT Topic Design

#### Topic Hierarchy

```
litra_bridge/
├── <device_id>/
│   ├── state          # Current state (published by bridge)
│   ├── command        # Commands (subscribed by bridge)
│   └── availability   # Online/offline status
└── status             # Bridge-wide status (LWT)

homeassistant/
└── light/
    └── <device_id>/
        └── config     # Discovery message (retained)
```

#### State Message Schema

```json
{
  "state": "ON|OFF",
  "brightness": 0-400,          // Lumen (device-specific range)
  "color_temp": 153-370,        // Mireds
  "revision": 0-18446744073709551615,
  "timestamp": 1673456789123,   // Milliseconds
  "origin": "instance-uuid"
}
```

#### Command Message Schema

```json
{
  "state": "ON|OFF",            // Optional
  "brightness": 0-400,          // Optional, lumen
  "color_temp": 153-370,        // Optional, mireds
  "revision": 5,                // Optional
  "timestamp": 1673456789123,   // Optional
  "origin": "source-id"         // Optional
}
```

**Note**: Commands can include only the fields that should change. Missing fields retain their current values.

### 6. Home Assistant Discovery

The bridge publishes MQTT Discovery messages for automatic integration.

#### Discovery Message Structure

```json
{
  "name": "Logitech Litra Glow",
  "unique_id": "logitech_litra-glow_AB123456",
  "object_id": "logitech_litra-glow_AB123456",
  "device_class": "light",
  "supported_color_modes": ["color_temp"],
  
  "device": {
    "name": "Logitech Litra Glow",
    "identifiers": ["logitech_litra-glow_AB123456"],
    "manufacturer": "Logitech",
    "model": "Litra Glow",
    "serial_number": "AB123456"
  },

  "availability_topic": "litra_bridge/logitech_litra-glow_AB123456/availability",
  "state_topic": "litra_bridge/logitech_litra-glow_AB123456/state",
  "command_topic": "litra_bridge/logitech_litra-glow_AB123456/command",
  "state_value_template": "{{ value_json.state }}",
  
  "brightness_scale": 250,
  "brightness_value_template": "{{ (value_json.brightness | int) - 150 }}",
  "brightness_command_template": "{{ (value | int) + 150 }}",

  "min_mireds": 153,
  "max_mireds": 370,
  "color_temp_value_template": "{{ value_json.color_temp | int }}",

  "json_attributes_topic": "litra_bridge/logitech_litra-glow_AB123456/state",
  "json_attributes_template": "{{ {'revision': value_json.revision, 'timestamp': value_json.timestamp, 'origin': value_json.origin} | tojson }}",

  "schema": "json"
}
```

**Key Points:**
- Published to: `homeassistant/light/<device_id>/config`
- Retained: Yes
- QoS: 1 (At Least Once)

### 7. Concurrency Model

#### Per-Device Worker Tasks

Each connected device gets its own Tokio task:

```
┌────────────────────────────────────────────────────┐
│ Main Thread                                        │
│  ├─ Enumerate devices                             │
│  ├─ For each device:                              │
│  │   └─ tokio::spawn(device_worker)               │
│  └─ tokio::spawn(mqtt_event_loop)                 │
└────────────────────────────────────────────────────┘
         │                           │
         │                           │
    ┌────▼───────┐             ┌────▼───────┐
    │ Device 1   │             │ Device 2   │
    │ Worker     │             │ Worker     │
    │            │             │            │
    │ - Poll HID │             │ - Poll HID │
    │ - Manage   │             │ - Manage   │
    │   state    │             │   state    │
    │ - Publish  │             │ - Publish  │
    └────────────┘             └────────────┘
```

**Benefits:**
- No global locks on hot paths
- Each device operates independently
- Isolation: one device failure doesn't affect others

#### Shared State

Devices are stored in a shared `HashMap`:

```rust
Arc<Mutex<HashMap<String, (DeviceHandle, DeviceBridge)>>>
```

- Protected by `tokio::sync::Mutex`
- Only locked during:
  - Initial device enumeration
  - MQTT command routing (brief lock to get device)
- Device workers don't hold the lock during operations

### 8. Resilience Mechanisms

#### MQTT Reconnection

The bridge handles MQTT disconnections gracefully:

```rust
loop {
    match eventloop.poll().await {
        Ok(Event::Incoming(Packet::ConnAck(_))) => {
            // Reconnected!
            // Resubscribe to all topics
            client.subscribe("litra_bridge/+/command", QoS::AtLeastOnce).await;
            // Republish bridge status
            client.publish("litra_bridge/status", QoS::AtLeastOnce, true, "online").await;
        }
        Err(e) => {
            error!("MQTT Error: {} - will attempt to reconnect", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            // Loop continues, rumqttc auto-reconnects
        }
    }
}
```

**Features:**
- Automatic reconnection (handled by `rumqttc`)
- Resubscription on reconnect
- State republication on reconnect
- 5-second backoff on errors

#### Last Will and Testament (LWT)

The bridge configures LWT on connection:

```rust
let lwt = LastWill::new(
    "litra_bridge/status",
    "offline",
    QoS::AtLeastOnce,
    true, // retain
);
```

**Behavior:**
- If bridge crashes or loses connection, broker publishes "offline"
- Clients (Home Assistant) see the bridge as unavailable
- On graceful shutdown, bridge publishes "offline" before disconnecting

#### Device Hotplug

**Current Implementation:**
- Devices are detected on startup
- Hot-plugging requires restart (limitation of current implementation)

**Future Enhancement:**
- Periodic re-enumeration to detect new devices
- Graceful handling of device removal

### 9. Error Handling

#### Device Errors

```rust
match device_handle.set_on(true) {
    Ok(_) => { /* success */ }
    Err(e) => {
        error!("Error setting power: {}", e);
        // Don't crash, continue operation
    }
}
```

**Strategy:** Log errors but continue operation. Device may have been unplugged or encountered temporary issue.

#### MQTT Errors

```rust
if let Err(e) = client.publish(...).await {
    error!("Error publishing: {}", e);
    // Message lost, but client will reconnect and state will resync
}
```

**Strategy:** Log and continue. State will eventually resync through hardware polling.

#### Parse Errors

```rust
match serde_json::from_str::<CommandPayload>(payload) {
    Ok(cmd) => { /* process */ }
    Err(e) => {
        error!("Failed to parse command: {}", e);
        // Ignore malformed commands
    }
}
```

**Strategy:** Ignore malformed input, log for debugging.

## Testing Strategy

### Unit Tests

Located in each module with `#[cfg(test)]`:

1. **Loop Prevention Tests** (`state.rs`)
   - Same origin → ignored
   - Different origin → processed

2. **Conflict Resolution Tests** (`state.rs`)
   - Higher revision wins
   - Timestamp tie-breaking

3. **Topic Formatting Tests** (`mqtt_topic.rs`)
   - Correct topic structure
   - Device ID extraction

4. **Command Parsing Tests** (`device_remote_event.rs`)
   - JSON parsing
   - Partial updates
   - Error handling

### Integration Testing

**Mock-based approach:**
- Mock MQTT client
- Mock device adapter
- Verify message flow

**Live testing:**
- Use test MQTT broker
- Physical Litra device
- Automated test scenarios

## Performance Characteristics

### Latency

- **Hardware → MQTT**: ~100-200ms (polling interval + network)
- **MQTT → Hardware**: <50ms (instant on command receive)

### CPU Usage

- Minimal: ~1-2% per device (mostly sleeping in polls)
- Tokio async runtime scales efficiently

### Memory Usage

- Base: ~5-10 MB
- Per device: ~1 MB (mostly state and buffers)

### Network Bandwidth

- State updates: ~200 bytes per change
- Typical usage: <1 KB/s per device

## Security Considerations

1. **MQTT Authentication**: Required (username/password)
2. **No TLS**: Not implemented (broker-level TLS recommended)
3. **USB Permissions**: Requires access to HID devices
4. **No Input Validation**: Device commands trusted (risk: malformed MQTT messages)

**Recommendations:**
- Use MQTT with TLS
- Restrict MQTT user permissions
- Run bridge as non-root user
- Use systemd with restricted capabilities

## Future Enhancements

1. **Hot-plug Support**: Dynamic device detection
2. **TLS Support**: Native MQTT over TLS
3. **Configuration File**: Alternative to env vars
4. **Metrics Export**: Prometheus endpoint
5. **Advanced Conflict Resolution**: Time-based tie-breaking rules (e.g., "device wins within 100ms")
6. **RGB Support**: For Litra Beam LX
7. **Web UI**: Status dashboard and configuration
