use std::{collections::HashMap, ffi::CString, marker::PhantomData};

use num::Complex;

use crate::Result;

use super::Usrp;

#[derive(Clone, Debug, Default)]
pub struct DeviceArgs {
    addr: Option<String>,
    serial: Option<String>,
    resource: Option<String>,
    name: Option<String>,
    type_: Option<String>,
    vid_pid: Option<(String, String)>,
}

impl DeviceArgs {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn addr(mut self, ip: &str) -> Self {
        self.addr = Some(ip.to_owned());
        self
    }

    pub fn serial(mut self, serial: &str) -> Self {
        self.serial = Some(serial.to_owned());
        self
    }

    pub fn resource(mut self, resource: &str) -> Self {
        self.resource = Some(resource.to_owned());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn type_(mut self, type_: &str) -> Self {
        self.type_ = Some(type_.to_owned());
        self
    }

    pub fn vid_pid(mut self, vid: &str, pid: &str) -> Self {
        self.vid_pid = Some((vid.to_owned(), pid.to_owned()));
        self
    }

    pub fn open(self) -> Result<Usrp> {
        Usrp::open(self)
    }

    fn iter(&self) -> impl Iterator<Item = String> + '_ {
        let mut args = vec![];
        if let Some(addr) = &self.addr {
            args.push(format!("args={addr}"));
        }
        if let Some(serial) = &self.serial {
            args.push(format!("serial={serial}"));
        }
        if let Some(resource) = &self.resource {
            args.push(format!("resource={resource}"));
        }
        if let Some(name) = &self.name {
            args.push(format!("name={name}"));
        }
        if let Some(type_) = &self.type_ {
            args.push(format!("type={type_}"));
        }
        if let Some((vid, pid)) = &self.vid_pid {
            args.push(format!("vid={vid}"));
            args.push(format!("pid={pid}"));
        }
        args.into_iter()
    }
}

impl ToString for DeviceArgs {
    fn to_string(&self) -> String {
        self.iter().collect::<Vec<String>>().join(",")
    }
}

pub trait SampleType {
    fn name() -> &'static str;
}
impl SampleType for Complex<f32> {
    fn name() -> &'static str {
        "fc32"
    }
}
impl SampleType for Complex<f64> {
    fn name() -> &'static str {
        "fc64"
    }
}
impl SampleType for Complex<i8> {
    fn name() -> &'static str {
        "sc8"
    }
}
impl SampleType for Complex<i16> {
    fn name() -> &'static str {
        "sc16"
    }
}

#[derive(Clone, Debug)]
pub struct StreamArgs<T: SampleType> {
    pub(crate) otw_format: Option<OtwFormat>,
    pub(crate) args: HashMap<&'static str, String>,
    pub(crate) channels: Option<Vec<usize>>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: SampleType> StreamArgs<T> {
    pub fn new() -> Self {
        Self {
            otw_format: Default::default(),
            args: Default::default(),
            channels: Default::default(),
            phantom: Default::default(),
        }
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

    pub fn otw_format(&mut self, format: OtwFormat) -> &mut Self {
        self.otw_format = Some(format);
        self
    }

    pub fn fullscale(&mut self, amplitude: f32) -> &mut Self {
        self.args.insert("fullscale", amplitude.to_string());
        self
    }

    pub fn peak(&mut self, amplitude: f32) -> &mut Self {
        self.args.insert("peak", amplitude.to_string());
        self
    }

    pub fn underflow_policy(&mut self, policy: UnderflowPolicy) -> &mut Self {
        self.args
            .insert("underflow_policy", policy.as_str().to_owned());
        self
    }

    pub fn spp(&mut self, samples_per_packet: usize) -> &mut Self {
        self.args.insert("spp", samples_per_packet.to_string());
        self
    }

    pub fn channels(&mut self, channels: &[usize]) -> &mut Self {
        self.channels = Some(channels.to_vec());
        self
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
