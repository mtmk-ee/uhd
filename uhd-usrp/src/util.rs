use std::marker::PhantomData;

pub type PhantomUnsync = PhantomData<std::cell::Cell<()>>;


macro_rules! gen_getter {
    ($name:path => ($($v1:expr),*, _)) => {
        {
            let mut result = std::mem::MaybeUninit::uninit();
            try_uhd!($name($($v1),*, result.as_mut_ptr()))
            .and_then(|_| Ok(result.assume_init()))
        }
    }
}
pub(crate) use gen_getter;
