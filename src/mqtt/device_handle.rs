use crate::litra::device_handle::DeviceHandle;

pub trait DeviceHandleDiscovery {
    fn minimum_temperature_in_mireds(&self) -> u16;
    fn maximum_temperature_in_mireds(&self) -> u16;
}

impl DeviceHandleDiscovery for DeviceHandle {
    fn minimum_temperature_in_mireds(&self) -> u16 {
        153
    }

    fn maximum_temperature_in_mireds(&self) -> u16 {
        370
    }
}
