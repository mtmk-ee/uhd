use std::{ffi::CString, mem::MaybeUninit, ptr::addr_of_mut};

use crate::{
    ffi::{self, OwnedHandle},
    try_uhd, Result,
};

/// A subdevice specification.
///
/// This is a list of pairs of daughterboard and subdevice names.
/// See the [page on subdevices](https://files.ettus.com/manual/page_configuration.html) for more information.
#[derive(Debug)]
pub struct SubdevSpec(OwnedHandle<ffi::uhd_subdev_spec_t>);

/// A pair of daughterboard and subdevice names.
///
/// A subdevice is selected using a string representation: `"db_name:sd_name"`.
/// See the [page on subdevices](https://files.ettus.com/manual/page_configuration.html) for more information.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubdevPair {
    db_name: String,
    sd_name: String,
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum SubdevSpecParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl SubdevSpec {
    /// Create an empty subdevice specification.
    pub fn new() -> Self {
        let mut spec: MaybeUninit<_> = MaybeUninit::uninit();
        let empty = CString::new("").unwrap();
        // unwrap(): this won't normally fail since under the hood it's just POD allocation.
        try_uhd!(unsafe { ffi::uhd_subdev_spec_make(spec.as_mut_ptr(), empty.as_ptr()) }).unwrap();
        let spec = unsafe { OwnedHandle::from_ptr(spec.assume_init(), ffi::uhd_subdev_spec_free) };
        Self(spec)
    }

    /// Create a subdevice specification from a string representation, such as `"A:0"`.
    ///
    /// # Panics
    ///
    /// Panics if the provided string is in an invalid format.
    pub fn from_str(s: &str) -> Self {
        Self::try_from(s).expect("invalid character(s) in subdev spec")
    }

    /// Add a subdevice.
    pub fn push(&mut self, db_name: &str, sd_name: &str) {
        let str = CString::new(format!("{db_name}:{sd_name}")).unwrap();
        try_uhd!(unsafe { ffi::uhd_subdev_spec_push_back(self.0.as_mut_ptr(), str.as_ptr()) })
            .unwrap();
    }

    /// Get the number of subdevices in the specification.
    pub fn len(&self) -> usize {
        let mut blah = 0;
        try_uhd!(unsafe { ffi::uhd_subdev_spec_size(self.0.as_mut_ptr(), addr_of_mut!(blah)) })
            .unwrap();
        blah
    }

    /// Check if the subdevice specification is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the subdevice at the given index.
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

    /// Iterate over the subdevices.
    pub fn iter(&self) -> impl Iterator<Item = SubdevPair> + '_ {
        (0..self.len()).map(|i| self.get(i).unwrap())
    }

    /// Get the subdevice specification as an [`OwnedHandle`].
    pub(crate) fn handle(&self) -> &OwnedHandle<ffi::uhd_subdev_spec_t> {
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

impl SubdevPair {
    /// The daughterboard name
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// The subdevice name
    pub fn sd_name(&self) -> &str {
        &self.sd_name
    }
}
