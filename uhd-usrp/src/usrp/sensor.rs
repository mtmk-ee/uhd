use std::ptr::addr_of_mut;

use crate::{
    ffi::{FfiString, OwnedHandle},
    try_uhd, Result,
};

pub struct SensorValue {
    handle: OwnedHandle<uhd_usrp_sys::uhd_sensor_value_t>,
}

impl SensorValue {
    pub(crate) fn new(handle: OwnedHandle<uhd_usrp_sys::uhd_sensor_value_t>) -> Self {
        Self { handle }
    }

    pub fn name(&self) -> Result<String> {
        let mut s = FfiString::<32>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_name(
                self.handle.as_mut_ptr(),
                s.as_mut_ptr(),
                s.max_chars(),
            )
        })?;
        s.into_string()
    }

    pub fn to_bool(&self) -> Result<bool> {
        let mut value = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_to_bool(self.handle.as_mut_ptr(), addr_of_mut!(value))
        })?;
        Ok(value)
    }

    pub fn to_f64(&self) -> Result<f64> {
        let mut value = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_to_realnum(self.handle.as_mut_ptr(), addr_of_mut!(value))
        })?;
        Ok(value)
    }

    pub fn to_i32(&self) -> Result<i32> {
        let mut value = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_to_int(self.handle.as_mut_ptr(), addr_of_mut!(value))
        })?;
        Ok(value)
    }

    pub fn to_pp_string(&self) -> Result<String> {
        let mut value = FfiString::<64>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_to_pp_string(
                self.handle.as_mut_ptr(),
                value.as_mut_ptr(),
                value.max_chars(),
            )
        })?;
        value.into_string()
    }

    pub fn to_string(&self) -> Result<String> {
        let mut value = FfiString::<64>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_value(
                self.handle.as_mut_ptr(),
                value.as_mut_ptr(),
                value.max_chars(),
            )
        })?;
        value.into_string()
    }

    pub fn unit(&self) -> Result<String> {
        let mut s = FfiString::<16>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_sensor_value_unit(
                self.handle.as_mut_ptr(),
                s.as_mut_ptr(),
                s.max_chars(),
            )
        })?;
        s.into_string()
    }
}

