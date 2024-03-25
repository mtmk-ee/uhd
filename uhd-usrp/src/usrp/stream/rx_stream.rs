use std::{
    cell::Cell,
    collections::HashMap,
    ffi::CString,
    marker::PhantomData,
    ptr::{addr_of, addr_of_mut},
    time::Duration,
};

use super::OtwFormat;
use crate::{
    buffer::SampleBuffer,
    error::try_uhd,
    ffi::OwnedHandle,
    usrp::{metadata::RxMetadata, Usrp},
    Result, Sample, TimeSpec, UhdError,
};

pub(crate) type RxStreamHandle = OwnedHandle<uhd_usrp_sys::uhd_rx_streamer>;

pub struct RxStreamBuilder<'usrp, T>
where
    T: Sample,
{
    usrp: &'usrp Usrp,
    otw_format: Option<OtwFormat>,
    args: HashMap<String, String>,
    channels: Option<Vec<usize>>,
    _phantom: PhantomData<T>,
}

impl<'usrp, T> RxStreamBuilder<'usrp, T>
where
    T: Sample,
{
    pub(crate) fn new(usrp: &'usrp Usrp) -> Self {
        Self {
            usrp,
            otw_format: None,
            args: HashMap::new(),
            channels: None,
            _phantom: PhantomData::default(),
        }
    }

    /// Specify the "over the wire" format to use.
    ///
    /// If unspecified, a format will be chosen automatically.
    pub fn with_otw_format(&mut self, format: OtwFormat) -> &mut Self {
        self.otw_format = Some(format);
        self
    }

    /// Specify which channels will be used for transmission.
    ///
    /// Defaults to a single channel, `0`.
    pub fn with_channels(&mut self, channels: &[usize]) -> &mut Self {
        // TODO: what happens with duplicate channel numbers?
        self.channels = Some(channels.to_vec());
        self
    }

    /// Specify a keyword argument for the stream.
    ///
    /// This can be used to set other less common arguments.
    /// The arguments are sent to UHD in the form `"arg=value"`.
    ///
    /// # Panics
    ///
    /// This will panic if either `arg` or `value` contains an `'='` character
    /// or null byte.
    pub fn with_kwarg(&mut self, arg: &str, value: &str) -> &mut Self {
        assert!(!arg.contains('='), "argument cannot contain '='");
        assert!(!value.contains('='), "value cannot contain '='");
        assert!(!arg.contains('\0'), "argument cannot contain null bytes");
        assert!(!value.contains('\0'), "value cannot contain null bytes");

        self.args.insert(arg.to_string(), value.to_string());
        self
    }

    /// Open the RX stream using the previously-specified arguments.
    #[must_use]
    pub fn open(&self) -> Result<RxStream<T>> {
        let mut handle: uhd_usrp_sys::uhd_rx_streamer_handle = std::ptr::null_mut();
        if let Err(e) = try_uhd!(unsafe { uhd_usrp_sys::uhd_rx_streamer_make(&mut handle) }) {
            unsafe { uhd_usrp_sys::uhd_rx_streamer_free(addr_of_mut!(handle)) };
            return Err(e);
        }

        let cpu_format = CString::new(T::name()).unwrap();
        let otw_format = self.otw_format.map(|f| f.as_str()).unwrap_or("");
        let args = CString::new(
            self.args
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(","),
        )
        .unwrap();
        let mut channels = self.channels.clone().unwrap_or_else(Vec::new);
        let mut stream_args = uhd_usrp_sys::uhd_stream_args_t {
            cpu_format: cpu_format.as_ptr() as *mut _,
            otw_format: otw_format.as_ptr() as *mut _,
            args: args.as_ptr() as *mut _,
            channel_list: channels.as_mut_ptr(),
            n_channels: channels.len() as i32,
        };

        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_rx_stream(
                self.usrp.handle().as_mut_ptr(),
                addr_of_mut!(stream_args),
                handle,
            )
        })?;
        RxStream::<T>::new(unsafe {
            OwnedHandle::from_ptr(handle, uhd_usrp_sys::uhd_rx_streamer_free)
        })
    }
}

pub struct RxStream<T>
where
    T: Sample,
{
    handle: RxStreamHandle,
    samples_per_buffer: usize,
    channels: usize,

    _unsync: PhantomData<Cell<T>>,
}

unsafe impl<T> Send for RxStream<T> where T: Sample {}

impl<T> RxStream<T>
where
    T: Sample,
{
    pub(crate) fn new(handle: RxStreamHandle) -> Result<Self> {
        let mut spb = 0;
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_max_num_samps(handle.as_mut_ptr(), addr_of_mut!(spb))
        })?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_num_channels(handle.as_mut_ptr(), addr_of_mut!(channels))
        })?;

        Ok(Self {
            handle,
            samples_per_buffer: spb,
            channels,
            _unsync: PhantomData::default(),
        })
    }

    pub(crate) fn handle(&self) -> &RxStreamHandle {
        &self.handle
    }

    pub fn max_samples_per_channel(&self) -> usize {
        self.samples_per_buffer
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    #[must_use = "commands must be sent to start the stream"]
    pub fn start_command(&self) -> RxStartCommand<T> {
        RxStartCommand::new(self)
    }

    pub fn stop_now(&self) -> Result<()> {
        let cmd = uhd_usrp_sys::uhd_stream_cmd_t {
            stream_mode: uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_STOP_CONTINUOUS,
            ..Default::default()
        };
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_issue_stream_cmd(self.handle.as_mut_ptr(), addr_of!(cmd))
        })?;
        Ok(())
    }

    pub fn reader(&mut self) -> RxStreamReader<T> {
        RxStreamReader::new(self)
    }
}

pub struct RxStartCommand<'stream, T>
where
    T: Sample,
{
    stream: &'stream RxStream<T>,
    at_time: TimeSpec,
    limit: Option<(usize, bool)>,
}

impl<'stream, T> RxStartCommand<'stream, T>
where
    T: Sample,
{
    pub(crate) fn new(stream: &'stream RxStream<T>) -> Self {
        Self {
            stream,
            at_time: TimeSpec::ZERO,
            limit: None,
        }
    }

    pub fn with_time(&mut self, at_time: TimeSpec) -> &mut Self {
        self.at_time = at_time;
        self
    }

    pub fn with_limit(&mut self, limit: usize, and_done: bool) -> &mut Self {
        self.limit = Some((limit, and_done));
        self
    }

    pub fn send(&self) -> Result<()> {
        let cmd = uhd_usrp_sys::uhd_stream_cmd_t {
            stream_mode: self.stream_mode(),
            num_samps: self.n_samples(),
            stream_now: !self.at_time.is_zero(),
            time_spec_full_secs: self.at_time.full_secs(),
            time_spec_frac_secs: self.at_time.frac_secs(),
        };
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_rx_streamer_issue_stream_cmd(
                self.stream.handle().as_mut_ptr(),
                addr_of!(cmd),
            )
        })?;
        Ok(())
    }

    fn n_samples(&self) -> usize {
        match self.limit {
            Some((n_samples, _)) => n_samples,
            None => usize::MAX,
        }
    }

    fn stream_mode(&self) -> uhd_usrp_sys::uhd_stream_mode_t::Type {
        match self.limit {
            Some((_, true)) => uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_NUM_SAMPS_AND_DONE,
            Some((_, false)) => uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_NUM_SAMPS_AND_MORE,
            None => uhd_usrp_sys::uhd_stream_mode_t::UHD_STREAM_MODE_START_CONTINUOUS,
        }
    }
}

pub struct RxStreamReader<'stream, 'md, T>
where
    T: Sample,
{
    stream: &'stream mut RxStream<T>,
    timeout: Option<Duration>,
    one_packet: bool,
    metadata: Option<&'md mut RxMetadata>,
}

impl<'stream, 'md, T> RxStreamReader<'stream, 'md, T>
where
    T: Sample,
{
    pub(crate) fn new(stream: &'stream mut RxStream<T>) -> Self {
        Self {
            stream,
            timeout: None,
            one_packet: false,
            metadata: None,
        }
    }

    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_one_packet(&mut self, one_packet: bool) -> &mut Self {
        self.one_packet = one_packet;
        self
    }

    pub fn with_metadata_output(&mut self, metadata: &'md mut RxMetadata) -> &mut Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn recv(&mut self, buff: &mut impl SampleBuffer<T>) -> Result<usize> {
        if buff.channels() != self.stream.channels()
            || buff.samples() != self.stream.samples_per_buffer
        {
            return Err(UhdError::Index);
        }
        unsafe { self.recv_unchecked(buff) }
    }

    pub fn recv_until<F, B>(&mut self, buff: &mut B, predicate: F) -> Result<()>
    where
        F: Fn(&mut B, Option<&RxMetadata>) -> bool,
        B: SampleBuffer<T>,
    {
        loop {
            self.recv(buff)?;
            if !predicate(buff, self.metadata.as_deref()) {
                break;
            }
        }
        Ok(())
    }

    pub unsafe fn recv_unchecked(&mut self, buff: &mut impl SampleBuffer<T>) -> Result<usize> {
        self.recv_raw(buff.as_mut_ptr(), buff.samples())
    }

    pub unsafe fn recv_raw(
        &mut self,
        buff: *mut *mut T,
        samples_per_channel: usize,
    ) -> Result<usize> {
        let mut received = 0;
        let metadata_handle = self
            .metadata
            .as_ref()
            .map(|md| md.handle().as_mut_mut_ptr())
            .unwrap_or(std::ptr::null_mut());
        try_uhd!(uhd_usrp_sys::uhd_rx_streamer_recv(
            self.stream.handle().as_mut_ptr(),
            buff.cast(),
            samples_per_channel,
            metadata_handle,
            self.timeout.unwrap_or(Duration::ZERO).as_secs_f64(),
            self.one_packet,
            addr_of_mut!(received),
        ))?;
        Ok(received)
    }
}
