use std::{ffi::CString, marker::PhantomData, ptr::addr_of_mut};

use crate::{error::try_uhd, ffi::OwnedHandle, stream::StreamArgs, DeviceTime, Result, Sample};

use super::{
    channel::{ChannelConfiguration, ChannelConfigurationBuilder, RX_DIR, TX_DIR},
    mboard::Motherboard,
    stream::{RxStream, TxStream},
    DeviceArgs,
};

/// The entry point for interacting with a connected USRP.
pub struct Usrp {
    handle: OwnedHandle<uhd_usrp_sys::uhd_usrp>,
    _unsync: PhantomData<std::cell::Cell<()>>,
}

impl Usrp {
    /// Attempts to open a USRP using the given [`DeviceArgs`]
    pub fn open(args: DeviceArgs) -> Result<Self> {
        Self::open_with_str(&args.to_string())
    }

    /// Open any connected USRP.
    /// 
    /// The behavior of this function is not guaranteed to be consistent if multiple USRPs are connected.
    pub fn open_any() -> Result<Self> {
        Self::open_with_str("")
    }

    /// Open a USRP using `"key=value"` style arguments.
    pub fn open_with_str(args: &str) -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        let args = CString::new(args).unwrap();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_usrp_make(addr_of_mut!(handle), args.as_ptr()) })?;
        Ok(Self {
            handle: unsafe { OwnedHandle::from_ptr(handle, uhd_usrp_sys::uhd_usrp_free) },
            _unsync: PhantomData::default(),
        })
    }

    pub(crate) fn handle(&self) -> &OwnedHandle<uhd_usrp_sys::uhd_usrp> {
        &self.handle
    }

    /// Access per-motherboard properties.
    pub fn mboard(&self, mboard: usize) -> Motherboard {
        Motherboard::new(self, mboard)
    }

    pub fn rx_config<'a>(&'a mut self, channel: usize) -> ChannelConfiguration<'a, { RX_DIR }> {
        ChannelConfiguration::<'a, RX_DIR>::new(self, channel)
    }

    pub fn rx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_num_channels(self.handle.as_mut_ptr(), addr_of_mut!(channels))
        })?;
        Ok(channels)
    }

    pub fn rx_stream<T: Sample>(&self, args: StreamArgs<T>) -> Result<RxStream<T>> {
        RxStream::open(self, args)
    }

    pub fn set_time_unknown_pps(&mut self, time: DeviceTime) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_unknown_pps(
                self.handle.as_mut_ptr(),
                time.full_seconds() as i64,
                time.fractional_seconds(),
            )
        })?;
        Ok(())
    }

    pub fn set_rx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { RX_DIR }> {
        ChannelConfigurationBuilder::<'a, RX_DIR>::new(self, channel)
    }

    pub fn set_tx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { TX_DIR }> {
        ChannelConfigurationBuilder::<'a, TX_DIR>::new(self, channel)
    }

    pub fn tx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_num_channels(self.handle.as_mut_ptr(), addr_of_mut!(channels))
        })?;
        Ok(channels)
    }

    pub fn tx_config<'a>(&'a mut self, channel: usize) -> ChannelConfiguration<'a, { TX_DIR }> {
        ChannelConfiguration::<'a, TX_DIR>::new(self, channel)
    }

    pub fn tx_stream<T: Sample>(&self, args: StreamArgs<T>) -> Result<TxStream<T>> {
        TxStream::open(self, args)
    }
}
