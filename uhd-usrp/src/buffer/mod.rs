use crate::Sample;

mod arraybuffer;

pub use arraybuffer::ArrayBuffer;

/// Trait indicating a type is compatible with UHD's notion of a sample buffer.
///
/// A sample buffer has a representation as two-dimensional array of `Sample`s.
/// The first dimension is indexed by the channel number, while the second is
/// indexed by the sample number.
pub trait SampleBuffer<S: Sample> {
    fn channels(&self) -> usize;
    fn samples(&self) -> usize;
    fn as_ptr(&self) -> *const *const S;
    fn as_mut_ptr(&mut self) -> *mut *mut S;

    fn channel(&self, channel: usize) -> Option<&[S]>;
    fn channel_mut(&mut self, channel: usize) -> Option<&mut [S]>;

    fn iter_channels<'a>(&'a self) -> impl Iterator<Item = &'a [S]>
    where
        S: 'a;

    fn iter_channels_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [S]>
    where
        S: 'a;

    fn iter_samples<'a>(&'a self) -> impl Iterator<Item = &'a S>
    where
        S: 'a,
    {
        self.iter_channels().map(|samples| samples.iter()).flatten()
    }

    fn iter_samples_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut S>
    where
        S: 'a,
    {
        self.iter_channels_mut()
            .map(|samples| samples.iter_mut())
            .flatten()
    }
}

/// A slice `[S]` can be treated as a 1-channel [`SampleBuffer`] without requiring an additional
/// pointer to the slice.
impl<S: Sample> SampleBuffer<S> for [S] {
    fn channels(&self) -> usize {
        1
    }

    fn samples(&self) -> usize {
        self.len()
    }

    fn as_ptr(&self) -> *const *const S {
        self.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut *mut S {
        self.as_mut_ptr().cast()
    }

    fn channel(&self, channel: usize) -> Option<&[S]> {
        match channel {
            0 => Some(self),
            _ => None,
        }
    }

    fn channel_mut(&mut self, channel: usize) -> Option<&mut [S]> {
        match channel {
            0 => Some(self),
            _ => None,
        }
    }

    fn iter_channels<'a>(&'a self) -> impl Iterator<Item = &'a [S]>
    where
        S: 'a,
    {
        std::iter::once(self)
    }

    fn iter_channels_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [S]>
    where
        S: 'a,
    {
        std::iter::once(self)
    }
}
