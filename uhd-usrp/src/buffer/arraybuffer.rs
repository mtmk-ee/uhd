use std::{
    fmt::Debug,
    mem::ManuallyDrop,
    ops::{Deref, Index, IndexMut},
};

use crate::{Sample, SampleBuffer};

/// Lightweight 2D sample buffer where each channel is backed by contiguous memory.
///
/// In many ways this type behaves as a `[&[S]]`. The first dimension is indexed by the
/// channel number, while the second is indexed by the sample number.
pub struct ArrayBuffer<S: Sample> {
    /// Sample memory. Each `*mut S` is a leaked boxed slice whose length is equal to `samples`.
    inner: Box<[*mut S]>,
    channels: usize,
    samples: usize,
}

impl<S: Sample> ArrayBuffer<S> {
    /// Creates a new `ArrayBuffer` with all samples initialized to the default sample value.
    pub fn new(channels: usize, samples: usize) -> Self
    where
        S: Clone + Default,
    {
        Self::with_fill(channels, samples, Default::default())
    }

    /// Creates a new `ArrayBuffer` with all samples initialized to the given fill value.
    pub fn with_fill(channels: usize, samples: usize, fill: S) -> Self
    where
        S: Clone,
    {
        Self {
            inner: (0..channels)
                .map(|_| {
                    let v = vec![fill.clone(); samples];
                    Box::into_raw(v.into_boxed_slice()).cast()
                })
                .collect(),
            channels,
            samples,
        }
    }

    /// Creates a new `ArrayBuffer` with uninitialized sample instances.
    ///
    /// This can be used to avoid overhead caused by zero-initialization of samples for
    /// large buffers.
    ///
    /// This function can also be used for creating a buffer of samples whose type do
    /// not implement `Clone` or `Default`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that appropriate portions of the buffer are initialized properly
    /// before being used. Proper initialization could be receiving samples from a USRP or setting
    /// necessary sample to a valid value.
    pub unsafe fn uninit(channels: usize, samples: usize) -> Self {
        Self {
            inner: (0..channels)
                .map(|_| {
                    let mut x: Vec<S> = Vec::with_capacity(samples);
                    unsafe { x.set_len(samples) };
                    Box::into_raw(x.into_boxed_slice()).cast()
                })
                .collect(),
            channels,
            samples,
        }
    }

    /// Create a new `ArrayBuffer` with the given iterator and number of channels.
    ///
    /// # Panics
    ///
    /// This function will panic if the number of elements yielded by the iterator is
    /// not divisible by the number of channels.
    pub fn from_iter_samples(channels: usize, iter: impl Iterator<Item = S>) -> Self
    where
        S: Clone,
    {
        Self::from_vec_samples(channels, iter.collect())
    }

    /// Create a new `ArrayBuffer` with the given nested iterators.
    ///
    /// # Panics
    ///
    /// This function will panic if the inner iterators do not yield the same
    /// number of samples.
    pub fn from_iter_channels<I>(iter: impl Iterator<Item = I>) -> Self
    where
        I: Iterator<Item = S>,
    {
        Self::from_vec_channels(iter.map(|i| i.collect()).collect())
    }

    /// Builds an `ArrayBuffer` from a flat list of samples.
    ///
    ///
    /// The order of the given samples is assumed to be:
    /// `[S(0,0), S(0,1), S(0,2)..., S(1,0), S(1,1), S(1,2), ...]`, where `S(i,j)`
    /// is sample `j` of channel `i`.
    ///
    /// # Panics
    ///
    /// Panics if the length of the given `Vec` is not divisible by `channels`.
    pub fn from_vec_samples(channels: usize, value: Vec<S>) -> Self
    where
        S: Clone,
    {
        if value.len() % channels != 0 {
            panic!("mismatched channel buffer lengths");
        }
        let samples = value.len() / channels;

        Self {
            inner: value
                .chunks(samples)
                .map(|c| Box::into_raw(c.to_vec().into_boxed_slice()).cast())
                .collect(),
            channels,
            samples,
        }
    }

    /// Builds an `ArrayBuffer` from a nested `Vec` of samples.
    ///
    /// Each inner vector corresponds to the samples for a single channel.
    /// All inner vectors must have the same length.
    ///
    /// # Panics
    ///
    /// Will panic if the inner vectors do not have the same length.
    pub fn from_vec_channels(value: Vec<Vec<S>>) -> Self {
        let channels = value.len();
        let samples = value.get(0).map(|c| c.len()).unwrap_or(0);
        if value.iter().skip(1).any(|c| c.len() != samples) {
            panic!("mismatched channel buffer lengths")
        }
        Self {
            inner: value
                .into_iter()
                .map(|c| Box::into_raw(c.into_boxed_slice()).cast())
                .collect(),
            channels,
            samples,
        }
    }

    /// Returns the samples belonging to the specified channel.
    ///
    /// `None` is returned if the channel is out of bounds.
    pub fn channel(&self, channel: usize) -> Option<&[S]> {
        // SAFETY: the data was originally obtained using `Vec::into_boxed_slice`,
        // which has the following implications:
        // - the memory is valid for both reads and writes and is aligned
        // - the memory is contiguous single allocation of the correct length
        //
        // Also, the lifetime constraints on this function and the shared reference
        // guarantee the memory is not mutated elsewhere for the lifetime of 'a
        Some(unsafe { std::slice::from_raw_parts(*self.inner.get(channel)?, self.samples) })
    }

    /// Returns the samples belonging to the specified channel.
    ///
    /// `None` is returned if the channel is out of bounds.
    pub fn channel_mut(&mut self, channel: usize) -> Option<&mut [S]> {
        // SAFETY: the data was originally obtained using `Vec::into_boxed_slice`,
        // which has the following implications:
        // - the memory is valid for both reads and writes and is aligned
        // - the memory is contiguous single allocation of the correct length
        //
        // Also, the lifetime constraints on this function and the exclusive reference
        // guarantee the memory is not accessed elsewhere for the lifetime of 'a
        Some(unsafe { std::slice::from_raw_parts_mut(*self.inner.get(channel)?, self.samples) })
    }

    /// Build a nested vector of samples from this `ArrayBuffer`.
    ///
    /// Each inner `Vec<S>` corresponds to a single channel, ordered in the same
    /// way as this `ArrayBuffer`.
    ///
    /// Using [`ArrayBuffer::into_vec`] is preferred over this function where possible,
    /// as this function copies each channel's sample buffer.
    pub fn to_vec(&self) -> Vec<Vec<S>>
    where
        S: Clone,
    {
        self.iter_channels().map(|c| c.to_vec()).collect()
    }

    /// Copies each sample from each channel into a flat vector.
    ///
    /// The order of the returned samples is guaranteed to be:
    /// `[S(0,0), S(0,1), S(0,2)..., S(1,0), S(1,1), S(1,2), ...]`, where `S(i,j)`
    /// is sample `j` of channel `i`.
    pub fn to_flat_vec(&self) -> Vec<S>
    where
        S: Clone,
    {
        self.iter_channels()
            .map(|c| c.iter())
            .flatten()
            .map(|s| s.clone())
            .collect()
    }

    /// Consume self and build a nested vector of samples.
    ///
    /// Each inner `Vec<S>` corresponds to a single channel, ordered in the same
    /// way as this `ArrayBuffer`.
    ///
    /// Using this function should be preferred over [`ArrayBuffer::to_vec`]
    /// as this function avoids copying each channel's sample buffer.
    pub fn into_vec(self) -> Vec<Vec<S>> {
        let mut shelf = ManuallyDrop::new(self);
        let inner = std::mem::take(&mut shelf.inner);
        let v = inner
            .iter()
            .map(|c| {
                // SAFETY:
                // - element type is the same before and after
                // - all elements are initialized, unless `Self::new_uninit` was used
                // - `Vec::into_boxed_slice` shrinks the capacity to len
                unsafe { Vec::from_raw_parts(*c, shelf.samples, shelf.samples) }
            })
            .collect();
        v
    }

    /// Returns an iterator over the channels of this `ArrayBuffer`.
    ///
    /// Each yielded element is a reference to the channel's corresponding sample buffer.
    pub fn iter_channels<'a>(&'a self) -> impl Iterator<Item = &'a [S]>
    where
        S: 'a,
    {
        self.inner.iter().map(|c| {
            // SAFETY: the data at `c` was originally obtained using `Vec::into_boxed_slice`,
            // which has the following implications:
            // - the memory is valid for both reads and writes and is aligned
            // - the memory is contiguous single allocation of the correct length
            //
            // Also, the lifetime constraints on this function and the shared reference
            // guarantee the memory is not mutated elsewhere for the lifetime of 'a
            unsafe { std::slice::from_raw_parts(*c, self.samples) }
        })
    }
    /// Returns an iterator over the channels of this `ArrayBuffer`.
    ///
    /// Each yielded element is a reference to the channel's corresponding sample buffer.
    pub fn iter_channels_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut [S]>
    where
        S: 'a,
    {
        self.inner.iter().map(|c| {
            // SAFETY: the data at `c` was originally obtained using `Vec::into_boxed_slice`,
            // which has the following implications:
            // - the memory is valid for both reads and writes and is aligned
            // - the memory is contiguous single allocation of the correct length
            //
            // Also, the lifetime constraints on this function and the exclusive reference
            // guarantee the memory is not accessed elsewhere for the lifetime of 'a
            unsafe { std::slice::from_raw_parts_mut(*c, self.samples) }
        })
    }
    /// Returns a flattened iterator over all samples from all channels.
    ///
    /// The order of the returned samples is guaranteed to be:
    /// `[S(0,0), S(0,1), S(0,2)..., S(1,0), S(1,1), S(1,2), ...]`, where `S(i,j)`
    /// is sample `j` of channel `i`.
    pub fn iter_samples<'a>(&'a self) -> impl Iterator<Item = &'a S>
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
    pub fn iter_samples_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut S>
    where
        S: 'a,
    {
        self.iter_channels_mut()
            .map(|samples| samples.iter_mut())
            .flatten()
    }
}

impl<S> SampleBuffer<S> for ArrayBuffer<S>
where
    S: Sample,
{
    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> usize {
        self.samples
    }

    fn as_ptr(&self) -> *const *const S {
        self.inner.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut *mut S {
        self.inner.as_mut_ptr()
    }
}

impl<S> Drop for ArrayBuffer<S>
where
    S: Sample,
{
    fn drop(&mut self) {
        for i in self.inner.iter() {
            // SAFETY: the data being reclaimed with `Box::from_raw` was originally obtained
            // using `Box::into_raw`.
            unsafe {
                let _ = Box::from_raw(i.cast::<&mut [S]>());
            }
        }
    }
}

impl<S: Sample + Clone> Clone for ArrayBuffer<S> {
    fn clone(&self) -> Self {
        Self::from_vec_channels(self.to_vec())
    }
}

impl<S: Sample + PartialEq> PartialEq for ArrayBuffer<S> {
    fn eq(&self, other: &Self) -> bool {
        self.channels == other.channels
            && self.samples == other.samples
            && self
                .iter_channels()
                .zip(other.iter_channels())
                .all(|(a, b)| a == b)
    }
}
impl<S: Sample + Eq> Eq for ArrayBuffer<S> {}

impl<S: Sample + Debug> Debug for ArrayBuffer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayBuffer")
            .field("channels", &self.channels)
            .field("samples", &self.samples)
            .finish()
    }
}

// e.g. From<&[&[S]]>, but also works for Vec<Vec<S>>, &[Vec<S>], etc.
impl<S, O, I> From<O> for ArrayBuffer<S>
where
    S: Sample + Clone,
    O: Deref<Target = [I]>,
    I: Deref<Target = [S]>,
{
    fn from(value: O) -> Self {
        Self::from_vec_channels(value.iter().map(|c| c.to_vec()).collect::<Vec<Vec<S>>>())
    }
}

impl<S> Index<usize> for ArrayBuffer<S>
where
    S: Sample,
{
    type Output = [S];

    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).expect("index out of bounds")
    }
}

impl<S> IndexMut<usize> for ArrayBuffer<S>
where
    S: Sample,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod test {
    use num_complex::Complex32;

    use crate::{ArrayBuffer, SampleBuffer};

    fn check_fill(mut a: ArrayBuffer<i16>) {
        a.iter_samples_mut()
            .enumerate()
            .for_each(|(i, s)| *s = i as i16);
        for i in 0..(a.channels * a.samples) {
            assert_eq!(a[i / a.samples][i % a.samples], i as i16);
        }
    }

    fn check_values(a: ArrayBuffer<i16>, s: impl Iterator<Item = i16>) {
        assert!(a.iter_samples().zip(s).all(|(s1, s2)| *s1 == s2));
    }

    #[test]
    pub fn test_creation() {
        check_fill(ArrayBuffer::new(3, 10));
        check_fill(unsafe { ArrayBuffer::uninit(3, 10) });
        check_values(ArrayBuffer::<i16>::from_iter_samples(5, 0..100), 0..100);
        check_values(ArrayBuffer::from_vec_samples(5, (0..100).collect()), 0..100);
    }

    #[test]
    pub fn test_shape() {
        let buff: ArrayBuffer<Complex32> = ArrayBuffer::new(10, 13);
        assert_eq!(buff.channels(), 10);
        assert_eq!(buff.samples(), 13);
        assert_eq!(buff.inner.len(), 10);
        assert!(buff.iter_channels().all(|c| c.len() == 13))
    }

    #[test]
    pub fn test_iter_samples() {
        const CHANNELS: usize = 10;
        const SAMPLES: usize = 13;

        let mut buff: ArrayBuffer<i16> = ArrayBuffer::new(CHANNELS, SAMPLES);
        buff.iter_samples_mut()
            .enumerate()
            .for_each(|(i, s)| *s = i as i16);
        check_values(buff, 0..(CHANNELS as i16 * SAMPLES as i16));
    }

    #[test]
    pub fn test_iter_channels() {
        const CHANNELS: usize = 10;

        let mut buff: ArrayBuffer<i16> = ArrayBuffer::from_iter_samples(CHANNELS, 0..100);
        buff.iter_channels_mut()
            .enumerate()
            .for_each(|(i, s)| s[0] = i as i16);
        for (i, chan) in buff.iter_channels().enumerate() {
            assert_eq!(chan[0], i as i16);
        }
    }

    #[test]
    pub fn test_clone() {
        let buff: ArrayBuffer<i16> = ArrayBuffer::from_iter_samples(5, 0..100);
        let clone = buff.clone();
        assert_eq!(buff, clone);
    }

    #[test]
    pub fn test_to_vec() {
        let v0: Vec<i16> = Vec::from_iter(0..100);
        let v1 = ArrayBuffer::from_iter_samples(10, v0.iter().map(Clone::clone)).to_vec();
        let v2 = ArrayBuffer::from_iter_samples(10, v0.iter().map(Clone::clone)).into_vec();
        let v3 = ArrayBuffer::from_iter_samples(10, v0.iter().map(Clone::clone)).to_flat_vec();
        assert_eq!(
            v0,
            v1.iter().flatten().map(|c| c.clone()).collect::<Vec<_>>()
        );
        assert_eq!(
            v0,
            v2.iter().flatten().map(|c| c.clone()).collect::<Vec<_>>()
        );
        assert_eq!(v0, v3);
    }
}
