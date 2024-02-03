#[repr(u32)]
pub enum TuneRequestPolicy {
    None = uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_NONE,
    Auto = uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_AUTO,
    Manual = uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_MANUAL,
}

#[derive(Clone, Debug)]
pub struct TuneRequest {
    inner: uhd_usrp_sys::uhd_tune_request_t,
}

impl TuneRequest {
    pub fn new() -> Self {
        Self {
            inner: uhd_usrp_sys::uhd_tune_request_t {
                target_freq: 0.0,
                rf_freq_policy:
                    uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_AUTO,
                rf_freq: 0.0,
                dsp_freq_policy:
                    uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_AUTO,
                dsp_freq: 0.0,
                args: std::ptr::null_mut(),
            },
        }
    }

    pub(crate) fn inner(&self) -> &uhd_usrp_sys::uhd_tune_request_t {
        &self.inner
    }

    pub fn center_freq(mut self, freq: f64) -> Self {
        self.inner.target_freq = freq;
        self
    }

    pub fn rf_freq_auto(mut self) -> Self {
        self.inner.rf_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_AUTO;
        self
    }

    pub fn rf_freq_unset(mut self) -> Self {
        self.inner.rf_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_NONE;
        self
    }

    pub fn rf_freq_manual(mut self, freq: f64) -> Self {
        self.inner.rf_freq = freq;
        self.inner.rf_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_MANUAL;
        self
    }

    pub fn dsp_freq_auto(mut self) -> Self {
        self.inner.dsp_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_AUTO;
        self
    }

    pub fn dsp_freq_unset(mut self) -> Self {
        self.inner.dsp_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_NONE;
        self
    }

    pub fn dsp_freq_manual(mut self, freq: f64) -> Self {
        self.inner.dsp_freq = freq;
        self.inner.dsp_freq_policy =
            uhd_usrp_sys::uhd_tune_request_policy_t::UHD_TUNE_REQUEST_POLICY_MANUAL;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TuneResult {
    inner: uhd_usrp_sys::uhd_tune_result_t,
}

impl TuneResult {
    pub(crate) fn new() -> Self {
        Self {
            inner: uhd_usrp_sys::uhd_tune_result_t {
                clipped_rf_freq: 0.0,
                target_rf_freq: 0.0,
                actual_rf_freq: 0.0,
                target_dsp_freq: 0.0,
                actual_dsp_freq: 0.0,
            },
        }
    }

    pub(crate) fn inner(&mut self) -> &uhd_usrp_sys::uhd_tune_result_t {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut uhd_usrp_sys::uhd_tune_result_t {
        &mut self.inner
    }

    pub fn clipped_rf_freq(&self) -> f64 {
        self.inner.clipped_rf_freq
    }

    pub fn actual_rf_freq(&self) -> f64 {
        self.inner.actual_rf_freq
    }

    pub fn actual_dsp_freq(&self) -> f64 {
        self.inner.actual_rf_freq
    }
}
