pub trait Sample {
    fn name() -> &'static str;
}

#[cfg(feature = "num")]
impl Sample for num_complex::Complex<f32> {
    fn name() -> &'static str {
        "fc32"
    }
}

#[cfg(feature = "num")]
impl Sample for num_complex::Complex<f64> {
    fn name() -> &'static str {
        "fc64"
    }
}

#[cfg(feature = "num")]
impl Sample for num_complex::Complex<i8> {
    fn name() -> &'static str {
        "sc8"
    }
}

#[cfg(feature = "num")]
impl Sample for num_complex::Complex<i16> {
    fn name() -> &'static str {
        "sc16"
    }
}

impl Sample for f32 {
    fn name() -> &'static str {
        "f32"
    }
}

impl Sample for f64 {
    fn name() -> &'static str {
        "f64"
    }
}

impl Sample for i8 {
    fn name() -> &'static str {
        "s8"
    }
}

impl Sample for i16 {
    fn name() -> &'static str {
        "s16"
    }
}
