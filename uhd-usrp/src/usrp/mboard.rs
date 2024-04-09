use std::{ffi::CString, ptr::addr_of_mut};

use crate::{
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    try_uhd, Result, SensorValue, TimeSpec, Usrp,
};

use super::subdev_spec::SubdevSpec;

/// Provides access to motherboard properties.
pub struct Motherboard<'a> {
    usrp: &'a Usrp,
    mboard: usize,
}

impl<'a> Motherboard<'a> {
    /// Create a new [`Motherboard`].
    ///
    /// This constructor does not perform any I/O; this struct is just a wrapper
    /// for motherboard properties.
    pub(crate) fn new<'b>(usrp: &'a Usrp, mboard: usize) -> Self {
        Self { usrp, mboard }
    }

    /// Get the current clock source of the motherboard.
    ///
    /// # Errors
    ///
    /// Returns an error if the clock source could not be retrieved,
    /// or if the returned string is not valid UTF-8.
    pub fn clock_source(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(16);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_clock_source(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    /// Get a list of clock sources of the motherboard.
    ///
    /// # Errors
    ///
    /// Returns an error if the list of clock sources could not be retrieved,
    /// or if the returned strings are not valid UTF-8.
    pub fn clock_sources(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_clock_sources(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Fetch information about a daughterboard EEPROM.
    pub fn dboard_eeprom(&self, unit: &str, slot: &str) -> Result<DaughterboardEeprom> {
        let unit = CString::new(unit).unwrap();
        let slot = CString::new(slot).unwrap();
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_dboard_eeprom_t>::new(
            uhd_usrp_sys::uhd_dboard_eeprom_make,
            uhd_usrp_sys::uhd_dboard_eeprom_free,
        )?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_dboard_eeprom(
                self.usrp.handle().as_mut_ptr(),
                handle.as_mut_ptr(),
                unit.as_ptr(),
                slot.as_ptr(),
                self.mboard,
            )
        })?;
        Ok(DaughterboardEeprom::from_handle(handle))
    }

    pub fn eeprom(&self) -> Result<MotherboardEeprom> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_mboard_eeprom_t>::new(
            uhd_usrp_sys::uhd_mboard_eeprom_make,
            uhd_usrp_sys::uhd_mboard_eeprom_free,
        )?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_eeprom(
                self.usrp.handle().as_mut_ptr(),
                handle.as_mut_ptr(),
                self.mboard,
            )
        })?;
        Ok(MotherboardEeprom::new(handle))
    }

    /// Get a list of GPIO banks associated with this motherboard.
    pub fn gpio_bank_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_gpio_banks(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Get a GPIO bank associated with this motherboard.
    pub fn gpio_bank(&self, name: &str) -> GpioBank {
        GpioBank::new(self.usrp, self.mboard, name)
    }

    /// Get the time when the last pps pulse occurred.
    ///
    /// For RFNoC devices with multiple timekeepers, this returns the time of the first timekeeper.
    /// To access specific timekeepers, use the corresponding RFNoC APIs.
    pub fn last_pps_time(&self) -> Result<TimeSpec> {
        let mut full_seconds: i64 = 0;
        let mut frac_seconds: f64 = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_last_pps(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                addr_of_mut!(full_seconds),
                addr_of_mut!(frac_seconds),
            )
        })?;
        Ok(TimeSpec::from_parts(full_seconds, frac_seconds))
    }

    /// Get the master clock rate in Hz.
    pub fn master_clock_rate(&self) -> Result<f64> {
        let mut result = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_master_clock_rate(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    /// Get canonical name for this USRP motherboard.
    pub fn name(&self) -> Result<String> {
        let mut result = FfiString::with_capacity(32);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_name(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                result.as_mut_ptr().cast(),
                result.max_chars(),
            )
        })?;
        result.into_string()
    }

    /// Get the Rx frontend specification currently in use.
    pub fn rx_subdev_spec(&self) -> Result<SubdevSpec> {
        let spec = SubdevSpec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                spec.handle().as_mut_ptr(),
            )
        })?;
        Ok(spec)
    }

    /// Get a list of possible motherboard sensor names.
    pub fn sensor_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_sensor_names(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Get a motherboard sensor value.
    pub fn sensor_value(&self, name: &str) -> Result<SensorValue> {
        let name = CString::new(name).unwrap();
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_sensor_value_t>::new(
            uhd_usrp_sys::uhd_sensor_value_make,
            uhd_usrp_sys::uhd_sensor_value_free,
        )?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_sensor(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.mboard,
                handle.as_mut_mut_ptr(),
            )
        })?;
        Ok(SensorValue::new(handle))
    }

    /// Set the clock source for the motherboard.
    ///
    /// This sets the source of the frequency reference, typically a 10 MHz signal.
    /// In order to frequency-align multiple USRPs, it is necessary to connect all of them
    /// to a common reference and provide them with the same clock source.
    ///
    /// Typical values for source are 'internal', 'external'.
    /// Refer to the specific device manual for a full list of options.
    ///
    /// This function does not force a re-initialization of the underlying hardware when the value does not change.
    ///
    /// # Errors
    ///
    /// An error is returned if the value for for source is not available for this device.
    /// Calling [`clock_sources`] will return a list of valid clock sources.
    pub fn set_clock_source(&self, source: &str) -> Result<()> {
        let source = CString::new(source).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_clock_source(
                self.usrp.handle().as_mut_ptr(),
                source.as_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Send the clock signal to an output connector.
    ///
    /// This call is only applicable on devices with reference outputs.
    /// By default, the reference output will be enabled for ease of use.
    /// This call may be used to enable or disable the output.
    ///
    /// # Errors
    ///
    /// Returns an error if the device does not support this operation.
    pub fn set_clock_source_out(&mut self, en: bool) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_clock_source_out(
                self.usrp.handle().as_mut_ptr(),
                en,
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Set the Rx frontend specification.
    pub fn set_rx_subdev_str(&mut self, subdev: &str) -> Result<()> {
        let sudev = SubdevSpec::from_str(subdev);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                sudev.handle().as_mut_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Sets the time registers on the USRP immediately.
    ///
    /// This will set the tick count on the timekeepers of all devices as soon as possible.
    /// It is done serially for multiple timekeepers, so times across multiple timekeepers will not be synchronized.
    pub fn set_time(&self, time: TimeSpec) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_now(
                self.usrp.handle().as_mut_ptr(),
                time.full_secs() as i64,
                time.frac_secs(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Set the time registers on the USRP at the next PPS rising edge.
    ///
    /// This will set the tick count on the timekeepers of all devices on the next rising edge of the PPS trigger signal.
    /// It is important to note that this means the time may not be set for up to 1 second after this call is made,
    /// so it is recommended to wait for 1 second after this call before making any calls that depend on the time to ensure
    /// that the time registers will be in a known state prior to use.
    ///
    /// Note: Because this call sets the time on the next PPS edge,
    /// the time spec supplied should correspond to the next pulse (i.e. current time + 1 second).
    ///
    /// Note: Make sure to not call this shortly before the next PPS edge.
    /// This should be called with plenty of time before the next PPS edge
    /// to ensure that all timekeepers on all devices will execute this command on the same PPS edge.
    /// If not, timekeepers could be unsynchronized in time by exactly one second.
    /// If in doubt, use set_time_unknown_pps() which will take care of this issue (but will also take longer to execute).
    pub fn set_time_next_pps(&mut self, time: TimeSpec) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_next_pps(
                self.usrp.handle().as_mut_ptr(),
                time.full_secs() as i64,
                time.frac_secs(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Set the time source for the USRP device
    ///
    /// This sets the method of time synchronization, typically a pulse per second signal.
    /// In order to time-align multiple USRPs, it is necessary to connect all of them to a common reference and provide them with the same time source.
    /// Typical values for source are 'internal', 'external'. Refer to the specific device manual for a full list of options.
    ///
    /// This function does not force a re-initialization of the underlying hardware when the value does not change.
    ///
    /// # Errors
    ///
    /// Returns an error if the value for source is not available for this device.
    pub fn set_time_source(&self, source: &str) -> Result<()> {
        let source = CString::new(source).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_source(
                self.usrp.handle().as_mut_ptr(),
                source.as_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Send the time signal (PPS) to an output connector.
    ///
    /// This call is only applicable on devices with PPS outputs.
    /// By default, the PPS output will be enabled for ease of use.
    /// This call may be used to enable or disable the output.
    ///
    /// # Errors
    ///
    /// Returns an error if the device does not support this operation.
    pub fn set_time_source_out(&mut self, en: bool) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_source_out(
                self.usrp.handle().as_mut_ptr(),
                en,
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Set the Tx frontend specification.
    pub fn set_tx_subdev_str(&mut self, subdev: &str) -> Result<()> {
        let sudev = SubdevSpec::from_str(subdev);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_tx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                sudev.handle().as_mut_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Perform an write on the user configuration register bus.
    ///
    /// These only exist if the user has implemented custom setting registers in the device FPGA.
    ///
    /// # Errors
    ///
    /// Returns an error if this API is not implemented.
    pub fn set_user_register(&self, addr: u8, data: u32) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_user_register(
                self.usrp.handle().as_mut_ptr(),
                addr,
                data,
                self.mboard,
            )
        })?;
        Ok(())
    }

    /// Get the current time in the usrp time registers.
    ///
    /// For RFNoC devices with multiple timekeepers, this returns the time of the first timekeeper.
    /// To access specific timekeepers, use the corresponding RFNoC APIs.
    pub fn time(&self) -> Result<TimeSpec> {
        let mut full_secs = 0;
        let mut frac_secs = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_now(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                addr_of_mut!(full_secs),
                addr_of_mut!(frac_secs),
            )
        })?;
        Ok(TimeSpec::from_parts(full_secs, frac_secs))
    }

    /// Get the currently set time source.
    pub fn time_source(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(16);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_source(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    /// Get a list of possible time sources.
    pub fn time_sources(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_sources(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Get the Tx frontend specification currently in use.
    pub fn tx_subdev_spec(&self) -> Result<SubdevSpec> {
        let spec = SubdevSpec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                spec.handle().as_mut_ptr(),
            )
        })?;
        Ok(spec)
    }
}

pub struct GpioBank<'a> {
    usrp: &'a Usrp,
    mboard: usize,
    bank: CString,
}

impl<'a> GpioBank<'a> {
    pub(crate) fn new(usrp: &'a Usrp, mboard: usize, bank: &str) -> Self {
        Self {
            usrp,
            mboard,
            bank: CString::new(bank).unwrap(),
        }
    }

    pub fn attr(&self, name: &str) -> Result<u32> {
        let name = CString::new(name).unwrap();
        let mut result = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_gpio_attr(
                self.usrp.handle().as_mut_ptr(),
                self.bank.as_ptr(),
                name.as_ptr(),
                self.mboard,
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    pub fn set_attr(&self, name: &str, mask: u32, value: u32) -> Result<()> {
        let name = CString::new(name).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_gpio_attr(
                self.usrp.handle().as_mut_ptr(),
                self.bank.as_ptr(),
                name.as_ptr(),
                value,
                mask,
                self.mboard,
            )
        })?;
        Ok(())
    }
}

pub struct MotherboardEeprom {
    handle: OwnedHandle<uhd_usrp_sys::uhd_mboard_eeprom_t>,
}

impl MotherboardEeprom {
    pub(crate) fn new(handle: OwnedHandle<uhd_usrp_sys::uhd_mboard_eeprom_t>) -> Self {
        Self { handle }
    }

    pub fn value(&self, key: &str) -> Option<String> {
        let key = CString::new(key).unwrap();
        let mut value = FfiString::with_capacity(32);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_mboard_eeprom_get_value(
                self.handle.as_mut_ptr(),
                key.as_ptr(),
                value.as_mut_ptr(),
                value.max_chars(),
            )
        })
        .ok();
        value.into_string().ok()
    }

    pub fn set_value(&self, key: &str, value: &str) {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            uhd_usrp_sys::uhd_mboard_eeprom_set_value(
                self.handle.as_mut_ptr(),
                key.as_ptr(),
                value.as_ptr(),
            );
        }
    }
}

pub struct DaughterboardEeprom {
    handle: OwnedHandle<uhd_usrp_sys::uhd_dboard_eeprom_t>,
}

impl DaughterboardEeprom {
    pub fn new() -> Self {
        Self {
            handle: OwnedHandle::new(
                uhd_usrp_sys::uhd_dboard_eeprom_make,
                uhd_usrp_sys::uhd_dboard_eeprom_free,
            )
            .unwrap(),
        }
    }

    pub(crate) fn from_handle(handle: OwnedHandle<uhd_usrp_sys::uhd_dboard_eeprom_t>) -> Self {
        Self { handle }
    }

    /// The ID for the daughterboard type.
    pub fn id(&self) -> Result<String> {
        let mut id = FfiString::with_capacity(16);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_get_id(
                self.handle.as_mut_ptr(),
                id.as_mut_ptr(),
                id.max_chars(),
            )
        })?;
        Ok(id.into_string().unwrap())
    }

    pub fn set_id(&self, id: &str) {
        let id = CString::new(id).unwrap();
        unsafe { uhd_usrp_sys::uhd_dboard_eeprom_set_id(self.handle.as_mut_ptr(), id.as_ptr()) };
    }

    /// The unique serial number.
    pub fn serial_number(&self) -> Result<String> {
        let mut id = FfiString::with_capacity(16);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_get_serial(
                self.handle.as_mut_ptr(),
                id.as_mut_ptr(),
                id.max_chars(),
            )
        })?;
        Ok(id.into_string().unwrap())
    }

    pub fn set_serial_number(&self, serial: &str) {
        let serial = CString::new(serial).unwrap();
        unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_set_serial(self.handle.as_mut_ptr(), serial.as_ptr());
        }
    }

    /// The hardware revision number.
    pub fn revision(&self) -> Result<i32> {
        let mut value = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_get_revision(
                self.handle.as_mut_ptr(),
                addr_of_mut!(value),
            )
        })?;
        Ok(value)
    }

    pub fn set_revision(&self, value: i32) {
        unsafe { uhd_usrp_sys::uhd_dboard_eeprom_set_revision(self.handle.as_mut_ptr(), value) };
    }
}
