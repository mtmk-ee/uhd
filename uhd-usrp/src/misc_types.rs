use std::{ffi::CStr, ptr::addr_of_mut};

use crate::{ffi::OwnedHandle, try_uhd, Result, UhdError};


/// A range object describes a set of discrete values of the form:
/// `y = start + step*n`, where `n` is an integer between `0` and `(stop - start)/step`.
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    /// The minimum value for this range.
    pub start: f64,
    /// The maximum value for this range.
    pub stop: f64,
    /// The step size for this range.
    pub step: f64,
}

pub struct MetaRange {
    inner: OwnedHandle<uhd_usrp_sys::uhd_meta_range_t>,
    ranges: Vec<Range>,
    start: f64,
    stop: f64,
    step: f64,
}

impl MetaRange {
    pub(crate) fn from_handle(handle: OwnedHandle<uhd_usrp_sys::uhd_meta_range_t>) -> Result<Self> {
        let (mut start, mut stop, mut step) = (0.0, 0.0, 0.0);
        let mut size = 0;
        let mut temp = uhd_usrp_sys::uhd_range_t {
            start: 0.0,
            stop: 0.0,
            step: 0.0,
        };
        let mut ranges: Vec<Range> = vec![];
        unsafe {
            try_uhd!(uhd_usrp_sys::uhd_meta_range_start(
                handle.as_mut_ptr(),
                addr_of_mut!(start)
            ))?;
            try_uhd!(uhd_usrp_sys::uhd_meta_range_stop(
                handle.as_mut_ptr(),
                addr_of_mut!(stop)
            ))?;
            try_uhd!(uhd_usrp_sys::uhd_meta_range_step(
                handle.as_mut_ptr(),
                addr_of_mut!(step)
            ))?;
            try_uhd!(uhd_usrp_sys::uhd_meta_range_size(
                handle.as_mut_ptr(),
                addr_of_mut!(size)
            ))?;

            for i in 0..size {
                try_uhd!(uhd_usrp_sys::uhd_meta_range_at(
                    handle.as_mut_ptr(),
                    i,
                    addr_of_mut!(temp)
                ))?;
                ranges.push(Range {
                    start: temp.start,
                    stop: temp.stop,
                    step: temp.stop,
                });
            }
        };
        Ok(Self {
            start,
            stop,
            step,
            ranges,
            inner: handle,
        })
    }

    /// Clip the target value so that it lies within one of the ranges.
    ///
    /// If `clip_step` is true, clip to steps as well.
    pub fn clip(&self, value: f64, clip_step: bool) -> Result<f64> {
        let mut result = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_meta_range_clip(
                self.inner.as_ptr().cast_mut(),
                value,
                clip_step,
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    pub fn ranges(&self) -> &[Range] {
        &self.ranges
    }

    /// Get the overall start (minimum) start value for this range.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Get the overall step size for this range.
    pub fn step(&self) -> f64 {
        self.step
    }

    /// Get the overall stop (maximum) value for this range.
    pub fn stop(&self) -> f64 {
        self.stop
    }
}

pub struct HardwareInfo {
    mboard_id: String,
    mboard_name: String,
    mboard_serial: String,
    dboard_id: String,
    dboard_subdev_name: String,
    dboard_subdev_spec: String,
    dboard_serial: String,
    dboard_antenna: String,
}

impl HardwareInfo {
    pub(crate) fn from_rx_raw(info: &uhd_usrp_sys::uhd_usrp_rx_info_t) -> Result<Self> {
        let fetch = |s| unsafe {
            CStr::from_ptr(s)
                .to_str()
                .or(Err(UhdError::Unknown))
                .map(ToString::to_string)
        };
        Ok(Self {
            mboard_id: fetch(info.mboard_id)?,
            mboard_name: fetch(info.mboard_name)?,
            mboard_serial: fetch(info.mboard_serial)?,
            dboard_id: fetch(info.rx_id)?,
            dboard_subdev_name: fetch(info.rx_subdev_name)?,
            dboard_subdev_spec: fetch(info.rx_subdev_spec)?,
            dboard_serial: fetch(info.rx_serial)?,
            dboard_antenna: fetch(info.rx_antenna)?,
        })
    }

    pub(crate) fn from_tx_raw(info: &uhd_usrp_sys::uhd_usrp_tx_info_t) -> Result<Self> {
        let fetch = |s| unsafe {
            CStr::from_ptr(s)
                .to_str()
                .or(Err(UhdError::Unknown))
                .map(ToString::to_string)
        };
        Ok(Self {
            mboard_id: fetch(info.mboard_id)?,
            mboard_name: fetch(info.mboard_name)?,
            mboard_serial: fetch(info.mboard_serial)?,
            dboard_id: fetch(info.tx_id)?,
            dboard_subdev_name: fetch(info.tx_subdev_name)?,
            dboard_subdev_spec: fetch(info.tx_subdev_spec)?,
            dboard_serial: fetch(info.tx_serial)?,
            dboard_antenna: fetch(info.tx_antenna)?,
        })
    }

    pub fn mboard_id(&self) -> &str {
        &self.mboard_id
    }

    pub fn mboard_name(&self) -> &str {
        &self.mboard_name
    }

    pub fn mboard_serial(&self) -> &str {
        &self.mboard_serial
    }

    pub fn dboard_antenna(&self) -> &str {
        &self.dboard_antenna
    }

    pub fn dboard_id(&self) -> &str {
        &self.dboard_id
    }

    pub fn dboard_serial(&self) -> &str {
        &self.dboard_serial
    }

    pub fn dboard_subdev_name(&self) -> &str {
        &self.dboard_subdev_name
    }

    pub fn dboard_subdev_spec(&self) -> &str {
        &self.dboard_subdev_spec
    }
}
