use hidapi::DeviceInfo;
use crate::litra::constants::{USAGE_PAGE, VENDOR_ID};
use crate::litra::device_error::DeviceError;
use crate::litra::device_handle::DeviceHandle;
use crate::litra::device_result::DeviceResult;
use crate::litra::device_type::DeviceType;
use crate::litra::litra::Litra;
use crate::litra::util::device_type_from_product_id;

/// A device that can be used.
#[derive(Debug)]
pub struct Device<'a> {
    device_info: &'a DeviceInfo,
    device_type: DeviceType,
}

impl<'a> TryFrom<&'a DeviceInfo> for Device<'a> {
    type Error = DeviceError;

    fn try_from(device_info: &'a DeviceInfo) -> Result<Self, DeviceError> {
        if device_info.vendor_id() != VENDOR_ID || device_info.usage_page() != USAGE_PAGE {
            return Err(DeviceError::Unsupported);
        }
        device_type_from_product_id(device_info.product_id())
            .map(|device_type| Device {
                device_info,
                device_type,
            })
            .ok_or(DeviceError::Unsupported)
    }
}

impl Device<'_> {
    /// The model of the device.
    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        self.device_info
    }

    /// The model of the device.
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Opens the device and returns a [`DeviceHandle`] that can be used for getting and setting the
    /// device status.
    pub fn open(&self, context: &Litra) -> DeviceResult<DeviceHandle> {
        let hid_device = self.device_info.open_device(context.hidapi())?;
        Ok(DeviceHandle {
            hid_device,
            device_type: self.device_type,
        })
    }
}
