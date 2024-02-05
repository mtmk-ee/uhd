use std::{ffi::CStr, ptr::addr_of_mut};

use crate::{ffi::OwnedHandle, try_uhd, Result, UhdError};

pub struct Range {
    pub start: f64,
    pub stop: f64,
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

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn step(&self) -> f64 {
        self.step
    }

    pub fn stop(&self) -> f64 {
        self.stop
    }
}

pub struct HardwareInfo {
    mboard_id: &'static str,
    mboard_name: &'static str,
    mboard_serial: &'static str,
    dboard_id: &'static str,
    dboard_subdev_name: &'static str,
    dboard_subdev_spec: &'static str,
    dboard_serial: &'static str,
    dboard_antenna: &'static str,
}

impl HardwareInfo {
    pub(crate) fn from_rx_raw(info: &uhd_usrp_sys::uhd_usrp_rx_info_t) -> Result<Self> {
        let fetch = |s| unsafe { CStr::from_ptr(s).to_str().or(Err(UhdError::Unknown)) };
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
        let fetch = |s| unsafe { CStr::from_ptr(s).to_str().or(Err(UhdError::Unknown)) };
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
        self.mboard_id
    }

    pub fn mboard_name(&self) -> &str {
        self.mboard_name
    }

    pub fn mboard_serial(&self) -> &str {
        self.mboard_serial
    }

    pub fn rx_antenna(&self) -> &str {
        self.dboard_antenna
    }

    pub fn rx_id(&self) -> &str {
        self.dboard_id
    }

    pub fn rx_serial(&self) -> &str {
        self.dboard_serial
    }

    pub fn rx_subdev_name(&self) -> &str {
        self.dboard_subdev_name
    }

    pub fn rx_subdev_spec(&self) -> &str {
        self.dboard_subdev_spec
    }
}



