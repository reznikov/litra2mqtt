use hidapi::HidDevice;
use crate::litra::device_error::DeviceError;
use crate::litra::device_result::DeviceResult;
use crate::litra::device_type::DeviceType;
use crate::litra::util::{generate_get_brightness_in_lumen_bytes, generate_get_temperature_in_kelvin_bytes, generate_is_on_bytes, generate_set_brightness_in_lumen_bytes, generate_set_on_bytes, generate_set_temperature_in_kelvin_bytes};

/// The handle of an opened device that can be used for getting and setting the device status.
#[derive(Debug)]
pub struct DeviceHandle {
    pub(crate) hid_device: HidDevice,
    pub(crate) device_type: DeviceType,
}

impl DeviceHandle {
    /// The model of the device.
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn serial_number(&self) -> DeviceResult<String> {
        match self.hid_device.get_device_info() {
            Ok(device_info) => match device_info.serial_number() {
                Some(serial_number) => Ok(serial_number.to_owned()),
                None => Err(DeviceError::NoSerial),
            },
            Err(error) => Err(DeviceError::HidError(error)),
        }
    }

    pub fn read_device(&self, response_buffer: &mut [u8; 20]) {
        self.hid_device.read(&mut response_buffer[..]).unwrap();
    }

    /// Queries the current power status of the device. Returns `true` if the device is currently on.
    pub fn is_on(&self) -> DeviceResult<bool> {
        let message = generate_is_on_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][4] == 1)
    }

    /// Sets the power status of the device. Turns the device on if `true` is passed and turns it
    /// of on `false`.
    pub fn set_on(&self, on: bool) -> DeviceResult<()> {
        let message = generate_set_on_bytes(&self.device_type, on);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Queries the device's current brightness in Lumen.
    pub fn brightness_in_lumen(&self) -> DeviceResult<u16> {
        let message = generate_get_brightness_in_lumen_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(
            u16::from(response_buffer[..response][4]) * 256
                + u16::from(response_buffer[..response][5])
        )
    }

    /// Sets the device's brightness in Lumen.
    pub fn set_brightness_in_lumen(&self, brightness_in_lumen: u16) -> DeviceResult<()> {
        if brightness_in_lumen < self.minimum_brightness_in_lumen()
            || brightness_in_lumen > self.maximum_brightness_in_lumen()
        {
            return Err(DeviceError::InvalidBrightness(brightness_in_lumen));
        }

        let message = generate_set_brightness_in_lumen_bytes(&self.device_type, brightness_in_lumen);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Returns the minimum brightness supported by the device in Lumen.
    #[must_use]
    pub fn minimum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 20,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 30,
        }
    }

    /// Returns the maximum brightness supported by the device in Lumen.
    #[must_use]
    pub fn maximum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 250,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 400,
        }
    }

    /// Queries the device's current color temperature in Kelvin.
    pub fn temperature_in_kelvin(&self) -> DeviceResult<u16> {
        let message = generate_get_temperature_in_kelvin_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(
            u16::from(response_buffer[..response][4]) * 256
                + u16::from(response_buffer[..response][5])
        )
    }

    /// Sets the device's color temperature in Kelvin.
    pub fn set_temperature_in_kelvin(&self, temperature_in_kelvin: u16) -> DeviceResult<()> {
        if temperature_in_kelvin < self.minimum_temperature_in_kelvin()
            || temperature_in_kelvin > self.maximum_temperature_in_kelvin()
            || (temperature_in_kelvin % 100) != 0
        {
            return Err(DeviceError::InvalidTemperature(temperature_in_kelvin));
        }

        let message = generate_set_temperature_in_kelvin_bytes(&self.device_type, temperature_in_kelvin);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Returns the minimum color temperature supported by the device in Kelvin.
    #[must_use]
    pub fn minimum_temperature_in_kelvin(&self) -> u16 {
        2700
    }

    /// Returns the maximum color temperature supported by the device in Kelvin.
    #[must_use]
    pub fn maximum_temperature_in_kelvin(&self) -> u16 {
        6500
    }
}
