use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::addr_of_mut,
};

use crate::{
    error::try_uhd,
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    misc_types::MetaRange,
    usrp::{TuneRequest, TuneResult, Usrp},
    HardwareInfo, Result, SensorValue, UhdError,
};

pub(crate) const TX_DIR: usize = 0;
pub(crate) const RX_DIR: usize = 1;

pub struct ChannelConfiguration<'usrp, const D: usize> {
    // D parameter is a hack until const enum generics are stabilized
    usrp: &'usrp Usrp,
    channel: usize,
}

impl<'usrp, const D: usize> ChannelConfiguration<'usrp, D> {
    pub fn antenna(&self) -> Result<String> {
        let mut name = FfiString::<16>::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_antenna,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_antenna,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                self.channel,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    pub fn antennas(&self) -> Result<Vec<String>> {
        let mut names = FfiStringVec::new()?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_antennas,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_antennas,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, names.as_mut_ptr(),) })?;
        names.to_vec()
    }

    pub fn bandwidth(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, result.as_mut_ptr(),) })
            .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn bandwidth_ranges(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, handle.as_mut_ptr(),) })?;
        MetaRange::from_handle(handle)
    }

    pub fn center_freq(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, result.as_mut_ptr(),) })
            .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn center_freq_ranges(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_freq_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_freq_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, handle.as_mut_ptr(),) })?;
        MetaRange::from_handle(handle)
    }

    pub fn frontend_freq_range(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_fe_rx_freq_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_fe_tx_freq_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, handle.as_mut_ptr(),) })?;
        MetaRange::from_handle(handle)
    }

    pub fn gain(&self, name: Option<&str>) -> Result<f64> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                self.channel,
                name.as_ptr(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn gain_ranges(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let mut name = FfiString::<64>::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_gain_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_gain_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_mut_ptr().cast(),
                self.channel,
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    pub fn hardware_info(&self) -> Result<HardwareInfo> {
        match D {
            RX_DIR => {
                let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_rx_info_t> = MaybeUninit::uninit();
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_get_rx_info(
                        self.usrp.handle(),
                        self.channel,
                        info.as_mut_ptr(),
                    )
                })?;
                HardwareInfo::from_rx_raw(unsafe { &info.assume_init() })
            }
            TX_DIR => {
                let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_tx_info_t> = MaybeUninit::uninit();
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_get_tx_info(
                        self.usrp.handle(),
                        self.channel,
                        info.as_mut_ptr(),
                    )
                })?;
                HardwareInfo::from_tx_raw(unsafe { &info.assume_init() })
            }
            _ => unreachable!(),
        }
    }

    pub fn lo_export_enabled(&self, name: Option<&str>) -> Result<bool> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = false;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_export_enabled,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_export_enabled,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_ptr(),
                self.channel,
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    pub fn lo_freq(&self, name: Option<&str>) -> Result<f64> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_ptr(),
                self.channel,
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn lo_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, vec.as_mut_ptr()) })?;
        vec.to_vec()
    }

    pub fn lo_source(&self, name: Option<&str>) -> Result<String> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut buf = FfiString::<32>::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_source,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_source,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_ptr(),
                self.channel,
                buf.as_mut_ptr(),
                buf.max_chars(),
            )
        })?;
        buf.into_string()
    }

    pub fn lo_sources(&self, name: Option<&str>) -> Result<Vec<String>> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut vec = FfiStringVec::new()?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_sources,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_sources,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_ptr(),
                self.channel,
                vec.as_mut_ptr(),
            )
        })?;
        vec.to_vec()
    }

    pub fn normalized_gain(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_normalized_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_normalized_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, result.as_mut_ptr(),) })
            .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn sample_rate(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_rate,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_rate,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, result.as_mut_ptr(),) })
            .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    pub fn sample_rates(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_rates,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_rates,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, handle.as_mut_ptr(),) })?;
        MetaRange::from_handle(handle)
    }

    pub fn sensor_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new()?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_sensor_names,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_sensor_names,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), self.channel, vec.as_mut_ptr()) })?;
        vec.to_vec()
    }

    pub fn sensor_value(&self, name: &str) -> Result<SensorValue> {
        let name = CString::new(name).unwrap();
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_sensor_value_t>::new(
            uhd_usrp_sys::uhd_sensor_value_make,
            uhd_usrp_sys::uhd_sensor_value_free,
        )?;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_sensor,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_sensor,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                name.as_ptr(),
                self.channel,
                handle.as_mut_mut_ptr(),
            )
        })?;
        Ok(SensorValue::new(handle))
    }

    pub fn subdev_name(&self) -> Result<String> {
        let mut name = FfiString::<64>::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_subdev_name,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_subdev_name,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                self.channel,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    pub fn subdev_spec(&self) -> Result<Vec<SubDevSpec>> {
        let mut spec = MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_subdev_spec,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_subdev_spec,
            _ => unimplemented!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), 0, spec.as_mut_ptr()) })?;
        let mut spec = unsafe { spec.assume_init() };
        let mut size = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_subdev_spec_size(addr_of_mut!(spec), addr_of_mut!(size))
        })?;
        let mut result = vec![];
        for i in 0..size {
            let mut pair = MaybeUninit::uninit();
            try_uhd!(unsafe {
                uhd_usrp_sys::uhd_subdev_spec_at(addr_of_mut!(spec), i, pair.as_mut_ptr())
            })?;
            let pair = unsafe { pair.assume_init() };
            unsafe {
                result.push(SubDevSpec {
                    sub_device: CStr::from_ptr(pair.sd_name)
                        .to_str()
                        .or(Err(UhdError::Unknown))?
                        .to_string(),
                    daughter_board: CStr::from_ptr(pair.db_name)
                        .to_str()
                        .or(Err(UhdError::Unknown))?
                        .to_string(),
                });
            }
        }
        Ok(result)
    }
}

impl<'a> ChannelConfiguration<'a, TX_DIR> {
    pub(crate) fn new(usrp: &'a Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }
}

impl<'a> ChannelConfiguration<'a, RX_DIR> {
    pub(crate) fn new(usrp: &'a Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }
}

pub struct ChannelConfigurationBuilder<'usrp, const D: usize> {
    usrp: &'usrp Usrp,
    channel: usize,
}

impl<'usrp, const D: usize> ChannelConfigurationBuilder<'usrp, D> {
    pub fn set_antenna(self, name: &str) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_antenna,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_antenna,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), name.as_ptr(), self.channel) })?;
        Ok(self)
    }

    pub fn set_bandwidth(self, bw: f64) -> Result<Self> {
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_bandwidth,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_bandwidth,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), bw, self.channel) })?;
        Ok(self)
    }

    pub fn set_center_freq(self, freq: f64) -> Result<Self> {
        self.tune(
            &TuneRequest::new()
                .center_freq(freq)
                .rf_freq_unset()
                .dsp_freq_unset(),
        )
    }

    pub fn set_gain(self, name: Option<&str>, gain: f64) -> Result<Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), gain, self.channel, name.as_ptr(),) })?;
        Ok(self)
    }

    pub fn set_lo_freq(self, name: Option<&str>, freq: f64) -> Result<Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = 0.0;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_lo_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_lo_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                freq,
                name.as_ptr(),
                self.channel,
                addr_of_mut!(result),
            )
        })?;
        Ok(self)
    }

    pub fn set_normalized_gain(self, gain: f64) -> Result<Self> {
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_normalized_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_normalized_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), gain, self.channel) })?;
        Ok(self)
    }

    pub fn set_sample_rate(self, rate: f64) -> Result<Self> {
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_rate,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_rate,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle(), rate, self.channel) })?;
        Ok(self)
    }

    pub fn tune(self, req: &TuneRequest) -> Result<Self> {
        let req = req.inner();
        let mut result = TuneResult::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle(),
                req as *const _ as *mut _,
                self.channel,
                result.inner_mut(),
            )
        })?;
        Ok(self)
    }
}

impl<'a> ChannelConfigurationBuilder<'a, TX_DIR> {
    pub(crate) fn new(usrp: &'a Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }
}

impl<'a> ChannelConfigurationBuilder<'a, RX_DIR> {
    pub(crate) fn new(usrp: &'a Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }

    pub fn set_agc_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_agc(self.usrp.handle(), en, self.channel)
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
}

pub struct SubDevSpec {
    pub sub_device: String,
    pub daughter_board: String,
}
