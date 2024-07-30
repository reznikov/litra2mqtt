use std::fmt;
use hidapi::HidApi;
use crate::litra::device::Device;
use crate::litra::device_result::DeviceResult;

/// Litra context.
///
/// This can be used to list available devices.
pub struct Litra(HidApi);

impl fmt::Debug for Litra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Litra").finish()
    }
}

impl Litra {
    /// Initialize a new Litra context.
    pub fn new() -> DeviceResult<Self> {
        Ok(HidApi::new().map(Litra)?)
    }

    /// Returns an [`Iterator`] of connected devices supported by this library.
    pub fn get_connected_devices(&self) -> impl Iterator<Item=Device<'_>> {
        self.0
            .device_list()
            .filter_map(|device_info| Device::try_from(device_info).ok())
    }

    /// Retrieve the underlying hidapi context.
    #[must_use]
    pub fn hidapi(&self) -> &HidApi {
        &self.0
    }
}
