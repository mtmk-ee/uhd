use std::{ffi::CString, fmt::Display, ptr::addr_of_mut};

use crate::{
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    try_uhd,
    types::{MetaRange, SensorValue, TuneRequest, TuneResult},
    HardwareInfo, Result, UhdError, Usrp,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Rx(usize),
    Tx(usize),
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Rx(i) => write!(f, "RX {i}"),
            Channel::Tx(i) => write!(f, "TX {i}"),
        }
    }
}

pub struct ChannelConfig<'u> {
    usrp: &'u Usrp,
    channel: Channel,
}

impl Channel {
    pub fn index(&self) -> usize {
        match self {
            Channel::Rx(i) | Channel::Tx(i) => *i,
        }
    }

    pub fn is_rx(&self) -> bool {
        matches!(self, Channel::Rx(_))
    }

    pub fn is_tx(&self) -> bool {
        matches!(self, Channel::Tx(_))
    }
}

impl<'u> ChannelConfig<'u> {
    pub fn new(usrp: &'u Usrp, channel: Channel) -> Self {
        Self { usrp, channel }
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    /// Get the name of the frontend.
    pub fn subdev_name(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(64);
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_subdev_name,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_subdev_name,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.to_string()
    }

    /// Fetch names, serial numbers, etc. of the channel's hardware.
    pub fn hardware_info(&self) -> Result<HardwareInfo> {
        match self.channel {
            Channel::Rx(i) => HardwareInfo::new_rx(self.usrp, i),
            Channel::Tx(i) => HardwareInfo::new_tx(self.usrp, i),
        }
    }

    pub fn print_common(&self) -> Result<()> {
        println!("Antenna: {}", self.antenna()?);
        println!("Freq: {:.2} MHz", self.center_freq()? / 1e6);
        println!("Bandwidth: {:.2} MHz", self.bandwidth()? / 1e6);
        println!("Gain: {} dB", self.gain(None)?);
        println!("Rate: {} ksps", self.sample_rate()? / 1e3);
        Ok(())
    }
}

// --------------------------------------------------------------------------
/// Antenna configuration
impl<'u> ChannelConfig<'u> {
    pub fn antenna(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(16);
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_antenna,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_antenna,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.to_string()
    }

    /// Get a list of antennas associated with the channel.
    pub fn antennas(&self) -> Result<Vec<String>> {
        let mut names = FfiStringVec::new();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_antennas,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_antennas,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                names.as_mut_ptr(),
            )
        })?;
        Ok(names.to_vec())
    }

    /// Select the antenna to use on the frontend.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid antenna name is provided.
    pub fn set_antenna(&self, name: &str) -> Result<&Self> {
        let name = CString::new(name).unwrap();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_antenna,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_antenna,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
            )
        })?;
        Ok(self)
    }
}

// --------------------------------------------------------------------------
/// Gain configuration
impl<'u> ChannelConfig<'u> {
    /// Enable or disable the RX AGC module.
    ///
    /// Once this module is enabled manual gain settings will be ignored.
    /// The AGC will start in a default configuration which should be good
    /// for most use cases.
    ///
    /// # Errors
    ///
    /// Returns an error if the device does not implement an AGC.
    ///
    /// Only some devices implement an AGC, including all USRPs from the B200 series,
    /// the E310, and the E320.
    pub fn set_agc_enabled(&self, en: bool) -> Result<&Self> {
        match self.channel {
            Channel::Rx(_) => {
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_set_rx_agc(
                        self.usrp.handle().as_mut_ptr(),
                        en,
                        self.channel.index(),
                    )
                })?;
                Ok(self)
            }
            Channel::Tx(_) => Err(UhdError::NotImplemented),
        }
    }

    /// Set the gain for the channel and given name.
    pub fn gain(&self, name: Option<&str>) -> Result<f64> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_gain,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_gain,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                name.as_ptr(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Get the RX gain range for the specified gain element.
    ///
    /// If `None` is provided, the overall gain range is returned.
    pub fn gain_ranges(&self, name: Option<&str>) -> Result<MetaRange> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_gain_range,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_gain_range,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Return the normalized gain value.
    ///
    /// This value is linearly mapped to the range [0, 1] for all devices.
    pub fn normalized_gain(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_normalized_rx_gain,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_normalized_tx_gain,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Set the normalized RX gain value.
    ///
    /// The normalized gain is a value in [0, 1], where 0 is the smallest gain value available,
    /// and 1 is the largest, independent of the device. In between, gains are linearly interpolated.
    ///
    /// Check the individual device manual for notes on the gain range.
    ///
    /// Note that it is not possible to specify a gain name for this function, it will always set the overall gain.
    ///
    /// # Panics
    ///
    /// Panics if the given gain is outside [0, 1].
    pub fn set_normalized_gain(&self, gain: f64) -> Result<&Self> {
        if gain < 0.0 || gain > 1.0 {
            panic!("gain must be 0 to 1");
        }
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_normalized_rx_gain,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_normalized_tx_gain,
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), gain, self.channel.index()) })?;
        Ok(self)
    }

    /// Set the RX gain value in dB for the specified gain element.
    ///
    /// If the requested gain value is outside the valid range,
    /// it will be coerced to a valid gain value.
    ///
    /// The name of the gain element to set can be provided.
    /// If `None`, it is distributed across all gain elements.
    pub fn set_gain(&self, name: Option<&str>, gain: f64) -> Result<&Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_gain,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_gain,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                gain,
                self.channel.index(),
                name.as_ptr(),
            )
        })?;
        Ok(self)
    }
}

// --------------------------------------------------------------------------
/// Bandwidth configuration
impl<'u> ChannelConfig<'u> {
    /// Get the bandwidth for the channel's frontend.
    pub fn bandwidth(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Get all possible bandwidth ranges for the channel's frontend.
    pub fn bandwidth_ranges(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth_range,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth_range,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Set the RX frontend's bandwidth in Hz.
    ///
    /// If a bandwidth is provided that is outside the valid range,
    /// it is coerced to the nearest valid value.
    pub fn set_bandwidth(&self, bw: f64) -> Result<&Self> {
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_bandwidth,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_bandwidth,
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), bw, self.channel.index()) })?;
        Ok(self)
    }
}

// --------------------------------------------------------------------------
/// Tuning configuration
impl<'u> ChannelConfig<'u> {
    /// Get the channel's center frequency.
    pub fn center_freq(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_freq,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_freq,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Get all possible center frequency ranges of the channel.
    ///
    /// This range includes the overall tunable range of the RX or TX chain,
    /// including frontend chain and digital down conversion chain. This tunable
    /// limit does not include the baseband bandwidth;
    /// users should assume that the actual range is ±sample_rate/2.
    pub fn center_freq_ranges(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_freq_range,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_freq_range,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Get all possible RF frequency ranges for the channel's RF frontend.
    pub fn frontend_freq_range(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_fe_rx_freq_range,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_fe_tx_freq_range,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Set the RX center frequency in Hz.
    ///
    /// If the requested frequency is outside of the valid frequency range,
    /// it will be coerced to the nearest valid frequency.
    pub fn set_center_freq(&self, freq: f64) -> Result<&Self> {
        self.tune(&TuneRequest::new(freq).rf_freq_auto().dsp_freq_auto())
    }

    /// Set the tuning parameters for the channel.
    ///
    /// This function allows setting more advanced parameters.
    pub fn tune(&self, req: &TuneRequest) -> Result<&Self> {
        let req = req.inner();
        let mut result = TuneResult::default();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_freq,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_freq,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                req as *const _ as *mut _,
                self.channel.index(),
                result.inner_mut(),
            )
        })?;
        Ok(self)
    }
}

// --------------------------------------------------------------------------
/// LO configuration
impl<'u> ChannelConfig<'u> {
    /// Returns true if the currently selected LO is being exported.
    pub fn lo_export_enabled(&self, name: Option<&str>) -> Result<bool> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = false;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_export_enabled,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_lo_export_enabled,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                addr_of_mut!(result),
            )
        })?;
        Ok(result)
    }

    /// Get the current RX LO frequency (Advanced).
    ///
    /// If the channel does not have independently configurable LOs,
    /// the current RF frequency will be returned.
    pub fn lo_freq(&self, name: Option<&str>) -> Result<f64> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_freq,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_lo_freq,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Get a list of possible LO stage names
    ///
    /// Example: On the TwinRX, this will return "LO1", "LO2".
    /// These names can are used in other LO-related API calls, so this function can be used for automatically enumerating LO stages.
    /// An empty return value doesn't mean there are no LOs, it means that this radio does not have an LO API implemented,
    /// and typically means the LOs have no direct way of being controlled other than setting the frequency.
    pub fn lo_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Get the currently selected LO source.
    ///
    /// Channels without controllable LO sources will always return "internal".
    pub fn lo_source(&self, name: Option<&str>) -> Result<String> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut buf = FfiString::with_capacity(32);
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_source,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_lo_source,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                buf.as_mut_ptr(),
                buf.max_chars(),
            )
        })?;
        buf.to_string()
    }

    /// Get a list of possible LO sources.
    ///
    /// Channels which do not have controllable LO sources will return "internal".
    /// Typical values are "internal" and "external", although the TwinRX, for example, has more options, such as "companion".
    /// These options are device-specific, so consult the individual device manual pages for details.
    pub fn lo_sources(&self, name: Option<&str>) -> Result<Vec<String>> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut vec = FfiStringVec::new();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_lo_sources,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_lo_sources,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Set the RX LO frequency (Advanced).
    ///
    /// The actual behaviour is device-specific. However, as a rule of thumb,
    /// this will coerce the underlying driver into some state. Typical situations include:
    ///
    /// - LOs are internal, and this function is called to pin an LO to a certain value.
    ///   This can force the driver to pick different IFs for different stages, and there
    ///   may be situations where this behaviour can be used to reduce spurs in specific bands.
    /// - LOs are external. In this case, this function is used to notify UHD what the actual
    ///   value of an externally provided LO is. The only time when calling this function is
    ///   necessary is when the LO source is set to external, but the external LO can't be
    ///   tuned to the exact value required by UHD to achieve a certain center frequency.
    ///   In this case, calling this function will let UHD know that the LO is not the
    ///   expected value, and it's possible that UHD will find other ways to compensate for
    ///   the LO offset.
    ///
    /// # Errors
    ///
    /// Returns an error if the LO name is not valid.
    pub fn set_lo_freq(&self, name: Option<&str>, freq: f64) -> Result<&Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = 0.0;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_lo_freq,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_lo_freq,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                freq,
                name.as_ptr(),
                self.channel.index(),
                addr_of_mut!(result),
            )
        })?;
        Ok(self)
    }

    /// Set whether the LO used by the device is exported
    ///
    /// For USRPs that support exportable LOs, this function configures
    /// if the LO used by the channel is exported or not.
    ///
    /// # Errors
    ///
    /// Returns an error if LO exporting is not available or if the
    /// given name is invalid.
    pub fn set_lo_export_enabled(&self, name: Option<&str>, en: bool) -> Result<&Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_lo_export_enabled(
                self.usrp.handle().as_mut_ptr(),
                en,
                name.as_ptr(),
                self.channel.index(),
            )
        })?;
        Ok(self)
    }
}

// --------------------------------------------------------------------------
/// Sample rate configuration
impl<'u> ChannelConfig<'u> {
    /// Return the sample rate in samples per second.
    pub fn sample_rate(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_rate,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_rate,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Get a range of possible sample rates.
    pub fn sample_rates(&self) -> Result<MetaRange> {
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_meta_range_t>::new(
            uhd_usrp_sys::uhd_meta_range_make,
            uhd_usrp_sys::uhd_meta_range_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_rates,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_rates,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Set the RX sample rate in samples per second.
    ///
    /// This function will coerce the requested rate to a rate that the
    /// device can handle. A warning may be logged during coercion.
    ///
    /// # Panics
    ///
    /// Panics if the given rate is non-positive.
    pub fn set_sample_rate(&self, rate: f64) -> Result<&Self> {
        if rate <= 0.0 {
            panic!("sample rate must be positive");
        }
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_set_rx_rate,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_set_tx_rate,
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), rate, self.channel.index()) })?;
        Ok(&self)
    }
}

// --------------------------------------------------------------------------
/// Sensors
impl<'u> ChannelConfig<'u> {
    /// Get a list of possible frontend sensor names.
    pub fn sensor_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_rx_sensor_names,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_tx_sensor_names,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel.index(),
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Get a frontend sensor value.
    ///
    /// # Errors
    ///
    /// Returns an error if the sensor name is invalid.
    ///
    /// # Panics
    ///
    /// Panics if the sensor name cannot be represented as a valid C string.
    pub fn sensor_value(&self, name: &str) -> Result<SensorValue> {
        let name = CString::new(name).expect("invalid characters in sensor name");
        let handle = OwnedHandle::<uhd_usrp_sys::uhd_sensor_value_t>::new(
            uhd_usrp_sys::uhd_sensor_value_make,
            uhd_usrp_sys::uhd_sensor_value_free,
        )?;
        let f = match self.channel {
            Channel::Rx(_) => uhd_usrp_sys::uhd_usrp_get_tx_sensor,
            Channel::Tx(_) => uhd_usrp_sys::uhd_usrp_get_rx_sensor,
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel.index(),
                handle.as_mut_mut_ptr(),
            )
        })?;
        SensorValue::from_handle(&handle)
    }

    pub fn iter_sensor_values(&self) -> Result<impl Iterator<Item = SensorValue>> {
        let vals = self
            .sensor_names()?
            .into_iter()
            .map(|name| self.sensor_value(&name))
            .collect::<Result<Vec<_>>>()?;
        Ok(vals.into_iter())
    }
}

// --------------------------------------------------------------------------
/// Misc
impl<'u> ChannelConfig<'u> {
    /// Enable/disable the automatic RX DC offset correction.
    /// The automatic correction subtracts out the long-run average.
    ///
    /// When disabled, the averaging option operation is halted.
    /// Once halted, the average value will be held constant until
    /// the user re-enables the automatic correction or overrides
    /// the value by manually setting the offset.
    pub fn set_dc_offset_enabled(&self, en: bool) -> Result<&Self> {
        match self.channel {
            Channel::Rx(_) => {
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_set_rx_dc_offset_enabled(
                        self.usrp.handle().as_mut_ptr(),
                        en,
                        self.channel.index(),
                    )
                })?;
                Ok(self)
            }
            Channel::Tx(_) => Err(UhdError::NotImplemented),
        }
    }

    /// Enable or disable RX IQ imbalance correction for the channel.
    pub fn set_iq_balance_enabled(&self, en: bool) -> Result<&Self> {
        match self.channel {
            Channel::Rx(_) => {
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_set_rx_iq_balance_enabled(
                        self.usrp.handle().as_mut_ptr(),
                        en,
                        self.channel.index(),
                    )
                })?;
                Ok(self)
            }
            Channel::Tx(_) => Err(UhdError::NotImplemented),
        }
    }
}
