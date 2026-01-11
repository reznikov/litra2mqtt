# Litra2MQTT

A Rust-based MQTT bridge that synchronizes **Logitech Litra USB lamps** with **Home Assistant**, supporting bidirectional communication and multi-device management.

## Features

✅ **Multi-device support** - Manage multiple Litra lamps concurrently  
✅ **Bidirectional sync** - Home Assistant commands ↔ Physical button presses  
✅ **Loop prevention** - Intelligent origin tracking prevents feedback loops  
✅ **Conflict resolution** - Deterministic handling of simultaneous changes  
✅ **Auto-discovery** - Automatic Home Assistant integration via MQTT Discovery  
✅ **Resilient** - Auto-reconnect, LWT support, graceful error handling  

## Quick Start

### Prerequisites

- Rust 1.70+ (edition 2021)
- USB access to Logitech Litra devices
- MQTT broker (e.g., Mosquitto)
- Home Assistant with MQTT integration (optional)

### System Dependencies

On Linux:
```bash
sudo apt-get install libudev-dev libhidapi-dev pkg-config
```

### Installation

```bash
# Clone the repository
git clone https://github.com/reznikov/litra2mqtt.git
cd litra2mqtt

# Build the binary
cargo build --release

# The binary will be at: target/release/litra2mqtt
```

### Configuration

Create a `.env` file:

```env
MQTT_HOST=192.168.1.100
MQTT_PORT=1883
MQTT_USERNAME=litra2mqtt
MQTT_PASSWORD=your_password
RUST_LOG=info
```

### Running

```bash
./target/release/litra2mqtt
```

The bridge will:
1. Detect connected Litra devices
2. Connect to MQTT broker
3. Publish Home Assistant discovery messages
4. Begin syncing state bidirectionally

## Documentation

- **[PROJECT.md](docs/PROJECT.md)** - Overview, features, and architecture
- **[RUNBOOK.md](docs/RUNBOOK.md)** - Configuration, operation, and troubleshooting
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Technical architecture and design decisions

## Usage

### Home Assistant Integration

Devices automatically appear in Home Assistant after starting the bridge:

1. Start the bridge with devices connected
2. Go to Home Assistant → Settings → Devices & Services → MQTT
3. Your Litra devices should appear automatically

### MQTT Topics

```
litra_bridge/<device_id>/state        # Device state (published)
litra_bridge/<device_id>/command      # Commands (subscribed)
litra_bridge/<device_id>/availability # Device availability
```

### Example Command

```bash
mosquitto_pub -h localhost -u litra2mqtt -P password \
  -t "litra_bridge/logitech_litra-glow_AB123456/command" \
  -m '{"state":"ON","brightness":200,"color_temp":250}'
```

## Testing

```bash
cargo test
```

All tests include:
- Loop prevention logic
- Conflict resolution
- Topic formatting
- Command parsing

## Supported Devices

- Logitech Litra Glow
- Logitech Litra Beam
- Logitech Litra Beam LX (basic support, no RGB yet)

## Architecture Highlights

### Loop Prevention
Uses instance-based origin tracking - each state change is tagged with the bridge's unique instance ID. The bridge ignores its own published messages.

### Conflict Resolution
Uses revision numbers + timestamps for deterministic conflict resolution. Hardware button presses are never lost.

### Per-Device Workers
Each device runs in its own Tokio task, providing isolation and parallel operation.

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Troubleshooting

See [RUNBOOK.md](docs/RUNBOOK.md) for detailed troubleshooting steps.

**Common issues:**

- **Device not detected**: Check USB permissions and udev rules
- **MQTT connection failed**: Verify broker is running and credentials are correct
- **State not updating**: Check logs with `RUST_LOG=debug`

## License

[Include your license here]

## Acknowledgments

- Built with [litra-rs](https://github.com/timvw/litra-rs) for device communication
- Uses [rumqttc](https://github.com/bytebeamio/rumqtt) for MQTT connectivity
- Powered by [Tokio](https://tokio.rs/) for async runtime
