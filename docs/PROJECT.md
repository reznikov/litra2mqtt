# Litra2MQTT - Logitech Litra ↔ Home Assistant MQTT Bridge

## Overview

Litra2MQTT is a Rust-based MQTT bridge that synchronizes the state of **Logitech Litra USB lamps** with **Home Assistant** using MQTT. It provides bidirectional communication, allowing both Home Assistant commands and physical button presses on the lamp to be synchronized across both systems.

## Features

### Multi-Device Support
- Detects and manages multiple connected Litra lamps concurrently
- Each lamp has a stable device ID based on serial number and model
- Devices can be hot-plugged (detected on startup)

### Bidirectional Synchronization
- **MQTT → Lamp**: Receives commands from Home Assistant and applies them to the physical lamp
- **Lamp → MQTT**: Detects button presses on the lamp and publishes state changes to MQTT

### Loop Prevention
- Implements instance-based origin tracking to prevent feedback loops
- State changes from the bridge are tagged with an instance ID
- The bridge ignores its own published messages
- Uses revision numbers and timestamps for conflict resolution

### Resilience
- MQTT auto-reconnection with resubscription
- Devices marked offline/online based on connection status
- Last Will and Testament (LWT) for graceful disconnect handling

### Home Assistant Integration
- Automatic device discovery via MQTT Discovery protocol
- Publishes retained configuration to `homeassistant/light/<device_id>/config`
- Supports brightness and color temperature control
- Provides availability status for each device

## Architecture

### Module Structure

```
src/
├── main.rs                      # Application entry point
├── state.rs                     # State management with loop prevention
├── device_bridge.rs            # Device-to-MQTT bridge functionality
├── device_hardware_event.rs    # Hardware event parsing
├── device_remote_event.rs      # MQTT command parsing
├── handle_device_events.rs     # Hardware event handler
├── handle_remote_events.rs     # MQTT event handler
├── mqtt_topic.rs               # Topic structure helpers
├── device_handle.rs            # Device handle extensions
└── util.rs                     # Utility functions
```

### Key Components

1. **State Manager** (`state.rs`)
   - Tracks device state with revision numbers
   - Implements loop prevention logic
   - Handles conflict resolution

2. **Device Bridge** (`device_bridge.rs`)
   - Manages MQTT publishing for devices
   - Generates Home Assistant discovery messages
   - Formats state messages with metadata

3. **Event Handlers**
   - `handle_device_events.rs`: Monitors hardware button events
   - `handle_remote_events.rs`: Processes MQTT commands

## Topic Structure

The bridge uses the following MQTT topic structure:

```
litra_bridge/<device_id>/state        # Device state (published by bridge)
litra_bridge/<device_id>/command      # Commands (subscribed by bridge)
litra_bridge/<device_id>/availability # Device availability (online/offline)
litra_bridge/status                   # Bridge status (online/offline, LWT)
homeassistant/light/<device_id>/config # HA discovery (retained)
```

### Device ID Format

Device IDs follow the pattern: `logitech_<model>_<serial>`

Example: `logitech_litra-glow_AB123456`

### State Message Format

State messages are published as JSON:

```json
{
  "state": "ON",
  "brightness": 250,
  "color_temp": 250,
  "revision": 5,
  "timestamp": 1673456789000,
  "origin": "instance-uuid"
}
```

### Command Message Format

Commands are received as JSON:

```json
{
  "state": "ON",
  "brightness": 200,
  "color_temp": 300
}
```

## Requirements

- Rust 1.70+ (edition 2021)
- USB access to Logitech Litra devices
- MQTT broker (e.g., Mosquitto)
- Home Assistant (optional, for integration)

## System Dependencies

On Linux:
```bash
sudo apt-get install libudev-dev libhidapi-dev pkg-config
```

## Building

```bash
cargo build --release
```

The binary will be located at `target/release/litra2mqtt`.

## Configuration

Configuration is done via environment variables. See [RUNBOOK.md](RUNBOOK.md) for details.

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover:
- Loop prevention logic
- Conflict resolution
- Topic formatting
- State management
- Command parsing

## License

See LICENSE file for details.
