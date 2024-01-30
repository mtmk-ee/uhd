mod error;
pub(crate) mod misc_types;
pub mod usrp;
pub(crate) mod util;

pub use error::{last_error_message, Result, UhdError};

use crate::error::try_uhd;

pub fn driver_version() -> Result<String> {
    const BUFF_LEN: usize = 16;
    let mut buff = [0u8; BUFF_LEN];
    try_uhd!(unsafe { uhd_usrp_sys::uhd_get_version_string(buff.as_mut_ptr().cast(), BUFF_LEN) })?;
    String::from_utf8(buff.to_vec()).or(Err(UhdError::Unknown))
}
