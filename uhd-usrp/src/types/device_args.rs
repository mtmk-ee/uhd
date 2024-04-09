use crate::Result;

use crate::Usrp;

/// Arguments for specifying a USRP available to the system.
///
/// # Examples
///
/// ```no_run
/// use uhd_usrp::DeviceArgs;
///
/// let device = DeviceArgs::new()
///     .addr("192.168.10.4")
///     .open()
///     .unwrap();
/// ```
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

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn open(self) -> Result<Usrp> {
        Usrp::open(self)
    }

    pub fn resource(mut self, resource: &str) -> Self {
        self.resource = Some(resource.to_owned());
        self
    }

    pub fn serial(mut self, serial: &str) -> Self {
        self.serial = Some(serial.to_owned());
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
}

impl ToString for DeviceArgs {
    fn to_string(&self) -> String {
        self.iter().collect::<Vec<String>>().join(",")
    }
}
