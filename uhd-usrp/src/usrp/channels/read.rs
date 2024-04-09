use std::{ffi::CString, mem::MaybeUninit, ptr::addr_of_mut};

use super::{RX_DIR, TX_DIR};
use crate::{
    error::try_uhd,
    ffi::{FfiString, FfiStringVec, OwnedHandle},
    types::{MetaRange, SensorValue},
    usrp::{Usrp, HardwareInfo},
    Result,
};

// D parameter is a hack until const enum generics are stabilized
pub struct ChannelConfiguration<'usrp, const D: usize> {
    /// The USRP acted upon.
    usrp: &'usrp Usrp,
    /// The specific channel being read.
    channel: usize,
}

impl<'usrp, const D: usize> ChannelConfiguration<'usrp, D> {
    pub(crate) fn new(usrp: &'usrp Usrp, channel: usize) -> Self {
        Self { usrp, channel }
    }

    pub fn antenna(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(16);
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_antenna,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_antenna,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    /// Get a list of antennas associated with the channel.
    pub fn antennas(&self) -> Result<Vec<String>> {
        let mut names = FfiStringVec::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_antennas,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_antennas,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                names.as_mut_ptr(),
            )
        })?;
        Ok(names.to_vec())
    }

    /// Get the bandwidth for the channel's frontend.
    pub fn bandwidth(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_bandwidth_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_bandwidth_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Get the channel's center frequency.
    pub fn center_freq(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_freq_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_freq_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_fe_rx_freq_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_fe_tx_freq_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Set the gain for the channel and given name.
    pub fn gain(&self, name: Option<&str>) -> Result<f64> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_gain_range,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_gain_range,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Fetch names, serial numbers, etc. of the channel's hardware.
    pub fn hardware_info(&self) -> Result<HardwareInfo> {
        match D {
            RX_DIR => {
                let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_rx_info_t> = MaybeUninit::uninit();
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_get_rx_info(
                        self.usrp.handle().as_mut_ptr(),
                        self.channel,
                        info.as_mut_ptr(),
                    )
                })?;
                HardwareInfo::from_rx_raw(unsafe { &info.assume_init() })
            }
            TX_DIR => {
                let mut info: MaybeUninit<uhd_usrp_sys::uhd_usrp_tx_info_t> = MaybeUninit::uninit();
                try_uhd!(unsafe {
                    uhd_usrp_sys::uhd_usrp_get_tx_info(
                        self.usrp.handle().as_mut_ptr(),
                        self.channel,
                        info.as_mut_ptr(),
                    )
                })?;
                HardwareInfo::from_tx_raw(unsafe { &info.assume_init() })
            }
            _ => unreachable!(),
        }
    }

    /// Returns true if the currently selected LO is being exported.
    pub fn lo_export_enabled(&self, name: Option<&str>) -> Result<bool> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut result = false;
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_export_enabled,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_export_enabled,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_freq,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_freq,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_names,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_source,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_source,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
                buf.as_mut_ptr(),
                buf.max_chars(),
            )
        })?;
        buf.into_string()
    }

    /// Get a list of possible LO sources.
    ///
    /// Channels which do not have controllable LO sources will return "internal".
    /// Typical values are "internal" and "external", although the TwinRX, for example, has more options, such as "companion".
    /// These options are device-specific, so consult the individual device manual pages for details.
    pub fn lo_sources(&self, name: Option<&str>) -> Result<Vec<String>> {
        let name = CString::new(name.unwrap_or("")).unwrap();
        let mut vec = FfiStringVec::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_lo_sources,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_lo_sources,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
                vec.as_mut_ptr(),
            )
        })?;
        Ok(vec.to_vec())
    }

    /// Return the normalized gain value.
    ///
    /// This value is linearly mapped to the range [0, 1] for all devices.
    pub fn normalized_gain(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_normalized_rx_gain,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_normalized_tx_gain,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                result.as_mut_ptr(),
            )
        })
        .and_then(|_| Ok(unsafe { result.assume_init() }))
    }

    /// Return the sample rate in samples per second.
    pub fn sample_rate(&self) -> Result<f64> {
        let mut result = std::mem::MaybeUninit::uninit();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_rate,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_rate,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_rates,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_rates,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                handle.as_mut_ptr(),
            )
        })?;
        MetaRange::from_handle(handle)
    }

    /// Get a list of possible frontend sensor names.
    pub fn sensor_names(&self) -> Result<Vec<String>> {
        let mut vec = FfiStringVec::new();
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_sensor_names,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_sensor_names,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
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
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_sensor,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_sensor,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                name.as_ptr(),
                self.channel,
                handle.as_mut_mut_ptr(),
            )
        })?;
        Ok(SensorValue::new(handle))
    }

    /// Get the name of the frontend.
    pub fn subdev_name(&self) -> Result<String> {
        let mut name = FfiString::with_capacity(64);
        let f = match D {
            RX_DIR => uhd_usrp_sys::uhd_usrp_get_rx_subdev_name,
            TX_DIR => uhd_usrp_sys::uhd_usrp_get_tx_subdev_name,
            _ => unreachable!(),
        };
        try_uhd!(unsafe {
            f(
                self.usrp.handle().as_mut_ptr(),
                self.channel,
                name.as_mut_ptr().cast(),
                name.max_chars(),
            )
        })?;
        name.into_string()
    }

    /// Convenience function to print common channel information.
    ///
    /// Info includes:
    /// - Antenna
    /// - Frequency
    /// - Bandwidth
    /// - Gain
    /// - Sample rate
    pub fn print_common(&self) -> Result<()> {
        let antenna = self.antenna()?;
        let freq = self.center_freq()?;
        let bw = self.bandwidth()?;
        let gain = self.gain(None)?;
        let rate = self.sample_rate()?;
        println!("Antenna: {}", antenna);
        println!("Frequency: {} MHz", freq / 1e6);
        println!("Bandwidth: {} MHz", bw / 1e6);
        println!("Gain: {} dB", gain);
        println!("Rate: {} Msps", rate / 1e6);
        Ok(())
    }
}
