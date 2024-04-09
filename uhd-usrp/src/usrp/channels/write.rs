use std::{ffi::CString, ptr::addr_of_mut};

use super::{RX_DIR, TX_DIR};
use crate::{
    error::try_uhd,
    types::{TuneRequest, TuneResult},
    usrp::Usrp,
    Result,
};

pub struct ChannelConfigurationBuilder<'usrp, const D: usize> {
    usrp: &'usrp Usrp,
    channel: usize,
}

impl<'usrp, const D: usize> ChannelConfigurationBuilder<'usrp, D> {
    /// Select the antenna to use on the frontend.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid antenna name is provided.
    pub fn set_antenna(self, name: &str) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_antenna,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_antenna,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), name.as_ptr(), self.channel) })?;
        Ok(self)
    }

    /// Set the RX frontend's bandwidth in Hz.
    ///
    /// If a bandwidth is provided that is outside the valid range,
    /// it is coerced to the nearest valid value.
    pub fn set_bandwidth(self, bw: f64) -> Result<Self> {
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_bandwidth,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_bandwidth,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), bw, self.channel) })?;
        Ok(self)
    }

    /// Set the RX center frequency in Hz.
    ///
    /// If the requested frequency is outside of the valid frequency range,
    /// it will be coerced to the nearest valid frequency.
    pub fn set_center_freq(self, freq: f64) -> Result<Self> {
        self.tune(&TuneRequest::new(freq).rf_freq_auto().dsp_freq_auto())
    }

    /// Set the RX gain value in dB for the specified gain element.
    ///
    /// If the requested gain value is outside the valid range,
    /// it will be coerced to a valid gain value.
    ///
    /// The name of the gain element to set can be provided.
    /// If `None`, it is distributed across all gain elements.
    pub fn set_gain(self, name: Option<&str>, gain: f64) -> Result<Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                gain,
                self.channel,
                name.as_ptr(),
            )
        })?;
        Ok(self)
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
                self.usrp.handle().as_mut_ptr(),
                freq,
                name.as_ptr(),
                self.channel,
                addr_of_mut!(result),
            )
        })?;
        Ok(self)
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
    pub fn set_normalized_gain(self, gain: f64) -> Result<Self> {
        if gain < 0.0 || gain > 1.0 {
            panic!("gain must be 0 to 1");
        }
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_normalized_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_normalized_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), gain, self.channel) })?;
        Ok(self)
    }

    /// Set the RX sample rate in samples per second.
    ///
    /// This function will coerce the requested rate to a rate that the
    /// device can handle. A warning may be logged during coercion.
    ///
    /// # Panics
    ///
    /// Panics if the given rate is non-positive.
    pub fn set_sample_rate(self, rate: f64) -> Result<Self> {
        if rate <= 0.0 {
            panic!("sample rate must be positive");
        }
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_rate,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_rate,
            _ => unreachable!(),
        };
        try_uhd!(unsafe { f(self.usrp.handle().as_mut_ptr(), rate, self.channel) })?;
        Ok(self)
    }

    /// Set the tuning parameters for the channel.
    ///
    /// This function allows setting more advanced parameters.
    pub fn tune(self, req: &TuneRequest) -> Result<Self> {
        let req = req.inner();
        let mut result = TuneResult::default();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_set_rx_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_set_tx_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
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
    pub fn set_agc_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_agc(self.usrp.handle().as_mut_ptr(), en, self.channel)
        })?;
        Ok(self)
    }

    /// Enable/disable the automatic RX DC offset correction.
    /// The automatic correction subtracts out the long-run average.
    ///
    /// When disabled, the averaging option operation is halted.
    /// Once halted, the average value will be held constant until
    /// the user re-enables the automatic correction or overrides
    /// the value by manually setting the offset.
    pub fn set_dc_offset_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_dc_offset_enabled(
                self.usrp.handle().as_mut_ptr(),
                en,
                self.channel,
            )
        })?;
        Ok(self)
    }

    /// Enable or disable RX IQ imbalance correction for the channel.
    pub fn set_iq_balance_enabled(self, en: bool) -> Result<Self> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_iq_balance_enabled(
                self.usrp.handle().as_mut_ptr(),
                en,
                self.channel,
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
    pub fn set_lo_export_enabled(self, name: Option<&str>, en: bool) -> Result<Self> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_rx_lo_export_enabled(
                self.usrp.handle().as_mut_ptr(),
                en,
                name.as_ptr(),
                self.channel,
            )
        })?;
        Ok(self)
    }
}
