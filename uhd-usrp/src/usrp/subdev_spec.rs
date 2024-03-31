use std::{ffi::CString, mem::MaybeUninit, ptr::addr_of_mut};

use crate::{
    ffi::{self, OwnedHandle},
    try_uhd, Result,
};

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum SubdevSpecParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubdevPair {
    db_name: String,
    sd_name: String,
}

impl SubdevPair {
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    pub fn sd_name(&self) -> &str {
        &self.sd_name
    }
}

#[derive(Debug)]
pub struct SubdevSpec(OwnedHandle<ffi::uhd_subdev_spec_t>);

impl SubdevSpec {
    pub fn new() -> Self {
        let mut spec: MaybeUninit<_> = MaybeUninit::uninit();
        let empty = CString::new("").unwrap();
        // unwrap(): this won't normally fail since under the hood it's just POD allocation.
        try_uhd!(unsafe { ffi::uhd_subdev_spec_make(spec.as_mut_ptr(), empty.as_ptr()) }).unwrap();
        let spec = unsafe { OwnedHandle::from_ptr(spec.assume_init(), ffi::uhd_subdev_spec_free) };
        Self(spec)
    }

    pub fn from_str(s: &str) -> Self {
        Self::try_from(s).expect("invalid character(s) in subdev spec")
    }

    pub fn push(&mut self, db_name: &str, sd_name: &str) {
        let str = CString::new(format!("{db_name}:{sd_name}")).unwrap();
        try_uhd!(unsafe { ffi::uhd_subdev_spec_push_back(self.0.as_mut_ptr(), str.as_ptr()) })
            .unwrap();
    }

    pub fn len(&self) -> usize {
        let mut blah = 0;
        try_uhd!(unsafe { ffi::uhd_subdev_spec_size(self.0.as_mut_ptr(), addr_of_mut!(blah)) })
            .unwrap();
        blah
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<SubdevPair> {
        let mut banana = ffi::uhd_subdev_spec_pair_t {
            db_name: std::ptr::null_mut(),
            sd_name: std::ptr::null_mut(),
        };
        try_uhd!(unsafe {
            ffi::uhd_subdev_spec_at(self.0.as_mut_ptr(), index, addr_of_mut!(banana))
        })
        .ok()?;
        Some(SubdevPair {
            db_name: unsafe { CString::from_raw(banana.db_name) }
                .into_string()
                .unwrap(),
            sd_name: unsafe { CString::from_raw(banana.sd_name) }
                .into_string()
                .unwrap(),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = SubdevPair> + '_ {
        (0..self.len()).map(|i| self.get(i).unwrap())
    }

    pub(crate) fn as_handle(&self) -> &OwnedHandle<ffi::uhd_subdev_spec_t> {
        &self.0
    }
}

impl TryFrom<String> for SubdevSpec {
    type Error = SubdevSpecParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(&value)
    }
}

impl TryFrom<&str> for SubdevSpec {
    type Error = SubdevSpecParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut subdev_spec = SubdevSpec::new();
        value
            .split(' ')
            .try_for_each(|pair| -> Result<(), SubdevSpecParseError> {
                if pair.is_empty() {
                    return Err(SubdevSpecParseError::InvalidFormat(pair.to_string()));
                }
                let s: Vec<&str> = pair.split(':').collect();
                match s.len() {
                    1 => subdev_spec.push("", s[0]),
                    2 => subdev_spec.push(s[0], s[1]),
                    _ => {
                        return Err(SubdevSpecParseError::InvalidFormat(pair.to_string()));
                    }
                };
                Ok(())
            })?;
        Ok(subdev_spec)
    }
}

impl PartialEq for SubdevSpec {
    fn eq(&self, other: &Self) -> bool {
        if self.0.as_ptr() == other.0.as_ptr() {
            return true;
        } else if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for SubdevSpec {}

impl Clone for SubdevSpec {
    fn clone(&self) -> Self {
        let mut spec = SubdevSpec::new();
        for pair in self.iter() {
            spec.push(&pair.db_name(), &pair.sd_name());
        }
        spec
    }
}