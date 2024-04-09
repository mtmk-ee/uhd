use std::{ffi::CString, marker::PhantomData, ptr::addr_of_mut};

use crate::{
    error::try_uhd,
    ffi::OwnedHandle,
    stream::{RxStreamBuilder, TxStreamBuilder},
    types::DeviceArgs,
    Result, Sample, TimeSpec,
};

use super::{
    channels::{ChannelConfiguration, ChannelConfigurationBuilder, RX_DIR, TX_DIR},
    mboard::Motherboard,
};

/// The entry point for interacting with a connected USRP.
///
/// A USRP instance can be opened using one of the following methods:
/// - `Usrp::open_any()` to open any recognized USRP
/// - `Usrp::open_with_args()` and a typical "key=value"` string
/// - `Usrp::open()` and a `DeviceArgs` struct for the most flexibility.
///
/// # Examples
///
/// ```no_run
///
/// use std::time::{Duration, Instant};
/// use num_complex::Complex32;
/// use uhd_usrp::{Usrp, timespec};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Open a network-attached USRP (e.g. x310).
///     // Other connection methods can be used as well
///     let mut usrp = Usrp::open_with_args("addr=192.168.10.4")?;
///
///     // Configure the USRP's RX channel zero
///     usrp.set_rx_config(0)
///         .set_antenna("RX2")?
///         .set_center_freq(1030e6)?
///         .set_bandwidth(2e6)?
///         .set_gain(None, 0.0)?
///         .set_sample_rate(4e6)?;
///
///     // Open an RX streamer
///     let mut rx_stream = usrp.rx_stream::<Complex32>().with_channels(&[0]).open()?;
///
///     // Allocate a new buffer for receiving samples
///     let mut buf = vec![Complex32::new(0.0, 0.0); rx_stream.max_samples_per_channel()];
///
///     // Start the RX stream in continuous mode with a 500ms delay
///     rx_stream
///         .start_command()
///         .with_time(timespec!(500 ms))
///         .send()?;
///
///     let start_time = Instant::now();
///     while start_time.elapsed() < Duration::from_secs(10) {
///         // Receive samples
///         let samples = rx_stream
///             .reader()
///             .with_timeout(Duration::from_millis(100))
///             .recv(&mut buf)?;
///
///         // Do something with the samples
///         process(&buf[..samples]);
///     }
///
///     Ok(())
/// }
///
/// fn process(samples: &[Complex32]) {
///     // Do something with the samples
/// }
/// ```
pub struct Usrp {
    handle: OwnedHandle<uhd_usrp_sys::uhd_usrp>,
    _unsync: PhantomData<std::cell::Cell<()>>,
}

impl Usrp {
    /// Attempts to open a USRP using the given [`DeviceArgs`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Usrp};
    ///
    /// let args = DeviceArgs::new()
    ///     .addr("192.168.10.4");
    ///
    /// let usrp = Usrp::open(args).expect("failed to open USRP");
    /// ```
    pub fn open(args: DeviceArgs) -> Result<Self> {
        Self::open_with_args(&args.to_string())
    }

    /// Open any connected USRP.
    ///
    /// The behavior of this function is not guaranteed to be consistent if multiple USRPs are connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Usrp};
    ///
    /// let usrp = Usrp::open_any().expect("failed to open USRP");
    /// ```
    pub fn open_any() -> Result<Self> {
        Self::open_with_args("")
    }

    /// Open a USRP using `"key=value"` style arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Usrp};
    ///
    /// let usrp = Usrp::open_with_args("addr=192.168.10.4").expect("failed to open USRP");
    /// ```
    pub fn open_with_args(args: &str) -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        let args = CString::new(args).unwrap();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_usrp_make(addr_of_mut!(handle), args.as_ptr()) })?;
        Ok(Self {
            handle: unsafe { OwnedHandle::from_ptr(handle, uhd_usrp_sys::uhd_usrp_free) },
            _unsync: PhantomData::default(),
        })
    }

    /// Get a reference to the underlying [`OwnedHandle`].
    pub(crate) fn handle(&self) -> &OwnedHandle<uhd_usrp_sys::uhd_usrp> {
        &self.handle
    }

    /// Access per-motherboard properties.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Result, Usrp};
    /// use uhd_usrp::types::SensorValue;
    ///
    /// fn fetch_sensor_values(usrp: &Usrp, mboard: usize) -> Result<Vec<SensorValue>> {
    ///     let sensor_values = usrp
    ///         .mboard(mboard)
    ///         .sensor_names()?
    ///         .into_iter()
    ///         .map(|name| usrp.mboard(mboard).sensor_value(&name))
    ///         .collect::<Result<Vec<SensorValue>>>()?;
    ///     Ok(sensor_values)
    /// }
    /// ```
    #[must_use]
    pub fn mboard(&self, mboard: usize) -> Motherboard {
        Motherboard::new(self, mboard)
    }

    /// Get the number of connected motherboards.
    pub fn n_mboards(&self) -> Result<usize> {
        let mut mboards = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_num_mboards(self.handle.as_mut_ptr(), addr_of_mut!(mboards))
        })?;
        Ok(mboards)
    }

    /// Synchronize the times across all motherboards in this configuration.
    ///
    /// Use this method to sync the times when the edge of the PPS is unknown.
    /// The provided [`TimeSpec`] will be latched at the next PPS after catching
    /// the edge.
    ///
    /// Ex: Host machine is not attached to serial port of GPSDO and can therefore
    /// not query the GPSDO for the PPS edge.
    ///
    /// This is a 2-step process, and will take at most 2 seconds to complete.
    /// Upon completion, the times will be synchronized to the time provided.
    ///
    /// 1. wait for the last pps time to transition to catch the edge
    /// 2. set the time at the next pps (synchronous for all boards)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{timespec, Usrp};
    ///
    /// let mut usrp = Usrp::open_any().expect("failed to open USRP");
    /// usrp.set_time_unknown_pps(timespec!(0));
    /// ```
    pub fn set_time_unknown_pps(&mut self, time: TimeSpec) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_unknown_pps(
                self.handle.as_mut_ptr(),
                time.full_secs(),
                time.frac_secs(),
            )
        })?;
        Ok(())
    }
}

/// RX and TX streaming.
impl Usrp {
    /// Get the total number of RX channels on this USRP.
    pub fn rx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_num_channels(
                self.handle.as_mut_ptr(),
                addr_of_mut!(channels),
            )
        })?;
        Ok(channels)
    }

    /// Returns a builder for opening an RX stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use num_complex::Complex32;
    /// use uhd_usrp::Usrp;
    ///
    /// let mut usrp = Usrp::open_any().expect("failed to open USRP");
    ///
    /// // <insert setup code here>
    ///
    /// // Open an RX streamer
    /// let rx_stream = usrp.rx_stream::<Complex32>()
    ///     .with_channels(&[0])
    ///     .open()
    ///     .expect("failed to open RX stream");
    /// ```
    #[must_use]
    pub fn rx_stream<T: Sample>(&self) -> RxStreamBuilder<T> {
        RxStreamBuilder::new(self)
    }

    /// Get the total number of Tx channels on this USRP.
    pub fn tx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_num_channels(
                self.handle.as_mut_ptr(),
                addr_of_mut!(channels),
            )
        })?;
        Ok(channels)
    }

    /// Returns a builder for opening an TX stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use num_complex::Complex32;
    /// use uhd_usrp::Usrp;
    ///
    /// let mut usrp = Usrp::open_any().expect("failed to open USRP");
    ///
    /// // <insert setup code here>
    ///
    /// // Open an TX streamer
    /// let rx_stream = usrp.tx_stream::<Complex32>()
    ///     .with_channels(&[0])
    ///     .open()
    ///     .expect("failed to open TX stream");
    /// ```
    #[must_use]
    pub fn tx_stream<T: Sample>(&self) -> TxStreamBuilder<T> {
        TxStreamBuilder::new(self)
    }
}

/// RX and TX configuration getters and setters.
impl Usrp {
    /// Read current settings for the given Rx channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Result, Usrp};
    ///
    /// let usrp = Usrp::open_any().expect("failed to open USRP");
    /// usrp.mboard(0)
    ///     .set_rx_subdev_str("A:0")
    ///     .expect("failed to set subdev spec");
    ///
    /// let ants = usrp.rx_config(0)
    ///     .antennas()
    ///     .expect("failed to get antennas");
    /// println!("possible RX antennas: {ants:?}");
    /// ```
    #[must_use]
    pub fn rx_config<'a>(&'a self, channel: usize) -> ChannelConfiguration<'a, { RX_DIR }> {
        ChannelConfiguration::<'a, RX_DIR>::new(self, channel)
    }

    /// Read current settings for the given Tx channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Result, Usrp};
    ///
    /// let usrp = Usrp::open_any().expect("failed to open USRP");
    /// usrp.mboard(0)
    ///     .set_rx_subdev_str("A:0")
    ///     .expect("failed to set subdev spec");
    ///
    /// let ants = usrp.tx_config(0)
    ///     .antennas()
    ///     .expect("failed to get antennas");
    /// println!("possible TX antennas: {ants:?}");
    /// ```
    #[must_use]
    pub fn tx_config<'a>(&'a self, channel: usize) -> ChannelConfiguration<'a, { TX_DIR }> {
        ChannelConfiguration::<'a, TX_DIR>::new(self, channel)
    }

    /// Write settings for the given RX channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Result, Usrp};
    ///
    /// let mut usrp = Usrp::open_any().expect("failed to open USRP");
    /// usrp.mboard(0)
    ///     .set_rx_subdev_str("A:0")
    ///     .expect("failed to set subdev spec");
    ///
    /// let ants = usrp.set_rx_config(0)
    ///     .set_antenna("RX2")
    ///     .expect("failed to set antenna");
    /// ```
    #[must_use]
    pub fn set_rx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { RX_DIR }> {
        ChannelConfigurationBuilder::<'a, RX_DIR>::new(self, channel)
    }

    /// Write settings for the given TX channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uhd_usrp::{DeviceArgs, Result, Usrp};
    ///
    /// let mut usrp = Usrp::open_any().expect("failed to open USRP");
    /// usrp.mboard(0)
    ///     .set_tx_subdev_str("A:0")
    ///     .expect("failed to set subdev spec");
    ///
    /// let ants = usrp.set_tx_config(0)
    ///     .set_antenna("TX/RX")
    ///     .expect("failed to set antenna");
    /// ```
    #[must_use]
    pub fn set_tx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { TX_DIR }> {
        ChannelConfigurationBuilder::<'a, TX_DIR>::new(self, channel)
    }
}
