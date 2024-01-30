use std::{marker::PhantomData, ptr::addr_of_mut, time::Duration};

use crate::{
    error::try_uhd,
    usrp::{
        args::{SampleType, StreamArgs},
        metadata::{TxMetadata, TxMetadataHandle},
        Usrp,
    },
    util::PhantomUnsync,
    Result, UhdError,
};

pub struct TxStream<T: SampleType> {
    handle: uhd_usrp_sys::uhd_tx_streamer_handle,
    samples_per_buffer: usize,
    channels: usize,
    _unsync: PhantomUnsync,
    _ugh: PhantomData<T>,
}

impl<T: SampleType> TxStream<T> {
    pub(crate) fn open(usrp: &Usrp, args: StreamArgs<T>) -> Result<Self> {
        let mut handle: uhd_usrp_sys::uhd_tx_streamer_handle = std::ptr::null_mut();
        let args = args.into_sys_guard();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_tx_streamer_make(&mut handle) })?;
        let res = try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_stream(
                usrp.handle(),
                args.inner() as *const _ as *mut _,
                handle,
            )
            .into()
        });
        match res {
            Ok(()) => Self::with_handle(handle),
            Err(e) => {
                unsafe {
                    uhd_usrp_sys::uhd_tx_streamer_free(&mut handle);
                }
                Err(e)
            }
        }
    }

    pub(crate) fn with_handle(handle: uhd_usrp_sys::uhd_tx_streamer_handle) -> Result<Self> {
        let mut spb = 0;
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_max_num_samps(handle, addr_of_mut!(spb))
        })?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_num_channels(handle, addr_of_mut!(channels))
        })?;

        Ok(Self {
            handle,
            samples_per_buffer: spb,
            channels,
            _unsync: PhantomData::default(),
            _ugh: PhantomData::default(),
        })
    }

    pub fn max_samples_per_buffer(&self) -> usize {
        self.samples_per_buffer
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn send(
        &mut self,
        data: &[&[T]],
        metadata: &TxMetadata,
        timeout: Duration,
    ) -> Result<usize> {
        if data.len() > 1 && data.iter().any(|e| e.len() != data[0].len()) {
            return Err(UhdError::Index);
        }
        let mut buff = data.iter().map(|buff| buff.as_ptr()).collect::<Vec<_>>();
        let mut sent = 0;
        let metadata = TxMetadataHandle::from_metadata(metadata)?;
        let mut metadata_handle = metadata.handle();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_send(
                self.handle,
                buff.as_mut_ptr().cast(),
                data[0].len(),
                addr_of_mut!(metadata_handle),
                timeout.as_secs_f64(),
                addr_of_mut!(sent),
            )
        })?;
        Ok(sent)
    }

    pub fn send_one_channel(
        &mut self,
        data: &[T],
        metadata: &TxMetadata,
        timeout: Duration,
    ) -> Result<usize> {
        self.send(&[data], metadata, timeout)
    }
}

impl<T: SampleType> Drop for TxStream<T> {
    fn drop(&mut self) {
        unsafe {
            uhd_usrp_sys::uhd_tx_streamer_free(addr_of_mut!(self.handle));
        }
    }
}
