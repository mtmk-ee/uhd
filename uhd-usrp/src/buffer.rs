use crate::Sample;

pub trait SampleBuffer<T: Sample> {
    fn channels(&self) -> usize;
    fn samples_per_channel(&self) -> usize;
    fn as_ptr(&self) -> *const *const T;
    fn as_mut_ptr(&mut self) -> *mut *mut T;
    fn len(&self) -> usize {
        self.channels() * self.samples_per_channel()
    }
}

/// Cheap sample buffer backed by an array.
pub struct ArraySampleBuffer<T: Sample> {
    buffs: Box<[T]>,
    channels: usize,
}

impl<T: Sample + Default> ArraySampleBuffer<T> {
    pub fn new(channels: usize, samples_per_channel: usize) -> Self
    where
        T: Default + Clone,
    {
        let buffs = vec![T::default(); channels * samples_per_channel].into_boxed_slice();
        ArraySampleBuffer { buffs, channels }
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn get(&self, channel: usize) -> Option<&[T]> {
        self.buffs
            .get(channel * self.samples_per_channel()..(channel + 1) * self.samples_per_channel())
    }

    pub fn get_mut(&mut self, channel: usize) -> Option<&mut [T]> {
        self.buffs.get_mut(
            channel * self.samples_per_channel()..(channel + 1) * self.samples_per_channel(),
        )
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.buffs
    }

    pub fn into_vec(mut self) -> Vec<T> {
        let len = self.buffs.len();
        unsafe { Vec::from_raw_parts((*self.buffs).as_mut_ptr(), len, len) }
    }

    pub fn into_2d_vec(self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        let mut vec = Vec::with_capacity(self.channels);
        for i in 0..self.channels {
            if let Some(buff) = self.get(i) {
                vec.push(buff.to_vec());
            }
        }
        vec
    }

    pub fn iter_channels(&self) -> impl Iterator<Item = &[T]> {
        (0..self.channels).filter_map(|i| self.get(i))
    }

    pub fn iter_channels_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        let stride = std::mem::size_of::<T>() * self.channels;
        (0..self.channels).map(move |i| unsafe {
            std::slice::from_raw_parts_mut(
                self.buffs.as_mut_ptr().offset((stride * i) as isize),
                stride,
            )
        })
    }
}

impl<T: Sample> SampleBuffer<T> for ArraySampleBuffer<T> {
    fn channels(&self) -> usize {
        self.channels
    }

    fn samples_per_channel(&self) -> usize {
        self.buffs.len() / self.channels
    }

    fn as_ptr(&self) -> *const *const T {
        self.buffs.as_ptr() as *const *const T
    }

    fn as_mut_ptr(&mut self) -> *mut *mut T {
        self.buffs.as_mut_ptr() as *mut *mut T
    }

    fn len(&self) -> usize {
        self.buffs.len()
    }
}
