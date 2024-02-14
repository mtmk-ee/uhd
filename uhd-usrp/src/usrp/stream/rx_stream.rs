use std::{
    cell::Cell,
    marker::PhantomData,
    ptr::{addr_of, addr_of_mut},
    time::Duration,
};

use crate::{
    buffer::SampleBuffer,
    error::try_uhd,
    usrp::{metadata::RxMetadata, Usrp},
    DeviceTime, Result, Sample, UhdError,
};

use super::stream_args::StreamArgs;

pub struct RxStream<T: Sample> {
    handle: uhd_usrp_sys::uhd_rx_streamer_handle,
    samples_per_buffer: usize,
    channels: usize,
    _unsync: PhantomData<Cell<T>>,
    _ugh: PhantomData<T>,
}

impl<T: Sample> RxStream<T> {
    pub(crate) fn open(usrp: &Usrp, args: StreamArgs<T>) -> Result<Self> {
        let mut handle: uhd_usrp_sys::uhd_rx_streamer_handle = std::ptr::null_mut();
        let args = args.leak();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_rx_streamer_make(&mut handle) })?;
        let res = try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_stream(
                usrp.handle().as_mut_ptr(),
                args.inner() as *const _ as *mut _,
                handle,
            )
            .into()
        });
        match res {
            Ok(()) => Self::with_handle(handle),
            Err(e) => {
                unsafe {
                    uhd_usrp_sys::uhd_rx_streamer_free(&mut handle);
                }
                Err(e)
            }
        }
    }

    pub(crate) fn with_handle(handle: uhd_usrp_sys::uhd_rx_streamer_handle) -> Result<Self> {
        let mut spb = 0;
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_max_num_samps(handle, addr_of_mut!(spb))
        })?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_num_channels(handle, addr_of_mut!(channels))
        })?;

        Ok(Self {
            handle,
            samples_per_buffer: spb,
            channels,
            _unsync: PhantomData::default(),
            _ugh: PhantomData::default(),
        })
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn max_samples_per_buffer(&self) -> usize {
        self.samples_per_buffer
    }

    pub fn reader<'a>(&'a mut self) -> RxStreamReaderOptions<'a, T> {
        RxStreamReaderOptions::new(self)
    }
}

pub struct RxStreamReaderOptions<'a, T: Sample> {
    stream: &'a RxStream<T>,
    at_time: Option<DeviceTime>,
    timeout: Option<Duration>,
    one_packet: bool,
    limit: Option<(usize, bool)>,
}

impl<'a, T: Sample> RxStreamReaderOptions<'a, T> {
    pub(crate) fn new(stream: &'a RxStream<T>) -> Self {
        Self {
            stream,
            at_time: None,
            timeout: None,
            one_packet: false,
            limit: None,
        }
    }

    pub fn at_time(mut self, delay: DeviceTime) -> Self {
        self.at_time = Some(delay);
        self
    }

    pub fn limit(mut self, n_samples: usize, and_done: bool) -> Self {
        self.limit = Some((n_samples, and_done));
        self
    }

    pub fn one_packet(mut self) -> Self {
        self.one_packet = true;
        self
    }

    pub fn open(self) -> Result<RxStreamReader<'a, T>> {
        let mut cmd = uhd_usrp_sys::uhd_stream_cmd_t {
            stream_mode: uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_START_CONTINUOUS,
            num_samps: 0,
            stream_now: self.at_time.is_none(),
            time_spec_full_secs: 0,
            time_spec_frac_secs: 0.0,
        };
        if let Some((n_samples, and_done)) = self.limit {
            cmd.num_samps = n_samples;
            cmd.stream_mode = if and_done {
                uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_NUM_SAMPS_AND_DONE
            } else {
                uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_NUM_SAMPS_AND_MORE
            };
        }
        if let Some(at_time) = self.at_time {
            cmd.time_spec_full_secs = at_time.full_seconds() as i64;
            cmd.time_spec_frac_secs = at_time.fractional_seconds();
        }

        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_issue_stream_cmd(self.stream.handle, addr_of!(cmd))
        })?;
        Ok(RxStreamReader {
            stream: self.stream,
            timeout: self.timeout,
            one_packet: self.one_packet,
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

unsafe impl<T: Sample + Send> Send for RxStream<T> {}

pub struct RxStreamReader<'a, T: Sample> {
    stream: &'a RxStream<T>,
    timeout: Option<Duration>,
    one_packet: bool,
}

impl<'a, T: Sample> RxStreamReader<'a, T> {
    pub fn recv(
        &mut self,
        buff: &mut impl SampleBuffer<T>,
        metadata: &mut RxMetadata,
    ) -> Result<usize> {
        if buff.channels() != self.stream.channels() {
            return Err(UhdError::Index);
        }
        unsafe { self.recv_unchecked(buff, metadata) }
    }

    pub fn recv_until<F: Fn(&mut B, &mut RxMetadata) -> bool, B: SampleBuffer<T>>(
        &mut self,
        buff: &mut B,
        metadata: &mut RxMetadata,
        predicate: F,
    ) -> Result<()> {
        loop {
            self.recv(buff, metadata)?;
            if !predicate(buff, metadata) {
                break;
            }
        }
        Ok(())
    }

    pub unsafe fn recv_unchecked(
        &mut self,
        buff: &mut impl SampleBuffer<T>,
        metadata: &mut RxMetadata,
    ) -> Result<usize> {
        self.recv_raw(buff.as_mut_ptr(), buff.samples(), metadata)
    }

    pub unsafe fn recv_raw(
        &mut self,
        buff: *mut *mut T,
        samples_per_channel: usize,
        metadata: &mut RxMetadata,
    ) -> Result<usize> {
        let mut received = 0;
        let handle = metadata.handle_mut();
        try_uhd!(uhd_usrp_sys::uhd_rx_streamer_recv(
            self.stream.handle,
            buff.cast(),
            samples_per_channel,
            handle.as_mut_mut_ptr(),
            self.timeout.unwrap_or(Duration::ZERO).as_secs_f64(),
            self.one_packet,
            addr_of_mut!(received),
        ))?;
        Ok(received)
    }
}

impl<'a, T: Sample> Drop for RxStreamReader<'a, T> {
    fn drop(&mut self) {
        let cmd = uhd_usrp_sys::uhd_stream_cmd_t {
            stream_mode: uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_STOP_CONTINUOUS,
            num_samps: 0,
            stream_now: false,
            time_spec_full_secs: 0,
            time_spec_frac_secs: 0.0,
        };
        unsafe {
            uhd_usrp_sys::uhd_rx_streamer_issue_stream_cmd(self.stream.handle, addr_of!(cmd));
        }
    }
}
