use std::{ffi::CString, marker::PhantomData, ptr::addr_of_mut, time::Duration};

use crate::{error::try_uhd, util::PhantomUnsync, Result};

use super::{
    args::{DeviceArgs, SampleType, StreamArgs},
    configuration::{RxChannelConfig, SetRxChannelConfig},
    stream::{RxStream, TxStream},
};

pub struct Usrp {
    handle: uhd_usrp_sys::uhd_usrp_handle,
    _unsync: PhantomUnsync,
}

impl Usrp {
    pub fn open(args: DeviceArgs) -> Result<Self> {
        let mut handle = std::ptr::null_mut();
        let args = CString::new(args.to_string()).unwrap();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_usrp_make(addr_of_mut!(handle), args.as_ptr()) })?;
        Ok(Self {
            handle,
            _unsync: PhantomData::default(),
        })
    }

    pub(crate) fn handle(&self) -> uhd_usrp_sys::uhd_usrp_handle {
        self.handle
    }

    pub fn rx_config<'a>(&'a mut self, channel: usize) -> RxChannelConfig<'a> {
        RxChannelConfig::new(self, channel)
    }

    pub fn set_rx_config<'a>(&'a mut self, channel: usize) -> SetRxChannelConfig<'a> {
        SetRxChannelConfig::new(self, channel)
    }

    pub fn rx_channels(&self) -> Result<usize> {
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_num_channels(self.handle, addr_of_mut!(channels))
        })?;
        Ok(channels)
    }

    pub fn tx_stream<T: SampleType>(&self, args: StreamArgs<T>) -> Result<TxStream<T>> {
        TxStream::open(self, args)
    }

    pub fn rx_stream<T: SampleType>(&self, args: StreamArgs<T>) -> Result<RxStream<T>> {
        RxStream::open(self, args)
    }

    pub fn set_time(&self, offset: Duration) -> Result<()> {
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_set_time_now(
                self.handle,
                offset.as_secs() as i64,
                offset.as_secs_f64() - offset.as_secs() as f64,
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
