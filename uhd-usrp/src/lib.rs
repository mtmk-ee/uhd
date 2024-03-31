mod buffer;
mod error;
pub(crate) mod ffi;
pub mod logging;
pub(crate) mod misc_types;
mod sample;
mod time;
pub mod usrp;

pub use buffer::{ArrayBuffer, SampleBuffer};
pub use error::{last_error_message, Result, UhdError};
pub use misc_types::*;
pub use sample::Sample;
pub use time::TimeSpec;
pub use usrp::*;

pub(crate) use crate::error::try_uhd;

pub fn driver_version() -> String {
    const BUFF_LEN: usize = 16;
    let mut buff = [0u8; BUFF_LEN];
    unsafe { uhd_usrp_sys::uhd_get_version_string(buff.as_mut_ptr().cast(), BUFF_LEN) };
    String::from_utf8(buff.to_vec()).unwrap()
}

pub fn abi_version() -> String {
    const BUFF_LEN: usize = 16;
    let mut buff = [0u8; BUFF_LEN];
    unsafe { ffi::uhd_get_abi_string(buff.as_mut_ptr().cast(), BUFF_LEN) };
    String::from_utf8(buff.to_vec()).unwrap()
}

