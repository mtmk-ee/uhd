use std::ops::DerefMut;

use crate::Sample;

mod arraybuffer;

pub use arraybuffer::ArrayBuffer;

/// Trait indicating a type is compatible with UHD's notion of a sample buffer.
///
/// A sample buffer has a representation as two-dimensional array of `Sample`s.
/// The first dimension is indexed by the channel number, while the second is
/// indexed by the sample number: `buff[channel][sample]`.
pub trait SampleBuffer<S: Sample> {
    /// The number of channels contained in the sample buffer.
    fn channels(&self) -> usize;
    /// The number of samples per channel.
    fn samples(&self) -> usize;
    /// Returns the data buffer as a bare pointer.
    ///
    /// This pointer points to an array where each element is
    /// a pointer to a channel's sample buffer.
    fn as_ptr(&self) -> *const *const S;
    /// Returns the data buffer as a bare pointer.
    ///
    /// This pointer points to an array where each element is
    /// a pointer to a channel's sample buffer.
    fn as_mut_ptr(&mut self) -> *mut *mut S;
}

/// A slice `[S]` can be treated as a 1-channel [`SampleBuffer`] without requiring an additional
/// pointer to the slice.
impl<S: Sample, T> SampleBuffer<S> for T
where
    T: DerefMut<Target = [S]>,
{
    fn channels(&self) -> usize {
        1
    }

    fn samples(&self) -> usize {
        self.len()
    }

    fn as_ptr(&self) -> *const *const S {
        (self as *const T).cast()
    }

    fn as_mut_ptr(&mut self) -> *mut *mut S {
        (self as *mut T).cast()
    }
}

/// A zero-sized buffer. Useful for sending metadata without any samples, such
/// as end of burst packets.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EmptyBuf;

impl<S> SampleBuffer<S> for EmptyBuf
where
    S: Sample,
{
    fn channels(&self) -> usize {
        0
    }

    fn samples(&self) -> usize {
        0
    }

    fn as_ptr(&self) -> *const *const S {
        b"\0".as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut *mut S {
        panic!("cannot mutate empty buffer")
    }
}

#[cfg(test)]
mod test {
    use crate::{Sample, SampleBuffer};

    fn takes_samplebuff<S: Sample>(_buff: &impl SampleBuffer<S>) {}
    fn takes_mut_samplebuff<S: Sample>(_buff: &mut impl SampleBuffer<S>) {}

    #[test]
    fn test_blah() {
        let mut v = vec![0i16; 10];
        takes_samplebuff(&v);
        takes_mut_samplebuff(&mut v);
    }
}
