use std::{ffi::CStr, mem::MaybeUninit};

use crate::{try_uhd, Result, UhdError, Usrp};

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
    pub(crate) fn new_rx(usrp: &Usrp, channel: usize) -> Result<Self> {
        let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_rx_info_t> = MaybeUninit::uninit();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_info(
                usrp.handle().as_mut_ptr(),
                channel,
                info.as_mut_ptr(),
            )
        })?;
        Self::from_rx_raw(unsafe { &info.assume_init() })
    }

    pub(crate) fn new_tx(usrp: &Usrp, channel: usize) -> Result<Self> {
        let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_tx_info_t> = MaybeUninit::uninit();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_info(
                usrp.handle().as_mut_ptr(),
                channel,
                info.as_mut_ptr(),
            )
        })?;
        Self::from_tx_raw(unsafe { &info.assume_init() })
    }

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
