use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::{addr_of, addr_of_mut},
};

use crate::{try_uhd, Result, UhdError};

pub(crate) struct FfiString<const N: usize> {
    s: [u8; N],
}

impl<const N: usize> FfiString<N> {
    pub fn new() -> Self {
        Self { s: [0; N] }
    }

    pub fn as_mut_ptr(&mut self) -> *mut i8 {
        self.s.as_mut_ptr().cast()
    }

    pub const fn max_chars(&self) -> usize {
        N - 1
    }

    pub fn into_string(self) -> Result<String> {
        Ok(CStr::from_bytes_until_nul(&self.s)
            .or(Err(UhdError::Unknown))?
            .to_string_lossy()
            .into_owned())
    }
}

pub(crate) struct FfiStringVec {
    handle: uhd_usrp_sys::uhd_string_vector_handle,
}

impl FfiStringVec {
    pub fn new() -> Result<FfiStringVec> {
        let mut handle = std::ptr::null_mut();
        try_uhd!(unsafe { uhd_usrp_sys::uhd_string_vector_make(addr_of_mut!(handle)) })?;
        Ok(Self { handle })
    }

    pub fn as_ptr(&self) -> *const uhd_usrp_sys::uhd_string_vector_handle {
        addr_of!(self.handle)
    }

    pub fn as_mut_ptr(&mut self) -> *mut uhd_usrp_sys::uhd_string_vector_handle {
        addr_of_mut!(self.handle)
    }

    pub fn push(&mut self, value: &str) -> Result<()> {
        let value = CString::new(value).unwrap();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_string_vector_push_back(addr_of_mut!(self.handle), value.as_ptr())
        })?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        let mut value = 0;
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_string_vector_size(self.handle, addr_of_mut!(value))
        })?;
        Ok(value)
    }

    pub fn get(&self, index: usize) -> Option<String> {
        let mut s = FfiString::<128>::new();
        try_uhd!(unsafe {
            uhd_usrp_sys::uhd_string_vector_at(self.handle, index, s.as_mut_ptr(), s.max_chars())
        })
        .ok()?;
        s.into_string().ok()
    }

    pub fn to_vec(&self) -> Result<Vec<String>> {
        let mut result = vec![];
        for i in 0..self.len()? {
            if let Some(v) = self.get(i) {
                result.push(v);
            }
        }
        Ok(result)
    }
}

impl Drop for FfiStringVec {
    fn drop(&mut self) {
        unsafe {
            uhd_usrp_sys::uhd_string_vector_free(addr_of_mut!(self.handle));
        }
    }
}

pub(crate) struct OwnedHandle<T> {
    handle: *mut T,
    free: unsafe extern "C" fn(*mut *mut T) -> u32,
}

impl<T> OwnedHandle<T> {
    pub fn new(
        alloc: unsafe extern "C" fn(*mut *mut T) -> u32,
        free: unsafe extern "C" fn(*mut *mut T) -> u32,
    ) -> Result<Self> {
        let mut handle = MaybeUninit::uninit();
        try_uhd!(unsafe { alloc(handle.as_mut_ptr()) })?;
        Ok(Self {
            handle: unsafe { handle.assume_init() },
            free,
        })
    }

    pub unsafe fn from_ptr(handle: *mut T, free: unsafe extern "C" fn(*mut *mut T) -> u32) -> Self {
        Self { handle, free }
    }

    pub fn as_ptr(&self) -> *const T {
        self.handle
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.handle
    }

    pub fn as_mut_mut_ptr(&self) -> *mut *mut T {
        addr_of!(self.handle).cast_mut()
    }
}

impl<T> Drop for OwnedHandle<T> {
    fn drop(&mut self) {
        unsafe {
            (self.free)(addr_of_mut!(self.handle));
        }
    }
}

