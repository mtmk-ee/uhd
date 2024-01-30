use std::{collections::HashMap, ffi::CString, marker::PhantomData};

use crate::Sample;


#[derive(Clone, Debug)]
pub struct StreamArgs<T: Sample> {
    otw_format: Option<OtwFormat>,
    args: HashMap<&'static str, String>,
    channels: Option<Vec<usize>>,
    phantom: PhantomData<T>,
}

impl<T: Sample> StreamArgs<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn into_sys_guard(self) -> StreamArgsSysGuard {
        let cpu_format = CString::new(T::name()).unwrap();
        let otw_format = self.otw_format.map(|f| f.as_str()).unwrap_or("");
        let args = self
            .args
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(",");
        let mut channels = self.channels.unwrap_or_else(Vec::new);
        let n_channels = channels.len();
        channels.shrink_to_fit();

        StreamArgsSysGuard {
            inner: uhd_usrp_sys::uhd_stream_args_t {
                cpu_format: CString::new(cpu_format).unwrap().into_raw(),
                otw_format: CString::new(otw_format).unwrap().into_raw(),
                args: CString::new(args).unwrap().into_raw(),
                channel_list: channels.leak().as_mut_ptr(),
                n_channels: n_channels as i32,
            },
        }
    }

    pub fn otw_format(mut self, format: OtwFormat) -> Self {
        self.otw_format = Some(format);
        self
    }

    pub fn fullscale(mut self, amplitude: f32) -> Self {
        self.args.insert("fullscale", amplitude.to_string());
        self
    }

    pub fn peak(mut self, amplitude: f32) -> Self {
        self.args.insert("peak", amplitude.to_string());
        self
    }

    pub fn underflow_policy(mut self, policy: UnderflowPolicy) -> Self {
        self.args
            .insert("underflow_policy", policy.as_str().to_owned());
        self
    }

    pub fn spp(mut self, samples_per_packet: usize) -> Self {
        self.args.insert("spp", samples_per_packet.to_string());
        self
    }

    pub fn channels(mut self, channels: &[usize]) -> Self {
        self.channels = Some(channels.to_vec());
        self
    }
}

impl<T: Sample> Default for StreamArgs<T> {
    fn default() -> Self {
        Self {
            otw_format: Default::default(),
            args: Default::default(),
            channels: Default::default(),
            phantom: Default::default(),
        }
    }
}

pub(crate) struct StreamArgsSysGuard {
    inner: uhd_usrp_sys::uhd_stream_args_t,
}

impl StreamArgsSysGuard {
    pub fn inner(&self) -> &uhd_usrp_sys::uhd_stream_args_t {
        &self.inner
    }
}

impl Drop for StreamArgsSysGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.inner.cpu_format);
            let _ = CString::from_raw(self.inner.otw_format);
            let _ = CString::from_raw(self.inner.args);
            Vec::<usize>::from_raw_parts(
                self.inner.channel_list,
                self.inner.n_channels as usize,
                self.inner.n_channels as usize,
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpuFormat {
    ComplexFloat64,
    ComplexFloat32,
    ComplexInt16,
    ComplexInt8,
}

impl CpuFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            CpuFormat::ComplexFloat64 => "fc64",
            CpuFormat::ComplexFloat32 => "fc32",
            CpuFormat::ComplexInt16 => "sc16",
            CpuFormat::ComplexInt8 => "sc8",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtwFormat {
    ComplexInt16,
    ComplexInt12,
    ComplexInt8,
}

impl OtwFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OtwFormat::ComplexInt16 => "sc16",
            OtwFormat::ComplexInt12 => "sc12",
            OtwFormat::ComplexInt8 => "sc8",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnderflowPolicy {
    NextBurst,
    NextPacket,
}

impl UnderflowPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnderflowPolicy::NextBurst => "next_burst",
            UnderflowPolicy::NextPacket => "next_packet",
        }
    }
}
