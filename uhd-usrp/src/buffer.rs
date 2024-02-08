use std::{
    ops::{Deref, Index, IndexMut},
    slice::SliceIndex,
};

use crate::Sample;

pub trait SampleBuffer<S: Sample> {
    fn channels(&self) -> usize;
    fn samples_per_channel(&self) -> usize;
    fn as_ptrs(&self) -> Box<[*const S]>;
    fn as_mut_ptrs(&mut self) -> Box<[*mut S]>;
}

impl<S: Sample> SampleBuffer<S> for [S] {
    fn channels(&self) -> usize {
        1
    }

    fn samples_per_channel(&self) -> usize {
        self.len()
    }

    fn as_ptrs(&self) -> Box<[*const S]> {
        Box::new([self.as_ptr()])
    }

    fn as_mut_ptrs(&mut self) -> Box<[*mut S]> {
        Box::new([self.as_ptr().cast_mut()])
    }
}

// Hack to allow [&[S]]
impl<S, O, I> SampleBuffer<S> for O
where
    S: Sample,
    O: Deref<Target = [I]>,
    I: Deref<Target = [S]>,
{
    fn channels(&self) -> usize {
        self.len()
    }

    fn samples_per_channel(&self) -> usize {
        if self.len() == 0 {
            return 0;
        } else if self.iter().skip(1).any(|c| c.len() != self[0].len()) {
            panic!("mismatched channel buffer lengths");
        } else {
            self[0].len()
        }
    }

    fn as_ptrs(&self) -> Box<[*const S]> {
        self.iter().map(|c| c.as_ptr()).collect()
    }

    fn as_mut_ptrs(&mut self) -> Box<[*mut S]> {
        self.iter().map(|c| c.as_ptr().cast_mut()).collect()
    }
}

/// Lightweight 2D sample buffer where each channel is backed by contiguous memory.
///
/// An `ArrayBuffer` implements `Deref` and can be indexed twice. The first index corresponds to the
/// channel, and the second to the sample. All channel buffers have the same length.
pub struct ArrayBuffer<S: Sample> {
    inner: Box<[Box<[S]>]>,
    samples_per_channel: usize,
}

impl<S: Sample> ArrayBuffer<S> {
    /// Creates a new `ArrayBuffer` initialized with the default value for the `S` sample type.
    pub fn new(channels: usize, samples_per_channel: usize) -> Self
    where
        S: Clone + Default,
    {
        Self {
            inner: (0..channels)
                .map(|_| vec![<S as Default>::default(); samples_per_channel].into_boxed_slice())
                .collect(),
            samples_per_channel,
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
    pub unsafe fn new_uninit(channels: usize, samples_per_channel: usize) -> Self {
        Self {
            inner: (0..channels)
                .map(|_| {
                    let mut x = Vec::with_capacity(samples_per_channel);
                    unsafe { x.set_len(samples_per_channel) };
                    x.into_boxed_slice()
                })
                .collect(),
            samples_per_channel,
        }
    }

    pub fn from_vec(vec: Vec<Vec<S>>) -> Self {
        let samples_per_channel = match vec.len() {
            0 => 0,
            _ => {
                if vec.iter().skip(1).any(|x| x.len() != vec[0].len()) {
                    panic!("mismatched channel buffer lengths");
                }
                vec[0].len()
            }
        };
        Self {
            inner: vec.into_iter().map(|x| x.into_boxed_slice()).collect(),
            samples_per_channel,
        }
    }

    pub fn into_inner(self) -> Box<[Box<[S]>]> {
        self.inner
    }

    pub fn to_vec(&self) -> Vec<Vec<S>>
    where
        S: Clone,
    {
        self.inner.iter().map(|x| x.to_vec()).collect()
    }

    pub fn into_vec(self) -> Vec<Vec<S>> {
        self.inner
            .into_vec()
            .into_iter()
            .map(|x| x.into_vec())
            .collect()
    }

    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[Box<[S]>]>,
    {
        self.inner.get(index)
    }

    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[Box<[S]>]>,
    {
        self.inner.get_mut(index)
    }
}

impl<S: Sample> SampleBuffer<S> for ArrayBuffer<S> {
    fn channels(&self) -> usize {
        self.inner.len()
    }

    fn samples_per_channel(&self) -> usize {
        self.samples_per_channel
    }

    fn as_ptrs(&self) -> Box<[*const S]> {
        self.inner.iter().map(|x| x.as_ptr()).collect()
    }

    fn as_mut_ptrs(&mut self) -> Box<[*mut S]> {
        self.inner.iter().map(|x| x.as_ptr().cast_mut()).collect()
    }
}

impl<S: Sample, I> Index<I> for ArrayBuffer<S>
where
    I: SliceIndex<[Box<[S]>]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<S: Sample, I> IndexMut<I> for ArrayBuffer<S>
where
    I: SliceIndex<[Box<[S]>]>,
{
    fn index_mut(&mut self, index: I) -> &mut <I as SliceIndex<[Box<[S]>]>>::Output {
        self.inner.index_mut(index)
    }
}
