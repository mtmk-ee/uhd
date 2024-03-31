use std::{ffi::CString, ptr::addr_of_mut};

use crate::{
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    try_uhd, Result, SensorValue, TimeSpec, Usrp,
};

use super::subdev_spec::SubdevSpec;

pub struct Motherboard<'a> {
    usrp: &'a Usrp,
    mboard: usize,
}

impl<'a> Motherboard<'a> {
    pub(crate) fn new<'b>(usrp: &'a Usrp, mboard: usize) -> Self {
        Self { usrp, mboard }
    }

    pub fn clock_source(&self) -> Result<String> {
        let mut name = FfiString::<16>::new();
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

    pub fn gpio_bank(&self, name: &str) -> GpioBank {
        GpioBank::new(self.usrp, self.mboard, name)
    }

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

    pub fn name(&self) -> Result<String> {
        let mut result = FfiString::<16>::new();
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

    pub fn rx_subdev_spec(&self) -> Result<SubdevSpec> {
        let spec = SubdevSpec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                spec.as_handle().as_mut_ptr(),
            )
        })?;
        Ok(spec)
    }

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

    pub fn set_rx_subdev_str(&mut self, subdev: &str) -> Result<()> {
        let sudev = SubdevSpec::from_str(subdev);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                sudev.as_handle().as_mut_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

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

    pub fn set_tx_subdev_str(&mut self, subdev: &str) -> Result<()> {
        let sudev = SubdevSpec::from_str(subdev);
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_tx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                sudev.as_handle().as_mut_ptr(),
                self.mboard,
            )
        })?;
        Ok(())
    }

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

    pub fn time_source(&self) -> Result<String> {
        let mut name = FfiString::<16>::new();
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

    pub fn tx_subdev_spec(&self) -> Result<SubdevSpec> {
        let spec = SubdevSpec::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_subdev_spec(
                self.usrp.handle().as_mut_ptr(),
                self.mboard,
                spec.as_handle().as_mut_ptr(),
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
        let mut value = FfiString::<32>::new();
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
            ).unwrap(),
        }
    }

    pub(crate) fn from_handle(handle: OwnedHandle<uhd_usrp_sys::uhd_dboard_eeprom_t>) -> Self {
        Self { handle }
    }

    pub fn id(&self) -> String {
        let mut id = FfiString::<16>::new();
        unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_get_id(
                self.handle.as_mut_ptr(),
                id.as_mut_ptr(),
                id.max_chars(),
            )
        };
        id.into_string().unwrap()
    }

    pub fn set_id(&self, id: &str) {
        let id = CString::new(id).unwrap();
        unsafe { uhd_usrp_sys::uhd_dboard_eeprom_set_id(self.handle.as_mut_ptr(), id.as_ptr()) };
    }

    pub fn serial_number(&self) -> String {
        let mut id = FfiString::<16>::new();
        unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_get_serial(
                self.handle.as_mut_ptr(),
                id.as_mut_ptr(),
                id.max_chars(),
            )
        };
        id.into_string().unwrap()
    }

    pub fn set_serial_number(&self, serial: &str) {
        let serial = CString::new(serial).unwrap();
        unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_set_serial(self.handle.as_mut_ptr(), serial.as_ptr());
        }
    }

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
        unsafe {
            uhd_usrp_sys::uhd_dboard_eeprom_set_revision(self.handle.as_mut_ptr(), value)
        };
    }
}
