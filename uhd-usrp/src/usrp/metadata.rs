use std::ptr::addr_of_mut;

use num_enum::TryFromPrimitive;

use crate::{ffi::OwnedHandle, Result, TimeSpec, UhdError};

pub struct TxMetadataBuilder {
    inner: TxMetadata,
}

impl TxMetadataBuilder {
    pub fn new() -> Self {
        Self {
            inner: TxMetadata::new(),
        }
    }

    pub fn with_time_spec(&mut self, time_spec: TimeSpec) -> &mut Self {
        self.inner.set_time_spec(Some(time_spec));
        self
    }

    pub fn with_start_of_burst(&mut self, sob: bool) -> &mut Self {
        self.inner.set_start_of_burst(sob);
        self
    }

    pub fn with_end_of_burst(&mut self, eob: bool) -> &mut Self {
        self.inner.set_end_of_burst(eob);
        self
    }

    pub fn build(&self) -> TxMetadata {
        self.inner.clone()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TxMetadata {
    time_spec: Option<TimeSpec>,
    start_of_burst: bool,
    end_of_burst: bool,
}

impl TxMetadata {
    pub fn new() -> Self {
        Self {
            time_spec: None,
            start_of_burst: false,
            end_of_burst: false,
        }
    }

    pub fn time_spec(&self) -> Option<TimeSpec> {
        self.time_spec
    }

    pub fn start_of_burst(&self) -> bool {
        self.start_of_burst
    }

    pub fn end_of_burst(&self) -> bool {
        self.end_of_burst
    }

    pub fn set_time_spec(&mut self, time_spec: Option<TimeSpec>) {
        self.time_spec = time_spec;
    }

    pub fn set_end_of_burst(&mut self, eob: bool) {
        self.end_of_burst = eob;
    }

    pub fn set_start_of_burst(&mut self, sob: bool) {
        self.start_of_burst = sob;
    }

    pub(crate) fn to_handle(&self) -> OwnedHandle<uhd_usrp_sys::uhd_tx_metadata_t> {
        let mut handle = std::ptr::null_mut();
        let (full_secs, frac_secs) = self
            .time_spec
            .map(|d| (d.full_secs() as i64, d.frac_secs()))
            .unwrap_or_default();
        unsafe {
            uhd_usrp_sys::uhd_tx_metadata_make(
                addr_of_mut!(handle),
                self.time_spec.is_some(),
                full_secs,
                frac_secs,
                self.start_of_burst,
                self.end_of_burst,
            )
        };
        unsafe { OwnedHandle::from_ptr(handle, uhd_usrp_sys::uhd_tx_metadata_free) }
    }
}

#[derive(Clone, Copy, Debug, num_enum::TryFromPrimitive)]
#[repr(u32)]
pub enum RxErrorCode {
    None = uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_NONE,
    Timeout = uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_TIMEOUT,
    LateCommand =
        uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_LATE_COMMAND,
    BrokenChain =
        uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_BROKEN_CHAIN,
    Overflow = uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_OVERFLOW,
    Alignment = uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_ALIGNMENT,
    BadPacket = uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_BAD_PACKET,
}

pub struct RxMetadata {
    handle: OwnedHandle<uhd_usrp_sys::uhd_rx_metadata_t>,
}

impl RxMetadata {
    pub fn new() -> Self {
        Self {
            // `uhd_rx_metadata_make` is a simple allocation so it is safe in
            // the overwhelming majority of cases, and handling OOM makes this
            // type unergonomic.
            handle: OwnedHandle::new(
                uhd_usrp_sys::uhd_rx_metadata_make,
                uhd_usrp_sys::uhd_rx_metadata_free,
            )
            .expect("uhd_rx_metadata_make failed"),
        }
    }

    pub(crate) fn handle(&self) -> &OwnedHandle<uhd_usrp_sys::uhd_rx_metadata_t> {
        &self.handle
    }

    #[allow(dead_code)]
    pub(crate) fn handle_mut(&mut self) -> &mut OwnedHandle<uhd_usrp_sys::uhd_rx_metadata_t> {
        &mut self.handle
    }

    pub fn end_of_burst(&self) -> bool {
        let mut result = false;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_end_of_burst(
                self.handle.as_mut_ptr(),
                addr_of_mut!(result),
            );
        }
        result
    }

    pub fn error_code(&self) -> Result<RxErrorCode> {
        let mut result =
            uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_NONE;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_error_code(self.handle.as_mut_ptr(), addr_of_mut!(result))
        };
        Ok(RxErrorCode::try_from_primitive(result).or(Err(UhdError::Unknown))?)
    }

    pub fn fragment_offset(&self) -> usize {
        let mut result = 0;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_fragment_offset(
                self.handle.as_mut_ptr(),
                addr_of_mut!(result),
            );
        }
        result
    }

    pub fn more_fragments(&self) -> bool {
        let mut result = false;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_more_fragments(
                self.handle.as_mut_ptr(),
                addr_of_mut!(result),
            );
        }
        result
    }

    pub fn out_of_sequence(&self) -> bool {
        let mut result = false;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_out_of_sequence(
                self.handle.as_mut_ptr(),
                addr_of_mut!(result),
            );
        }
        result
    }

    pub fn start_of_burst(&self) -> bool {
        let mut result = false;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_start_of_burst(
                self.handle.as_mut_ptr(),
                addr_of_mut!(result),
            );
        }
        result
    }

    pub fn time_spec(&self) -> Option<TimeSpec> {
        let mut has_time_spec = false;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_has_time_spec(
                self.handle.as_mut_ptr(),
                addr_of_mut!(has_time_spec),
            );
        }
        if !has_time_spec {
            return None;
        }

        let mut full_secs = 0;
        let mut frac_secs = 0.0;
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_time_spec(
                self.handle.as_mut_ptr(),
                addr_of_mut!(full_secs),
                addr_of_mut!(frac_secs),
            );
        }
        // `TimeSpec::from_parts_unchecked` is an option, but it safer to check
        // since we don't have any solid guarantees about the validity of the
        // returned timespec.
        TimeSpec::try_from_parts(full_secs, frac_secs)
    }
}
