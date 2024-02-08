use std::ops::Deref;

use crate::Sample;

pub trait SampleBuffer<T: Sample> {
    fn channels(&self) -> usize;
    fn samples_per_channel(&self) -> usize;
    fn as_ptrs(&self) -> Box<[*const T]>;
    fn as_mut_ptrs(&mut self) -> Box<[*mut T]>;
}

impl<T: Sample, D> SampleBuffer<T> for [D]
where
    D: Deref<Target = [T]>,
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

    fn as_ptrs(&self) -> Box<[*const T]> {
        self.iter().map(|c| c.as_ptr()).collect()
    }

    fn as_mut_ptrs(&mut self) -> Box<[*mut T]> {
        self.iter().map(|c| c.as_ptr().cast_mut()).collect()
    }
}
