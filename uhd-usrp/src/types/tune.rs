#![allow(unused)]

use std::{
    ffi::CString,
    ptr::{addr_of, addr_of_mut},
};

use uhd_usrp_sys::uhd_tune_request_policy_t::*;

use crate::ffi::FfiString;

/// A tune request instructs the implementation how to tune the RF chain.
///
/// The policies can be used to select automatic tuning or fine control
/// over the daughterboard IF and DSP tuning. Not all combinations of
/// policies are applicable. Convenience constructors are supplied for
/// most use cases.
#[derive(Clone, Debug)]
pub struct TuneRequest {
    inner: uhd_usrp_sys::uhd_tune_request_t,
}

/// Contains the RF and DSP tuned frequencies.
#[derive(Clone, Debug, Default)]
pub struct TuneResult {
    inner: uhd_usrp_sys::uhd_tune_result_t,
}

impl TuneRequest {
    /// Make a new tune request for a particular center frequency.
    ///
    /// Defaults to an automatic policy for the RF and DSP frequency to tune the chain
    /// as close as possible to the target frequency.
    ///
    /// Note: there is currently no support for specifying additional arguments.
    pub fn new(target_freq: f64) -> Self {
        Self {
            inner: uhd_usrp_sys::uhd_tune_request_t {
                target_freq,
                rf_freq_policy: UHD_TUNE_REQUEST_POLICY_AUTO,
                rf_freq: 0.0,
                dsp_freq_policy: UHD_TUNE_REQUEST_POLICY_AUTO,
                dsp_freq: 0.0,
                /// TODO: add support for args
                args: std::ptr::null_mut(),
            },
        }
    }

    /// Make a new tune request for a particular center frequency.
    ///
    /// Use a manual policy for the RF frequency, and an automatic policy
    /// for the DSP frequency, to tune the chain as close as possible to
    /// the target frequency.
    pub fn with_lo_offset(target_freq: f64, lo_offset: f64) -> Self {
        Self::new(target_freq)
            .rf_freq_manual(target_freq + lo_offset)
            .dsp_freq_auto()
    }

    /// Retrieve the inner [`uhd_tune_request_t`](uhd_usrp_sys::uhd_tune_request_t) struct.
    pub(crate) fn inner(&self) -> &uhd_usrp_sys::uhd_tune_request_t {
        &self.inner
    }

    /// Retrieve the inner [`uhd_tune_request_t`](uhd_usrp_sys::uhd_tune_request_t) struct.
    pub(crate) fn inner_mut(&mut self) -> &mut uhd_usrp_sys::uhd_tune_request_t {
        &mut self.inner
    }

    /// Automatically set the DSP frequency (default).
    ///
    /// This will automatically set the value to the difference between the target and IF.
    pub fn dsp_freq_auto(mut self) -> Self {
        self.inner.dsp_freq_policy = UHD_TUNE_REQUEST_POLICY_AUTO;
        self
    }

    /// Manually set the DSP frequency in Hz.
    ///
    /// Note that the meaning of the DSP frequency's sign differs between TX and RX operations.
    /// The target frequency is the result of `target_freq = rf_freq + sign * dsp_freq`.
    /// For TX, sign is negative, and for RX, sign is positive.
    /// Example: If both RF and DSP tuning policies are set to manual, and rf_freq is set to
    /// 1 GHz, and dsp_freq is set to 10 MHz, the actual target frequency is 990 MHz for a
    /// TX tune request, and 1010 MHz for an RX tune request.
    pub fn dsp_freq_manual(mut self, freq: f64) -> Self {
        self.inner.dsp_freq = freq;
        self.inner.dsp_freq_policy = UHD_TUNE_REQUEST_POLICY_MANUAL;
        self
    }

    /// Specify that the DSP frequency should not be changed.
    pub fn dsp_freq_unset(mut self) -> Self {
        self.inner.dsp_freq_policy = UHD_TUNE_REQUEST_POLICY_NONE;
        self
    }

    /// Automatically set the RF frequency (default).
    ///
    /// This will automatically set the value to the `target frequency + default LO offset`.
    pub fn rf_freq_auto(mut self) -> Self {
        self.inner.rf_freq_policy = UHD_TUNE_REQUEST_POLICY_AUTO;
        self
    }

    /// Manually set the RF frequency in Hz.
    pub fn rf_freq_manual(mut self, freq: f64) -> Self {
        self.inner.rf_freq = freq;
        self.inner.rf_freq_policy = UHD_TUNE_REQUEST_POLICY_MANUAL;
        self
    }

    /// Specify that the RF frequency should not be changed.
    pub fn rf_freq_unset(mut self) -> Self {
        self.inner.rf_freq_policy = UHD_TUNE_REQUEST_POLICY_NONE;
        self
    }
}

impl TuneResult {
    /// Retrieve the inner [`uhd_tune_result_t`](uhd_usrp_sys::uhd_tune_result_t) struct.
    pub(crate) fn inner(&mut self) -> &uhd_usrp_sys::uhd_tune_result_t {
        &self.inner
    }

    /// Retrieve the inner [`uhd_tune_result_t`](uhd_usrp_sys::uhd_tune_result_t) struct.
    pub(crate) fn inner_mut(&mut self) -> &mut uhd_usrp_sys::uhd_tune_result_t {
        &mut self.inner
    }

    /// The frequency to which the CORDIC in the DSP actually tuned
    ///
    /// If we failed to hit the target DSP frequency, it is either because the
    /// requested resolution wasn't possible or something went wrong in the DSP.
    /// In most cases, it should equal the `target_dsp_freq` above.
    pub fn actual_dsp_freq(&self) -> f64 {
        self.inner.actual_rf_freq
    }

    /// The frequency to which the RF LO actually tuned
    ///
    /// If this does not equal the target_rf_freq, then it is because the target
    /// was outside of the range of the LO, or the LO was not able to hit it
    /// exactly due to tuning accuracy.
    pub fn actual_rf_freq(&self) -> f64 {
        self.inner.actual_rf_freq
    }

    /// The target RF frequency, clipped to be within system range
    ///
    /// If the requested frequency is within the range of the system,
    /// then this variable will equal the requested frequency.
    /// If the requested frequency is outside of the tunable range,
    /// however, this variable will hold the value that it was 'clipped'
    /// to in order to keep tuning in-bounds.
    pub fn clipped_rf_freq(&self) -> f64 {
        self.inner.clipped_rf_freq
    }

    /// The frequency the CORDIC must adjust the RF.
    ///
    /// **When automatically set:**
    /// It is fairly common for the RF LO to not be able to exactly
    /// hit the requested frequency. This value holds the required
    /// adjustment the CORDIC must make to the signal to bring it to the
    /// requested center frequency.
    ///
    /// **When manually set:**
    /// This value equals the DSP frequency in the tune request,
    /// clipped to be within range of the DSP if it was outside.
    pub fn target_dsp_freq(&self) -> f64 {
        self.inner.target_dsp_freq
    }

    /// Target RF Freq, including RF FE offset
    ///
    /// **When automatically set:**
    /// This value holds the requested center frequency, plus any LO offset
    /// required by the radio front-end. Note that this is not the LO offset
    /// requested by the user (if one exists), but rather one required by the
    /// hardware (if required).
    ///
    /// **When manually set:**
    /// This value equals the RF frequency in the tune request.
    pub fn target_rf_freq(&self) -> f64 {
        self.inner.target_rf_freq
    }
}

impl ToString for TuneResult {
    fn to_string(&self) -> String {
        format!(
            "Tune Result:\n\
            Target RF  Freq: {} (MHz)\n\
            Actual RF  Freq: {} (MHz)\n\
            Target DSP Freq: {} (MHz)\n\
            Actual DSP Freq: {} (MHz)",
            self.target_rf_freq() / 1e6,
            self.actual_rf_freq() / 1e6,
            self.target_dsp_freq() / 1e6,
            self.actual_dsp_freq() / 1e6
        )
    }
}
