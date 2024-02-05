use std::{ffi::CString, ptr::addr_of_mut};

use crate::{
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    try_uhd, DeviceTime, Result, SensorValue, Usrp,
};

pub struct Motherboard<'a> {
    usrp: &'a Usrp,
    mboard: usize,
}

impl<'a> Motherboard<'a> {
    pub(crate) fn new<'b>(usrp: &'a Usrp, mboard: usize) -> Self {
        Self { usrp, mboard }
    }

    pub fn name(&self) -> Result<String> {
        let mut result = FfiString::<16>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_name(
                self.usrp.handle(),
                self.mboard,
                result.as_mut_ptr().cast(),
                result.max_chars(),
            )
        })?;
        result.into_string()
    }

    pub fn time(&self) -> Result<DeviceTime> {
        let mut full_secs = 0;
        let mut frac_secs = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_now(
                self.usrp.handle(),
                self.mboard,
                addr_of_mut!(full_secs),
                addr_of_mut!(frac_secs),
            )
        })?;
        Ok(DeviceTime::from_parts(full_secs as u64, frac_secs))
    }

    pub fn set_time(&self, time: DeviceTime) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_now(
                self.usrp.handle(),
                time.full_seconds() as i64,
                time.fractional_seconds(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    pub fn set_time_next_pps(&mut self, time: DeviceTime) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_next_pps(
                self.usrp.handle(),
                time.full_seconds() as i64,
                time.fractional_seconds(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    pub fn last_pps_time(&self) -> Result<DeviceTime> {
        let mut full_seconds: i64 = 0;
        let mut frac_seconds: f64 = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_last_pps(
                self.usrp.handle(),
                self.mboard,
                addr_of_mut!(full_seconds),
                addr_of_mut!(frac_seconds),
            )
        })?;
        Ok(DeviceTime::from_parts(full_seconds as u64, frac_seconds))
    }

    pub fn time_source(&self) -> Result<String> {
        let mut name = FfiString::<16>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_source(
                self.usrp.handle(),
                self.mboard,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    pub fn set_time_source(&self, source: &str) -> Result<()> {
        let source = CString::new(source).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_source(self.usrp.handle(), source.as_ptr(), self.mboard)
        })?;
        Ok(())
    }

    pub fn time_sources(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_sources(
                self.usrp.handle(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        vec.to_vec()
    }

    pub fn clock_source(&self) -> Result<String> {
        let mut name = FfiString::<16>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_clock_source(
                self.usrp.handle(),
                self.mboard,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    pub fn set_clock_source(&self, source: &str) -> Result<()> {
        let source = CString::new(source).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_clock_source(
                self.usrp.handle(),
                source.as_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

    pub fn clock_sources(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_clock_sources(
                self.usrp.handle(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        vec.to_vec()
    }

    pub fn set_time_source_out(&mut self, en: bool) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_source_out(self.usrp.handle(), en, self.mboard)
        })?;
        Ok(())
    }

    pub fn set_clock_source_out(&mut self, en: bool) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_clock_source_out(self.usrp.handle(), en, self.mboard)
        })?;
        Ok(())
    }

    pub fn master_clock_rate(&self) -> Result<f64> {
        let mut result = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_master_clock_rate(
                self.usrp.handle(),
                self.mboard,
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    pub fn set_master_clock_rate(&mut self, rate: f64) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_master_clock_rate(self.usrp.handle(), rate, self.mboard)
        })?;
        Ok(())
    }

    pub fn sensor_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_sensor_names(
                self.usrp.handle(),
                self.mboard,
                vec.as_mut_ptr(),
            )
        })?;
        vec.to_vec()
    }

    pub fn sensor_value(&self, name: &str) -> Result<SensorValue> {
        let name = CString::new(name).unwrap();
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_sensor_value_t>::new(
            uhd_usrp_sys::uhd_sensor_value_make,
            uhd_usrp_sys::uhd_sensor_value_free,
        )?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_mboard_sensor(
                self.usrp.handle(),
                name.as_ptr(),
                self.mboard,
                handle.as_mut_mut_ptr(),
            )
        })?;
        Ok(SensorValue::new(handle))
    }

    pub fn gpio_bank_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_gpio_banks(self.usrp.handle(), self.mboard, vec.as_mut_ptr())
        })?;
        vec.to_vec()
    }

    pub fn gpio_bank(&self, name: &str) -> GpioBank {
        GpioBank::new(self.usrp, self.mboard, name)
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
                self.usrp.handle(),
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
                self.usrp.handle(),
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
