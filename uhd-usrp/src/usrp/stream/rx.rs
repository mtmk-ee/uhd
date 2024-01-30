use std::{
    marker::PhantomData,
    ptr::{addr_of, addr_of_mut},
    time::Duration,
};

use crate::{
    error::try_uhd,
    usrp::{
        args::{SampleType, StreamArgs},
        metadata::RxMetadata,
        Usrp,
    },
    util::PhantomUnsync,
    Result, UhdError,
};

pub struct RxStream<T: SampleType> {
    handle: uhd_usrp_sys::uhd_rx_streamer_handle,
    samples_per_buffer: usize,
    channels: usize,
    _unsync: PhantomUnsync,
    _ugh: PhantomData<T>,
}

impl<T: SampleType> RxStream<T> {
    pub(crate) fn open(usrp: &Usrp, args: StreamArgs<T>) -> Result<Self> {
        let mut handle: uhd_usrp_sys::uhd_rx_streamer_handle = std::ptr::null_mut();
        let args = args.into_sys_guard();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_rx_streamer_make(&mut handle) })?;
        let res = try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_stream(
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

    pub fn max_samples_per_buffer(&self) -> usize {
        self.samples_per_buffer
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn reader<'a>(&'a mut self) -> RxStreamReaderOptions<'a, T> {
        RxStreamReaderOptions::new(self)
    }
}

pub struct RxStreamReaderOptions<'a, T: SampleType> {
    stream: &'a RxStream<T>,
    wait_until: Option<Duration>,
    timeout: Option<Duration>,
    one_packet: bool,
    limit: Option<(usize, bool)>,
}

impl<'a, T: SampleType> RxStreamReaderOptions<'a, T> {
    pub(crate) fn new(stream: &'a RxStream<T>) -> Self {
        Self {
            stream,
            wait_until: None,
            timeout: None,
            one_packet: false,
            limit: None,
        }
    }

    pub fn limit(mut self, n_samples: usize, and_done: bool) -> Self {
        self.limit = Some((n_samples, and_done));
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.wait_until = Some(delay);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
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
            stream_now: self.wait_until.is_none(),
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
        if let Some(delay) = self.wait_until {
            cmd.time_spec_full_secs = delay.as_secs() as i64;
            cmd.time_spec_frac_secs = delay.as_secs_f64() - delay.as_secs() as f64;
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
}

pub struct RxStreamReader<'a, T: SampleType> {
    stream: &'a RxStream<T>,
    timeout: Option<Duration>,
    one_packet: bool,
}

impl<'a, T: SampleType> RxStreamReader<'a, T> {
    pub fn recv(&mut self, buffs: &[&mut [T]], metadata: &mut RxMetadata) -> Result<usize> {
        if buffs.len() > 1 && buffs.iter().any(|e| e.len() != buffs[0].len()) {
            return Err(UhdError::Index);
        }
        let mut ptr_buffs = buffs
            .iter()
            .map(|buff| buff.as_ptr().cast_mut())
            .collect::<Vec<_>>();
        let mut received = 0;
        let mut handle = metadata.handle();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_recv(
                self.stream.handle,
                ptr_buffs.as_mut_ptr().cast(),
                buffs[0].len(),
                addr_of_mut!(handle),
                self.timeout.unwrap_or(Duration::ZERO).as_secs_f64(),
                self.one_packet,
                addr_of_mut!(received),
            )
        })?;
        Ok(received)
    }
}

impl<'a, T: SampleType> Drop for RxStreamReader<'a, T> {
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
