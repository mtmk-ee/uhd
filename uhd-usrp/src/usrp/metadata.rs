use std::{ptr::addr_of_mut, time::Duration};

use num_enum::TryFromPrimitive;
use uhd_usrp_sys::uhd_rx_metadata_handle;

use crate::{error::try_uhd, Result, UhdError};

#[derive(Clone, Copy, Debug, Default)]
pub struct TxMetadata {
    offset: Option<Duration>,
    start_of_burst: bool,
    end_of_burst: bool,
}

impl TxMetadata {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn offset(mut self, offset: Option<Duration>) -> Self {
        self.offset = offset;
        self
    }

    pub fn start_of_burst(mut self, sob: bool) -> Self {
        self.start_of_burst = sob;
        self
    }
    pub fn end_of_burst(mut self, eob: bool) -> Self {
        self.end_of_burst = eob;
        self
    }
}

pub(crate) struct TxMetadataHandle(uhd_usrp_sys::uhd_tx_metadata_handle);

impl TxMetadataHandle {
    pub fn from_metadata(metadata: &TxMetadata) -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        let (full_secs, frac_secs) = match metadata.offset {
            Some(dur) => (
                dur.as_secs() as i64,
                dur.as_secs_f64() - dur.as_secs() as f64,
            ),
            None => (0i64, 0f64),
        };
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_metadata_make(
                addr_of_mut!(handle),
                metadata.offset.is_some(),
                full_secs,
                frac_secs,
                metadata.start_of_burst,
                metadata.end_of_burst,
            )
        })?;
        Ok(TxMetadataHandle(handle))
    }

    pub fn handle(&self) -> uhd_usrp_sys::uhd_tx_metadata_handle {
        self.0
    }
}

impl Drop for TxMetadataHandle {
    fn drop(&mut self) {
        unsafe {
            uhd_usrp_sys::uhd_tx_metadata_free(addr_of_mut!(self.0));
        }
    }
}

#[derive(Clone, Copy, Debug, num_enum::TryFromPrimitive)]
#[repr(u32)]
pub enum RxErrorcode {
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


pub struct RxMetadata(uhd_usrp_sys::uhd_rx_metadata_handle);

impl RxMetadata {
    pub fn new() -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_rx_metadata_make(addr_of_mut!(handle)) })?;
        Ok(Self(handle))
    }

    pub fn handle(&self) -> uhd_rx_metadata_handle {
        self.0
    }

    pub fn time_spec(&self) -> Result<Option<Duration>> {
        let mut has_time_spec = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_has_time_spec(self.0, addr_of_mut!(has_time_spec))
        })?;
        if !has_time_spec {
            return Ok(None);
        }

        let mut full_secs = 0;
        let mut frac_secs = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_time_spec(
                self.0,
                addr_of_mut!(full_secs),
                addr_of_mut!(frac_secs),
            )
        })?;
        // TODO: may have precision issues for large times
        Ok(Some(Duration::from_secs_f64(full_secs as f64 + frac_secs)))
    }

    pub fn start_of_burst(&self) -> Result<bool> {
        let mut result = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_start_of_burst(self.0, addr_of_mut!(result))
        })?;
        Ok(result)
    }

    pub fn end_of_burst(&self) -> Result<bool> {
        let mut result = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_end_of_burst(self.0, addr_of_mut!(result))
        })?;
        Ok(result)
    }

    pub fn fragment_offset(&self) -> Result<usize> {
        let mut result = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_fragment_offset(self.0, addr_of_mut!(result))
        })?;
        Ok(result)
    }

    pub fn more_fragments(&self) -> Result<bool> {
        let mut result = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_more_fragments(self.0, addr_of_mut!(result))
        })?;
        Ok(result)
    }

    pub fn out_of_sequence(&self) -> Result<bool> {
        let mut result = false;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_out_of_sequence(self.0, addr_of_mut!(result))
        })?;
        Ok(result)
    }

    pub fn error_code(&self) -> Result<RxErrorcode> {
        let mut result =
            uhd_usrp_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_NONE;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_metadata_error_code(self.0, addr_of_mut!(result))
        })?;
        Ok(RxErrorcode::try_from_primitive(result).or(Err(UhdError::Unknown))?)
    }
}

impl Drop for RxMetadata {
    fn drop(&mut self) {
        unsafe {
            uhd_usrp_sys::uhd_rx_metadata_free(addr_of_mut!(self.0));
        }
    }
}
