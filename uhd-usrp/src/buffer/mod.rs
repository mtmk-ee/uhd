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
    /// Returns the samples belonging to the specified channel.
    ///
    /// `None` is returned if the channel is out of bounds.
    fn channel(&self, channel: usize) -> Option<&[S]>;
    /// Returns the samples belonging to the specified channel.
    ///
    /// `None` is returned if the channel is out of bounds.
    fn channel_mut(&mut self, channel: usize) -> Option<&mut [S]>;
    /// Returns an iterator over the channels of this `ArrayBuffer`.
    ///
    /// Each yielded element is a reference to the channel's corresponding sample buffer.
    fn iter_channels<'a>(&'a self) -> impl Iterator<Item = &'a [S]>
    where
        S: 'a;
    /// Returns an iterator over the channels of this `ArrayBuffer`.
    ///
    /// Each yielded element is a reference to the channel's corresponding sample buffer.
    fn iter_channels_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [S]>
    where
        S: 'a;
    /// Returns a flattened iterator over all samples from all channels.
    ///
    /// The order of the returned samples is guaranteed to be:
    /// `[S(0,0), S(0,1), S(0,2)..., S(1,0), S(1,1), S(1,2), ...]`, where `S(i,j)`
    /// is sample `j` of channel `i`.
    fn iter_samples<'a>(&'a self) -> impl Iterator<Item = &'a S>
    where
        S: 'a,
    {
        self.iter_channels().map(|samples| samples.iter()).flatten()
    }
    /// Returns a flattened iterator over all samples from all channels.
    ///
    /// The order of the returned samples is guaranteed to be:
    /// `[S(0,0), S(0,1), S(0,2)..., S(1,0), S(1,1), S(1,2), ...]`, where `S(i,j)`
    /// is sample `j` of channel `i`.
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
