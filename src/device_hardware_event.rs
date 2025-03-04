use std::fmt;

pub enum DeviceHardwareEvent {
    Power(bool),
    PowerAck,
    BrightnessInLumen(u16),
    BrightnessInLumenAck,
    ColorTemperatureInKelvin(u16),
    ColorTemperatureInKelvinAck,
    Unknown([u8; 6]),
    // todo: rgb for LitraBeamLX
}

impl fmt::Display for DeviceHardwareEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceHardwareEvent::Power(value) => write!(f, "Power {}", value),
            DeviceHardwareEvent::PowerAck => write!(f, "Power ack"),
            DeviceHardwareEvent::BrightnessInLumen(value) => write!(f, "Brightness {} lm", value),
            DeviceHardwareEvent::BrightnessInLumenAck => write!(f, "Brightness ack"),
            DeviceHardwareEvent::ColorTemperatureInKelvin(value) => {
                write!(f, "Color temperature {} K", value)
            }
            DeviceHardwareEvent::ColorTemperatureInKelvinAck => write!(f, "Color temperature ack"),
            DeviceHardwareEvent::Unknown(bytes) => write!(f, "Unknown event: {:?}", bytes),
        }
    }
}

impl DeviceHardwareEvent {
    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        match bytes {
            [0x11, 0xFF, 0x04, 0x00, b1, _] => DeviceHardwareEvent::Power(*b1 == 1),
            [0x11, 0xFF, 0x04, 0x1C, _, _] => DeviceHardwareEvent::PowerAck,
            [0x11, 0xFF, 0x04, 0x10, b1, b2] => {
                let brightness = u16::from(*b1) * 256 + u16::from(*b2);
                DeviceHardwareEvent::BrightnessInLumen(brightness)
            }
            [0x11, 0xFF, 0x04, 0x4C, _, _] => DeviceHardwareEvent::BrightnessInLumenAck,
            [0x11, 0xFF, 0x04, 0x20, b1, b2] => {
                let color_temperature = u16::from(*b1) * 256 + u16::from(*b2);
                DeviceHardwareEvent::ColorTemperatureInKelvin(color_temperature)
            }
            [0x11, 0xFF, 0x04, 0x9C, _, _] => DeviceHardwareEvent::ColorTemperatureInKelvinAck,
            _ => DeviceHardwareEvent::Unknown(*bytes),
        }
    }
}
