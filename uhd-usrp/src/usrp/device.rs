use std::{ffi::CString, marker::PhantomData, ptr::addr_of_mut};

use crate::{error::try_uhd, stream::StreamArgs, DeviceTime, Result, Sample};

use super::{
    channel::{ChannelConfiguration, ChannelConfigurationBuilder, RX_DIR, TX_DIR},
    stream::{RxStream, TxStream},
    DeviceArgs,
};

pub struct Usrp {
    handle: uhd_usrp_sys::uhd_usrp_handle,
    _unsync: PhantomData<std::cell::Cell<()>>,
}

impl Usrp {
    pub fn open(args: DeviceArgs) -> Result<Self> {
        Self::open_with_str(&args.to_string())
    }

    pub fn open_any() -> Result<Self> {
        Self::open_with_str("")
    }

    pub fn open_with_str(args: &str) -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        let args = CString::new(args).unwrap();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_usrp_make(addr_of_mut!(handle), args.as_ptr()) })?;
        Ok(Self {
            handle,
            _unsync: PhantomData::default(),
        })
    }

    pub(crate) fn handle(&self) -> uhd_usrp_sys::uhd_usrp_handle {
        self.handle
    }

    pub fn rx_config<'a>(&'a mut self, channel: usize) -> ChannelConfiguration<'a, { RX_DIR }> {
        ChannelConfiguration::<'a, RX_DIR>::new(self, channel)
    }

    pub fn set_rx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { RX_DIR }> {
        ChannelConfigurationBuilder::<'a, RX_DIR>::new(self, channel)
    }

    pub fn tx_config<'a>(&'a mut self, channel: usize) -> ChannelConfiguration<'a, { TX_DIR }> {
        ChannelConfiguration::<'a, TX_DIR>::new(self, channel)
    }

    pub fn set_tx_config<'a>(
        &'a mut self,
        channel: usize,
    ) -> ChannelConfigurationBuilder<'a, { TX_DIR }> {
        ChannelConfigurationBuilder::<'a, TX_DIR>::new(self, channel)
    }

    pub fn rx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_num_channels(self.handle, addr_of_mut!(channels))
        })?;
        Ok(channels)
    }

    pub fn tx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_num_channels(self.handle, addr_of_mut!(channels))
        })?;
        Ok(channels)
    }

    pub fn tx_stream<T: Sample>(&self, args: StreamArgs<T>) -> Result<TxStream<T>> {
        TxStream::open(self, args)
    }

    pub fn rx_stream<T: Sample>(&self, args: StreamArgs<T>) -> Result<RxStream<T>> {
        RxStream::open(self, args)
    }

    pub fn time(&self) -> Result<DeviceTime> {
        let mut full_secs = 0;
        let mut frac_secs = 0.0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_time_now(
                self.handle,
                0,
                addr_of_mut!(full_secs),
                addr_of_mut!(frac_secs),
            )
        })?;
        Ok(DeviceTime::from_parts(full_secs as u64, frac_secs))
    }

    pub fn set_time(&self, time: DeviceTime) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_now(
                self.handle,
                time.full_seconds() as i64,
                time.fractional_seconds(),
                0,
            )
        })?;
        Ok(())
    }
}

impl Drop for Usrp {
    fn drop(&mut self) {
        unsafe {
            uhd_usrp_sys::uhd_usrp_free(addr_of_mut!(self.handle));
        }
    }
}
