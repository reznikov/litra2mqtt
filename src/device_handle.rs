use litra::DeviceHandle;

pub trait DeviceHandleWithMerids {
    fn minimum_temperature_in_mireds(&self) -> u16;
    fn maximum_temperature_in_mireds(&self) -> u16;
}

impl DeviceHandleWithMerids for DeviceHandle {
    fn minimum_temperature_in_mireds(&self) -> u16 {
        153
    }

    fn maximum_temperature_in_mireds(&self) -> u16 {
        370
    }
}
