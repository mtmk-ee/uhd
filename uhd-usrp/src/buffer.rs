use std::{
    fmt::Debug,
    mem::ManuallyDrop,
    ops::{Deref, Index, IndexMut},
};

use crate::Sample;

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
}

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
                    Box::leak(v.into_boxed_slice()).as_mut_ptr()
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
                    let mut x = Vec::with_capacity(samples);
                    unsafe { x.set_len(samples) };
                    Box::leak(x.into_boxed_slice()).as_mut_ptr()
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
    pub fn from_iter(channels: usize, iter: impl Iterator<Item = S>) -> Self
    where
        S: Clone,
    {
        Self::from_vec(channels, iter.collect())
    }

    pub fn from_nested_iter<I>(iter: impl Iterator<Item = I>) -> Self
    where
        I: Iterator<Item = S>,
    {
        Self::from_nested_vec(iter.map(|c| c.collect()).collect::<Vec<Vec<S>>>())
    }

    pub fn from_vec(channels: usize, value: Vec<S>) -> Self
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
                .map(|c| Box::leak(c.to_vec().into_boxed_slice()).as_mut_ptr())
                .collect(),
            channels,
            samples,
        }
    }

    pub fn from_nested_vec(value: Vec<Vec<S>>) -> Self {
        let channels = value.len();
        let samples = value.get(0).map(|c| c.len()).unwrap_or(0);
        if value.iter().skip(1).any(|c| c.len() != samples) {
            panic!("mismatched channel buffer lengths")
        }
        Self {
            inner: value
                .into_iter()
                .map(|c| Box::leak(c.into_boxed_slice()).as_mut_ptr())
                .collect(),
            channels,
            samples,
        }
    }

    pub fn get(&self, channel: usize) -> Option<&[S]> {
        Some(unsafe { std::slice::from_raw_parts(*self.inner.get(channel)?, self.samples) })
    }

    pub fn get_mut(&mut self, channel: usize) -> Option<&mut [S]> {
        Some(unsafe { std::slice::from_raw_parts_mut(*self.inner.get(channel)?, self.samples) })
    }

    pub fn fill(&mut self, value: S)
    where
        S: Clone,
    {
        self.iter_samples_mut().for_each(|s| *s = value.clone());
    }

    pub fn fill_channel(&mut self, channel: usize, value: S)
    where
        S: Clone,
    {
        self[channel].iter_mut().for_each(|s| *s = value.clone());
    }

    pub fn iter(&self) -> impl Iterator<Item = &[S]> {
        self.inner
            .iter()
            .map(|c| unsafe { std::slice::from_raw_parts(*c, self.samples) })
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = &mut [S]> {
        self.inner
            .iter()
            .map(|c| unsafe { std::slice::from_raw_parts_mut(*c, self.samples) })
    }

    pub fn iter_samples(&self) -> impl Iterator<Item = &S> {
        self.iter().map(|samples| samples.iter()).flatten()
    }

    pub fn iter_samples_mut(&mut self) -> impl Iterator<Item = &mut S> {
        self.iter_mut().map(|samples| samples.iter_mut()).flatten()
    }

    pub fn to_nested_vec(&self) -> Vec<Vec<S>>
    where
        S: Clone,
    {
        Vec::from_iter(self.iter().map(|c| c.to_vec()))
    }

    pub fn into_nested_vec(self) -> Vec<Vec<S>> {
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
}

impl<S> Drop for ArrayBuffer<S>
where
    S: Sample,
{
    fn drop(&mut self) {
        for i in self.inner.iter() {
            unsafe {
                let _ = Box::from_raw(i.cast::<&mut [S]>());
            }
        }
    }
}

impl<S: Sample + Clone> Clone for ArrayBuffer<S> {
    fn clone(&self) -> Self {
        Self::from_nested_vec(self.to_nested_vec())
    }
}

impl<S: Sample + PartialEq> PartialEq for ArrayBuffer<S> {
    fn eq(&self, other: &Self) -> bool {
        self.channels == other.channels
            && self.samples == other.samples
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
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

impl<S, O, I> From<O> for ArrayBuffer<S>
where
    S: Sample + Clone,
    O: Deref<Target = [I]>,
    I: Deref<Target = [S]>,
{
    fn from(value: O) -> Self {
        Self::from_nested_vec(value.iter().map(|c| c.to_vec()).collect::<Vec<Vec<S>>>())
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

impl<S> Index<usize> for ArrayBuffer<S>
where
    S: Sample,
{
    type Output = [S];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<S> IndexMut<usize> for ArrayBuffer<S>
where
    S: Sample,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod test_array_buff {
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
        check_values(ArrayBuffer::<i16>::from_iter(5, 0..100), 0..100);
        check_values(ArrayBuffer::from_vec(5, (0..100).collect()), 0..100);
    }

    #[test]
    pub fn test_shape() {
        let buff: ArrayBuffer<Complex32> = ArrayBuffer::new(10, 13);
        assert_eq!(buff.channels(), 10);
        assert_eq!(buff.samples(), 13);
        assert_eq!(buff.inner.len(), 10);
        assert!(buff.iter().all(|c| c.len() == 13))
    }

    #[test]
    pub fn test_iter() {
        const CHANNELS: usize = 10;
        const SAMPLES: usize = 13;

        let mut buff: ArrayBuffer<i16> = ArrayBuffer::new(CHANNELS, SAMPLES);
        buff.iter_samples_mut()
            .enumerate()
            .for_each(|(i, s)| *s = i as i16);
        check_values(buff, 0..(CHANNELS as i16 * SAMPLES as i16));
    }

    #[test]
    pub fn test_clone() {
        let buff: ArrayBuffer<i16> = ArrayBuffer::from_iter(5, 0..100);
        let clone = buff.clone();
        assert_eq!(buff, clone);
    }
}
