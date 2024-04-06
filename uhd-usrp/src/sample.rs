
/// Trait indicating that a type can be used to represent samples.
///
/// This trait is marked unsafe because sending and receiving samples
/// using a type with an incompatible memory layout is undefined behavior.
pub unsafe trait Sample {
    /// The name corresponding to this type's data format.
    ///
    /// For example, `"fc32"` for a complex f32 sample.
    fn name() -> &'static str;
}

#[cfg(feature = "num")]
unsafe impl Sample for num_complex::Complex<f32> {
    fn name() -> &'static str {
        "fc32"
    }
}

#[cfg(feature = "num")]
unsafe impl Sample for num_complex::Complex<f64> {
    fn name() -> &'static str {
        "fc64"
    }
}

#[cfg(feature = "num")]
unsafe impl Sample for num_complex::Complex<i8> {
    fn name() -> &'static str {
        "sc8"
    }
}

#[cfg(feature = "num")]
unsafe impl Sample for num_complex::Complex<i16> {
    fn name() -> &'static str {
        "sc16"
    }
}

unsafe impl Sample for f32 {
    fn name() -> &'static str {
        "f32"
    }
}

unsafe impl Sample for f64 {
    fn name() -> &'static str {
        "f64"
    }
}

unsafe impl Sample for i8 {
    fn name() -> &'static str {
        "s8"
    }
}

unsafe impl Sample for i16 {
    fn name() -> &'static str {
        "s16"
    }
}

unsafe impl Sample for [f32; 2] {
    fn name() -> &'static str {
        "fc32"
    }
}

unsafe impl Sample for [f64; 2] {
    fn name() -> &'static str {
        "fc64"
    }
}

unsafe impl Sample for [i8; 2] {
    fn name() -> &'static str {
        "sc8"
    }
}

unsafe impl Sample for [i16; 2] {
    fn name() -> &'static str {
        "sc16"
    }
}
