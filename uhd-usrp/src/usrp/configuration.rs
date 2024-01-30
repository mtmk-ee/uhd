use std::{
    ffi::{CStr, CString},
    ptr::addr_of_mut,
};

use crate::{error::try_uhd, util::gen_getter, Result, UhdError};

use super::{
    tune::{TuneRequest, TuneResult},
    Usrp,
};

pub struct RxChannelConfig<'usrp> {
    usrp: &'usrp Usrp,
    channel: usize,
}

impl<'usrp> RxChannelConfig<'usrp> {
    pub(crate) fn new(usrp: &'usrp Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }

    pub fn antenna(&self) -> Result<String> {
        const NAME_LEN: usize = 16;
        let mut name = [0u8; NAME_LEN];
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_antenna(
                self.usrp.handle(),
                self.channel,
                name.as_mut_ptr().cast(),
                NAME_LEN - 1,
            )
        })?;
        Ok(CStr::from_bytes_until_nul(&name)
            .or(Err(UhdError::Unknown))?
            .to_string_lossy()
            .into_owned())
    }

    pub fn subdev_name(&self) -> Result<String> {
        const NAME_LEN: usize = 64;
        let mut name = [0u8; NAME_LEN];
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_subdev_name(
                self.usrp.handle(),
                self.channel,
                name.as_mut_ptr().cast(),
                NAME_LEN - 1,
            )
        })?;
        Ok(CStr::from_bytes_until_nul(&name)
            .or(Err(UhdError::Unknown))?
            .to_string_lossy()
            .into_owned())
    }

    pub fn sample_rate(&self) -> Result<f64> {
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_rx_rate => (self.usrp.handle(), self.channel, _))
        }
    }

    pub fn bandwidth(&self) -> Result<f64> {
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_rx_bandwidth => (self.usrp.handle(), self.channel, _))
        }
    }

    pub fn center_freq(&self) -> Result<f64> {
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_rx_freq => (self.usrp.handle(), self.channel, _))
        }
    }

    pub fn lo_freq(&self) -> Result<f64> {
        let name = CString::new("").unwrap();
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_rx_lo_freq  => (self.usrp.handle(), name.as_ptr(), self.channel, _))
        }
    }

    pub fn gain(&self) -> Result<f64> {
        let name = CString::new("").unwrap();
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_rx_gain => (
                self.usrp.handle(),
                self.channel,
                name.as_ptr(),
                _
            ))
        }
    }

    pub fn normalized_gain(&self) -> Result<f64> {
        unsafe {
            gen_getter!(uhd_usrp_sys::uhd_usrp_get_normalized_rx_gain => (
                self.usrp.handle(),
                self.channel,
                _
            ))
        }
    }

    pub fn tune(&self, req: &TuneRequest) -> Result<TuneResult> {
        let req = req.inner();
        let mut result = TuneResult::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_freq(
                self.usrp.handle(),
                req as *const _ as *mut _,
                self.channel,
                result.inner_mut(),
            )
        })?;
        Ok(result)
    }
}

pub struct SetRxChannelConfig<'usrp> {
    usrp: &'usrp Usrp,
    channel: usize,
}

impl<'usrp> SetRxChannelConfig<'usrp> {
    pub(crate) fn new(usrp: &'usrp Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }

    pub fn set_sample_rate(self, rate: f64) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_rate(self.usrp.handle(), rate, self.channel)
        })?;
        Ok(self)
    }

    pub fn set_gain(self, gain: f64) -> Result<Self> {
        let name = CString::new("").unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_gain(
                self.usrp.handle(),
                gain,
                self.channel,
                name.as_ptr(),
            )
        })?;
        Ok(self)
    }

    pub fn set_normalized_gain(self, gain: f64) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_normalized_rx_gain(self.usrp.handle(), gain, self.channel)
        })?;
        Ok(self)
    }

    pub fn set_lo_freq(self, freq: f64) -> Result<Self> {
        let name = CString::new("").unwrap();
        let mut result = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_lo_freq(
                self.usrp.handle(),
                freq,
                name.as_ptr(),
                self.channel,
                addr_of_mut!(result),
            )
        })?;
        Ok(self)
    }

    pub fn set_antenna(self, name: &str) -> Result<Self> {
        let name = CString::new(name).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_antenna(self.usrp.handle(), name.as_ptr(), self.channel)
        })?;
        Ok(self)
    }

    pub fn set_dc_offset_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_dc_offset_enabled(self.usrp.handle(), en, self.channel)
        })?;
        Ok(self)
    }

    pub fn set_iq_balance_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_iq_balance_enabled(self.usrp.handle(), en, self.channel)
        })?;
        Ok(self)
    }

    pub fn set_agc_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_agc(self.usrp.handle(), en, self.channel)
        })?;
        Ok(self)
    }
}
