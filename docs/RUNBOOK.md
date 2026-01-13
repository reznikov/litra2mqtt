# Litra2MQTT Runbook

## Configuration

Litra2MQTT is configured entirely through environment variables. You can set these in your shell, a `.env` file, or through systemd service configuration.

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `MQTT_HOST` | MQTT broker hostname or IP | `192.168.1.100` or `mqtt.example.com` |
| `MQTT_PORT` | MQTT broker port | `1883` |
| `MQTT_USERNAME` | MQTT authentication username | `litra2mqtt` |
| `MQTT_PASSWORD` | MQTT authentication password | `secretpassword` |

### Optional Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MQTT_CLIENT_ID` | MQTT client identifier | `litra2mqtt` |
| `LITRA_INSTANCE_ID` | Unique instance identifier for loop prevention | Random UUID |
| `RUST_LOG` | Logging level | `info` |

### Example .env File

Create a `.env` file in the same directory as the binary:

```env
MQTT_HOST=192.168.1.100
MQTT_PORT=1883
MQTT_USERNAME=litra2mqtt
MQTT_PASSWORD=my_secure_password
MQTT_CLIENT_ID=litra2mqtt
RUST_LOG=info
```

## Running the Bridge

### Command Line

```bash
# Export environment variables
export MQTT_HOST=192.168.1.100
export MQTT_PORT=1883
export MQTT_USERNAME=litra2mqtt
export MQTT_PASSWORD=secretpassword
export RUST_LOG=info

# Run the bridge
./target/release/litra2mqtt
```

### Using .env File

```bash
# Place your .env file in the current directory
./target/release/litra2mqtt
```

The application will automatically load variables from `.env`.

### Systemd Service

Create `/etc/systemd/system/litra2mqtt.service`:

```ini
[Unit]
Description=Litra2MQTT Bridge
After=network.target mosquitto.service
Wants=mosquitto.service

[Service]
Type=simple
User=litra2mqtt
Group=plugdev
WorkingDirectory=/opt/litra2mqtt
Environment="MQTT_HOST=192.168.1.100"
Environment="MQTT_PORT=1883"
Environment="MQTT_USERNAME=litra2mqtt"
Environment="MQTT_PASSWORD=secretpassword"
Environment="RUST_LOG=info"
ExecStart=/opt/litra2mqtt/litra2mqtt
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable litra2mqtt
sudo systemctl start litra2mqtt
sudo systemctl status litra2mqtt
```

## Home Assistant Configuration

### Automatic Discovery

The bridge automatically publishes MQTT Discovery messages. In Home Assistant:

1. Ensure MQTT integration is configured
2. Start the bridge with Litra devices connected
3. Devices should appear automatically in Home Assistant under "Integrations" → "MQTT"

### Manual Configuration (if needed)

If automatic discovery doesn't work, add manually to `configuration.yaml`:

```yaml
mqtt:
  light:
    - name: "Logitech Litra Glow"
      unique_id: "logitech_litra-glow_AB123456"
      state_topic: "litra_bridge/logitech_litra-glow_AB123456/state"
      command_topic: "litra_bridge/logitech_litra-glow_AB123456/command"
      availability_topic: "litra_bridge/logitech_litra-glow_AB123456/availability"
      schema: json
      brightness: true
      color_temp: true
      min_mireds: 153
      max_mireds: 370
```

## Verifying Operation

### Check Logs

```bash
# With systemd
sudo journalctl -u litra2mqtt -f

# Command line
RUST_LOG=debug ./target/release/litra2mqtt
```

Expected log output:
```
INFO  litra2mqtt] Starting litra2mqtt bridge with instance_id: <uuid>
INFO  litra2mqtt::handle_device_events] Found device: logitech_litra-glow_AB123456
INFO  litra2mqtt::handle_device_events] Starting device event loop for logitech_litra-glow_AB123456
INFO  litra2mqtt::handle_remote_events] MQTT Connected! Subscribing to command topics...
```

### Test MQTT Communication

Subscribe to device topics:

```bash
# Monitor state changes
mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/state" -v

# Monitor availability
mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/availability" -v
```

Send a test command:

```bash
mosquitto_pub -h localhost -u litra2mqtt -P password \
  -t "litra_bridge/logitech_litra-glow_AB123456/command" \
  -m '{"state":"ON","brightness":200,"color_temp":250}'
```

### Test Physical Button

1. Press the power button on your Litra device
2. Check MQTT for state update:
   ```bash
   mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/state" -v
   ```
3. Verify state change appears in Home Assistant

### Test Home Assistant Control

1. Open Home Assistant
2. Navigate to the Litra device
3. Toggle power or adjust brightness/temperature
4. Verify the physical lamp responds

## Troubleshooting

### Device Not Detected

**Problem**: Bridge starts but no devices are found.

**Solutions**:
1. Check USB connection
2. Verify USB permissions:
   ```bash
   # Add user to plugdev group
   sudo usermod -a -G plugdev $USER
   
   # Create udev rule for Litra devices
   echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="046d", MODE="0666"' | sudo tee /etc/udev/rules.d/99-litra.rules
   sudo udevadm control --reload-rules
   ```
3. Restart the bridge

### MQTT Connection Failed

**Problem**: Bridge can't connect to MQTT broker.

**Solutions**:
1. Verify broker is running:
   ```bash
   sudo systemctl status mosquitto
   ```
2. Check network connectivity:
   ```bash
   ping <MQTT_HOST>
   ```
3. Verify credentials with mosquitto_pub:
   ```bash
   mosquitto_pub -h <MQTT_HOST> -p <MQTT_PORT> -u <USERNAME> -P <PASSWORD> -t test -m test
   ```
4. Check broker logs:
   ```bash
   sudo journalctl -u mosquitto -f
   ```

### State Not Updating

**Problem**: Physical button presses don't update Home Assistant.

**Solutions**:
1. Check device event logs:
   ```bash
   RUST_LOG=debug ./litra2mqtt 2>&1 | grep "hardware"
   ```
2. Verify MQTT state topic receives updates:
   ```bash
   mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/state" -v
   ```
3. Check Home Assistant MQTT integration is working

### Commands Not Applied to Device

**Problem**: Home Assistant commands don't affect the physical lamp.

**Solutions**:
1. Monitor command topic:
   ```bash
   mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/command" -v
   ```
2. Check bridge logs for errors:
   ```bash
   sudo journalctl -u litra2mqtt -n 50
   ```
3. Verify device USB connection is stable
4. Check for permission issues

### Feedback Loop / Oscillation

**Problem**: State oscillates between two values.

**Solutions**:
1. This should be prevented by loop prevention logic
2. Check that each bridge instance has a unique `LITRA_INSTANCE_ID`
3. Verify only one bridge instance is running per device
4. Check logs for loop prevention messages:
   ```bash
   sudo journalctl -u litra2mqtt | grep "origin"
   ```

### Device Goes Offline

**Problem**: Device shows as unavailable in Home Assistant.

**Solutions**:
1. Check USB connection
2. Restart the bridge
3. Check availability topic:
   ```bash
   mosquitto_sub -h localhost -u litra2mqtt -P password -t "litra_bridge/+/availability" -v
   ```
4. Verify bridge is still running:
   ```bash
   sudo systemctl status litra2mqtt
   ```

### Multiple Instances

**Problem**: Running multiple bridge instances causes conflicts.

**Solutions**:
1. Set unique `LITRA_INSTANCE_ID` for each instance:
   ```bash
   export LITRA_INSTANCE_ID=bridge-01
   ```
2. Each instance will ignore messages from other instances
3. Useful for high availability or load distribution

## Monitoring

### Important Metrics to Monitor

1. **Bridge Status**: `litra_bridge/status`
   - Should be "online"
   - LWT ensures it goes "offline" on ungraceful disconnect

2. **Device Availability**: `litra_bridge/<device_id>/availability`
   - Should be "online" for connected devices

3. **State Updates**: `litra_bridge/<device_id>/state`
   - Monitor revision numbers for proper sequencing
   - Check timestamps for timing issues

### Log Levels

Set `RUST_LOG` for different verbosity:

- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (recommended)
- `debug`: Detailed debugging information
- `trace`: Very verbose (not recommended for production)

Example:
```bash
export RUST_LOG=debug
```

## Performance Tuning

### Polling Interval

The bridge polls hardware events every 100ms when no events are detected. This is configurable in the code but provides a good balance between responsiveness and CPU usage.

### MQTT QoS

The bridge uses QoS 1 (At Least Once) for all messages, ensuring reliable delivery while maintaining good performance.

### Retained Messages

The following messages are retained on the broker:
- Discovery messages (`homeassistant/light/<device_id>/config`)
- State messages (`litra_bridge/<device_id>/state`)
- Availability messages (`litra_bridge/<device_id>/availability`)
- Bridge status (`litra_bridge/status`)

This ensures Home Assistant has the latest state even after a restart.

## Backup and Recovery

### Configuration Backup

Backup your `.env` file or systemd service configuration.

### MQTT Broker Backup

If using Mosquitto, backup:
```bash
sudo cp /var/lib/mosquitto/mosquitto.db /backup/location/
```

### Recovery

1. Restore configuration files
2. Start MQTT broker
3. Start litra2mqtt
4. Devices will automatically republish discovery and state

## Upgrading

1. Stop the service:
   ```bash
   sudo systemctl stop litra2mqtt
   ```

2. Backup current binary:
   ```bash
   sudo cp /opt/litra2mqtt/litra2mqtt /opt/litra2mqtt/litra2mqtt.backup
   ```

3. Replace binary with new version:
   ```bash
   sudo cp target/release/litra2mqtt /opt/litra2mqtt/
   ```

4. Start the service:
   ```bash
   sudo systemctl start litra2mqtt
   ```

5. Verify operation:
   ```bash
   sudo systemctl status litra2mqtt
   sudo journalctl -u litra2mqtt -n 50
   ```
