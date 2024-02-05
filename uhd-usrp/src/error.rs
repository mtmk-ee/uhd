use std::ffi::CString;

use uhd_usrp_sys::uhd_error;

pub type Result<T, E = UhdError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug, Clone)]
#[repr(u32)]
pub enum UhdError {
    #[error("invalid device arguments")]
    InvalidDevice = uhd_error::UHD_ERROR_INVALID_DEVICE as u32,
    #[error("index out of range")]
    Index = uhd_error::UHD_ERROR_INDEX as u32,
    #[error("invalid key")]
    Key = uhd_error::UHD_ERROR_KEY as u32,
    #[error("function is not yet implemented")]
    NotImplemented = uhd_error::UHD_ERROR_NOT_IMPLEMENTED as u32,
    #[error("error occurred during USB transaction")]
    Usb = uhd_error::UHD_ERROR_USB as u32,
    #[error("error occurred during I/O")]
    Io = uhd_error::UHD_ERROR_IO as u32,
    #[error("system-related error")]
    Os = uhd_error::UHD_ERROR_OS as u32,
    #[error("assertion failed")]
    Assertion = uhd_error::UHD_ERROR_ASSERTION as u32,
    #[error("key or index is invalid")]
    Lookup = uhd_error::UHD_ERROR_LOOKUP as u32,
    #[error("function called with incorrect types")]
    Type = uhd_error::UHD_ERROR_TYPE as u32,
    #[error("function called with incorrect values")]
    Value = uhd_error::UHD_ERROR_VALUE as u32,
    #[error("runtime error")]
    Runtime = uhd_error::UHD_ERROR_RUNTIME as u32,
    #[error("external error")]
    Environment = uhd_error::UHD_ERROR_ENVIRONMENT as u32,
    #[error("system error")]
    System = uhd_error::UHD_ERROR_SYSTEM as u32,
    #[error("uhd exception thrown")]
    Except = uhd_error::UHD_ERROR_EXCEPT as u32,
    #[error("boost::exception thrown")]
    BoostExcept = uhd_error::UHD_ERROR_BOOSTEXCEPT as u32,
    #[error("std::exception thrown")]
    StdExcept = uhd_error::UHD_ERROR_STDEXCEPT as u32,
    #[error("unknown error occurred")]
    Unknown = uhd_error::UHD_ERROR_UNKNOWN as u32,
}

impl<T> Into<Result<T>> for UhdError {
    fn into(self) -> Result<T> {
        Err(self)
    }
}

impl UhdError {
    pub(crate) fn from_sys(e: u32) -> Option<Self> {
        match e {
            uhd_error::UHD_ERROR_NONE => None,

            uhd_error::UHD_ERROR_INVALID_DEVICE => Some(UhdError::InvalidDevice),
            uhd_error::UHD_ERROR_INDEX => Some(UhdError::Index),
            uhd_error::UHD_ERROR_KEY => Some(UhdError::Key),
            uhd_error::UHD_ERROR_NOT_IMPLEMENTED => Some(UhdError::NotImplemented),
            uhd_error::UHD_ERROR_USB => Some(UhdError::Usb),
            uhd_error::UHD_ERROR_IO => Some(UhdError::Io),
            uhd_error::UHD_ERROR_OS => Some(UhdError::Os),
            uhd_error::UHD_ERROR_ASSERTION => Some(UhdError::Assertion),
            uhd_error::UHD_ERROR_LOOKUP => Some(UhdError::Lookup),
            uhd_error::UHD_ERROR_TYPE => Some(UhdError::Type),
            uhd_error::UHD_ERROR_VALUE => Some(UhdError::Value),
            uhd_error::UHD_ERROR_RUNTIME => Some(UhdError::Runtime),
            uhd_error::UHD_ERROR_ENVIRONMENT => Some(UhdError::Environment),
            uhd_error::UHD_ERROR_SYSTEM => Some(UhdError::System),
            uhd_error::UHD_ERROR_EXCEPT => Some(UhdError::Except),
            uhd_error::UHD_ERROR_BOOSTEXCEPT => Some(UhdError::BoostExcept),
            uhd_error::UHD_ERROR_STDEXCEPT => Some(UhdError::StdExcept),
            _ => Some(UhdError::Unknown),
        }
    }
}

macro_rules! try_uhd {
    ($e: expr) => {
        match $crate::UhdError::from_sys($e) {
            None => Ok(()),
            Some(e) => Err(e),
        }
    };
}
pub(crate) use try_uhd;

pub fn last_error_message() -> Result<String> {
    let mut message: [u8; 128] = [0; 128];
    try_uhd!(unsafe {
        uhd_usrp_sys::uhd_get_last_error(message.as_mut_ptr().cast(), message.len())
    })?;
    Ok(CString::new(message)
        .or(Err(UhdError::Unknown))?
        .to_string_lossy()
        .into_owned())
}
