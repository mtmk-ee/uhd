//!
//! # uhd
//!
//! Rust bindings for Ettus Research's UHD (Universal Software Radio Peripheral (USRP) Hardware Driver) library.
//!
//! ## Setup
//!
//! 1. [Install UHD](https://files.ettus.com/manual/page_install.html)
//! 2. Add `uhd-usrp` to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! uhd-usrp = "0.1.0"
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use std::time::{Duration, Instant};
//! use num_complex::Complex32;
//! use uhd_usrp::{Usrp, timespec};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a network-attached USRP (e.g. x310).
//!     // Other connection methods can be used as well
//!     let mut usrp = Usrp::open_with_args("addr=192.168.10.4")?;
//!
//!     // Configure the USRP's RX channel zero
//!     usrp.set_rx_config(0)
//!         .set_antenna("RX2")?
//!         .set_center_freq(1030e6)?
//!         .set_bandwidth(2e6)?
//!         .set_gain(None, 0.0)?
//!         .set_sample_rate(4e6)?;
//!
//!     // Open an RX streamer
//!     let mut rx_stream = usrp.rx_stream::<Complex32>().with_channels(&[0]).open()?;
//!
//!     // Allocate a new buffer for receiving samples
//!     let mut buf = vec![Complex32::new(0.0, 0.0); rx_stream.max_samples_per_channel()];
//!
//!     // Start the RX stream in continuous mode with a 500ms delay
//!     rx_stream
//!         .start_command()
//!         .with_time(timespec!(500 ms))
//!         .send()?;
//!
//!     let start_time = Instant::now();
//!     while start_time.elapsed() < Duration::from_secs(10) {
//!         // Receive samples
//!         let samples = rx_stream
//!             .reader()
//!             .with_timeout(Duration::from_millis(100))
//!             .recv(&mut buf)?;
//!
//!         // Do something with the samples
//!         process(&buf[..samples]);
//!     }
//!
//!     Ok(())
//! }
//!
//! fn process(samples: &[Complex32]) {
//!     // Do something with the samples
//! }
//! ```

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

/// Returns the UHD version string.
///
/// Uses `uhd_get_version_string`.
pub fn driver_version() -> String {
    const BUFF_LEN: usize = 16;
    let mut buff = [0u8; BUFF_LEN];
    unsafe { uhd_usrp_sys::uhd_get_version_string(buff.as_mut_ptr().cast(), BUFF_LEN) };
    String::from_utf8(buff.to_vec()).unwrap()
}

/// Returns the UHD ABI version string.
///
/// Uses `uhd_get_abi_string`.
pub fn abi_version() -> String {
    const BUFF_LEN: usize = 16;
    let mut buff = [0u8; BUFF_LEN];
    unsafe { ffi::uhd_get_abi_string(buff.as_mut_ptr().cast(), BUFF_LEN) };
    String::from_utf8(buff.to_vec()).unwrap()
}
