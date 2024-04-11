use std::{mem::MaybeUninit, ptr::addr_of_mut};

use crate::{
    ffi::{FfiString, OwnedHandle},
    try_uhd, Result, UhdError,
};

/// A sensor value stores a sensor reading as a string with unit and data type.
pub struct SensorValue {
    kind: SensorValueValue,
    unit: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SensorValueValue {
    Boolean(bool),
    Real(f64),
    Integer(i32),
    String(String),
}

impl SensorValue {
    /// Try to create a new sensor value from a C handle.
    pub(crate) fn from_handle(
        handle: &OwnedHandle<uhd_usrp_sys::uhd_sensor_value_t>,
    ) -> Result<Self> {
        let mut buf = FfiString::with_capacity(32);
        unsafe {
            uhd_usrp_sys::uhd_sensor_value_unit(
                handle.as_mut_ptr(),
                buf.as_mut_ptr(),
                buf.max_chars(),
            );
        }
        let unit = buf.to_string()?;
        unsafe {
            uhd_usrp_sys::uhd_sensor_value_name(
                handle.as_mut_ptr(),
                buf.as_mut_ptr(),
                buf.max_chars(),
            );
        }
        let name = buf.to_string()?;
        Ok(Self {
            kind: SensorValueValue::from_handle(handle)?,
            unit,
            name,
        })
    }

    /// The name of the sensor this value is associated with.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The unit of the sensor this value is associated with.
    ///
    /// For boolean types, the unit is a description of what the
    /// true and false values represent.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// The actual numeric or boolean value.
    pub fn value(&self) -> &SensorValueValue {
        &self.kind
    }

    /// Returns `Some` if the value is a boolean, `None` otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match self.kind {
            SensorValueValue::Boolean(b) => Some(b),
            _ => None,
        }
    }

    /// Returns `Some` if the value is a real number or integer, `None` otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match self.kind {
            SensorValueValue::Real(f) => Some(f),
            SensorValueValue::Integer(i) => Some(i as f64),
            _ => None,
        }
    }

    /// Returns `Some` if the value is an integer, `None` otherwise.
    pub fn as_i32(&self) -> Option<i32> {
        match self.kind {
            SensorValueValue::Integer(i) => Some(i),
            _ => None,
        }
    }
}

impl std::fmt::Display for SensorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value() {
            SensorValueValue::Boolean(_) => write!(f, "{}: {}", self.name(), self.unit()),
            SensorValueValue::Real(v) => write!(f, "{}: {} {}", self.name(), v, self.unit()),
            SensorValueValue::Integer(v) => write!(f, "{}: {} {}", self.name(), v, self.unit()),
            SensorValueValue::String(v) => write!(f, "{}: {} {}", self.name(), v, self.unit()),
        }
    }
}

impl SensorValueValue {
    pub(crate) fn from_handle(
        handle: &OwnedHandle<uhd_usrp_sys::uhd_sensor_value_t>,
    ) -> Result<Self> {
        let mut dtype: MaybeUninit<uhd_usrp_sys::uhd_sensor_value_data_type_t::Type> =
            MaybeUninit::uninit();
        unsafe {
            uhd_usrp_sys::uhd_sensor_value_data_type(handle.as_mut_ptr(), dtype.as_mut_ptr());
        }
        let dtype = unsafe { dtype.assume_init() };

        match dtype {
            uhd_usrp_sys::uhd_sensor_value_data_type_t::UHD_SENSOR_VALUE_BOOLEAN => {
                let mut val = false;
                // Doesn't throw
                unsafe {
                    uhd_usrp_sys::uhd_sensor_value_to_bool(handle.as_mut_ptr(), addr_of_mut!(val))
                };
                Ok(Self::Boolean(val))
            }
            uhd_usrp_sys::uhd_sensor_value_data_type_t::UHD_SENSOR_VALUE_REALNUM => {
                let mut val = 0.0;
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_sensor_value_to_realnum(
                        handle.as_mut_ptr(),
                        addr_of_mut!(val),
                    )
                })?;
                Ok(Self::Real(val))
            }
            uhd_usrp_sys::uhd_sensor_value_data_type_t::UHD_SENSOR_VALUE_INTEGER => {
                let mut val = 0;
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_sensor_value_to_int(handle.as_mut_ptr(), addr_of_mut!(val))
                })?;
                Ok(Self::Integer(val))
            }
            uhd_usrp_sys::uhd_sensor_value_data_type_t::UHD_SENSOR_VALUE_STRING => {
                let mut val = FfiString::with_capacity(32);
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_sensor_value_value(handle.as_mut_ptr(), val.as_mut_ptr(), val.max_chars())
                })?;
                Ok(Self::String(val.to_string()?))
            }
            _ => Err(UhdError::NotImplemented),
        }
    }
}
