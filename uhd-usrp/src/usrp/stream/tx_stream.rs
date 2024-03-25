use std::{
    cell::Cell, collections::HashMap, ffi::CString, marker::PhantomData, ptr::addr_of_mut,
    time::Duration,
};

use super::OtwFormat;
use crate::{
    error::try_uhd,
    ffi::OwnedHandle,
    usrp::{metadata::TxMetadata, Usrp},
    Result, Sample, SampleBuffer,
};

/// An owned handle for a USRP TX stream.
pub(crate) type TxStreamHandle = OwnedHandle<uhd_usrp_sys::uhd_tx_streamer>;

/// Arguments for creating a TX stream.
pub struct TxStreamBuilder<'usrp, T>
where
    T: Sample,
{
    usrp: &'usrp Usrp,
    otw_format: Option<OtwFormat>,
    args: HashMap<String, String>,
    channels: Vec<usize>,
    _phantom: PhantomData<T>,
}

impl<'usrp, T> TxStreamBuilder<'usrp, T>
where
    T: Sample,
{
    pub(crate) fn new(usrp: &'usrp Usrp) -> Self {
        Self {
            usrp,
            otw_format: None,
            args: HashMap::new(),
            channels: vec![0],
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
        self.channels = channels.to_vec();
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

    /// Open the TX stream using the previously-specified arguments.
    #[must_use]
    pub fn open(&self) -> Result<TxStream<T>> {
        let mut handle: uhd_usrp_sys::uhd_tx_streamer_handle = std::ptr::null_mut();
        if let Err(e) = try_uhd!(unsafe { uhd_usrp_sys::uhd_tx_streamer_make(&mut handle) }) {
            unsafe { uhd_usrp_sys::uhd_tx_streamer_free(addr_of_mut!(handle)) };
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
        let mut stream_args = uhd_usrp_sys::uhd_stream_args_t {
            cpu_format: cpu_format.as_ptr() as *mut _,
            otw_format: otw_format.as_ptr() as *mut _,
            args: args.as_ptr() as *mut _,
            channel_list: self.channels.as_ptr().cast_mut(),
            n_channels: self.channels.len() as i32,
        };

        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_usrp_get_tx_stream(
                self.usrp.handle().as_mut_ptr(),
                addr_of_mut!(stream_args),
                handle,
            )
        })?;
        TxStream::<T>::new(unsafe {
            OwnedHandle::from_ptr(handle, uhd_usrp_sys::uhd_tx_streamer_free)
        })
    }
}

pub struct TxStream<T: Sample> {
    handle: TxStreamHandle,
    samples_per_buffer: usize,
    channels: usize,

    _unsync: PhantomData<Cell<T>>,
}

impl<T: Sample> TxStream<T> {
    pub(crate) fn new(handle: TxStreamHandle) -> Result<Self> {
        let mut spb = 0;
        let mut channels = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_max_num_samps(handle.as_mut_ptr(), addr_of_mut!(spb))
        })?;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_num_channels(handle.as_mut_ptr(), addr_of_mut!(channels))
        })?;

        Ok(Self {
            handle,
            samples_per_buffer: spb,
            channels,
            _unsync: PhantomData::default(),
        })
    }

    pub(crate) fn handle(&self) -> &TxStreamHandle {
        &self.handle
    }

    pub fn max_samples_per_channel(&self) -> usize {
        self.samples_per_buffer
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn writer(&mut self) -> TxStreamWriter<T> {
        TxStreamWriter::new(self)
    }
}

unsafe impl<T: Sample + Send> Send for TxStream<T> {}

pub struct TxStreamWriter<'stream, 'md, T>
where
    T: Sample,
{
    stream: &'stream mut TxStream<T>,
    timeout: Option<Duration>,
    one_packet: bool,
    metadata: Option<&'md mut TxMetadata>,
}

impl<'stream, 'md, T> TxStreamWriter<'stream, 'md, T>
where
    T: Sample,
{
    pub fn new(stream: &'stream mut TxStream<T>) -> Self {
        Self {
            stream,
            timeout: None,
            one_packet: false,
            metadata: None,
        }
    }

    pub fn with_one_packet(&mut self, one_packet: bool) -> &mut Self {
        self.one_packet = one_packet;
        self
    }

    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_metadata(&mut self, metadata: &'md mut TxMetadata) -> &mut Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn send<B>(&mut self, buff: &B) -> Result<usize>
    where
        B: SampleBuffer<T>,
    {
        unsafe { self.send_raw(buff.as_ptr(), buff.samples()) }
    }

    pub unsafe fn send_raw(
        &mut self,
        buff: *const *const T,
        samples_per_channel: usize,
    ) -> Result<usize> {
        let mut sent = 0;
        let metadata_handle = self
            .metadata
            .as_ref()
            .map(|md| md.to_handle())
            .unwrap_or_else(|| TxMetadata::new().to_handle());
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_tx_streamer_send(
                self.stream.handle().as_mut_ptr(),
                buff.cast_mut().cast(),
                samples_per_channel,
                metadata_handle.as_mut_mut_ptr(),
                self.timeout.unwrap_or_default().as_secs_f64(),
                addr_of_mut!(sent),
            )
        })?;
        Ok(sent)
    }
}
