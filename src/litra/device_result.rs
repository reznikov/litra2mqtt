use crate::litra::device_error::DeviceError;

/// The [`Result`] of a Litra device operation.
pub type DeviceResult<T> = Result<T, DeviceError>;
