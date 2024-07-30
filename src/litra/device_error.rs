use std::error::Error;
use std::fmt;
use hidapi::HidError;

#[derive(Debug)]
pub enum DeviceError {
    /// Tried to use a device that is not supported.
    Unsupported,
    /// A device doesn't have a serial number
    NoSerial,
    /// Tried to set an invalid brightness value.
    InvalidBrightness(u16),
    /// Tried to set an invalid temperature value.
    InvalidTemperature(u16),
    /// A [`hidapi`] operation failed.
    HidError(HidError),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::Unsupported => write!(f, "Device is not supported"),
            DeviceError::NoSerial => write!(f, "Device does not have a serial number"),
            DeviceError::InvalidBrightness(value) => write!(f, "Brightness {} lm is not supported", value),
            DeviceError::InvalidTemperature(value) => write!(f, "Temperature {} K is not supported", value),
            DeviceError::HidError(error) => write!(f, "HID error occurred: {}", error),
        }
    }
}

impl Error for DeviceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeviceError::HidError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<HidError> for DeviceError {
    fn from(error: HidError) -> Self {
        DeviceError::HidError(error)
    }
}
