use std::fmt;

/// The model of the device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    /// Logitech [Litra Glow][glow] streaming light with TrueSoft.
    ///
    /// [glow]: https://www.logitech.com/products/lighting/litra-glow.html
    LitraGlow,
    /// Logitech [Litra Beam][beam] LED streaming key light with TrueSoft.
    ///
    /// [beam]: https://www.logitechg.com/products/cameras-lighting/litra-beam-streaming-light.html
    LitraBeam,
    /// Logitech [Litra Beam LX][beamlx] dual-sided RGB streaming key light.
    ///
    /// [beamlx]: https://www.logitechg.com/products/cameras-lighting/litra-beam-lx-led-light.html
    LitraBeamLX,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::LitraGlow => write!(f, "Litra Glow"),
            DeviceType::LitraBeam => write!(f, "Litra Beam"),
            DeviceType::LitraBeamLX => write!(f, "Litra Beam LX"),
        }
    }
}
