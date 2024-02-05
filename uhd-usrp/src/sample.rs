use num::Complex;

pub trait Sample {
    fn name() -> &'static str;
}

impl Sample for Complex<f32> {
    fn name() -> &'static str {
        "fc32"
    }
}

impl Sample for Complex<f64> {
    fn name() -> &'static str {
        "fc64"
    }
}

impl Sample for Complex<i8> {
    fn name() -> &'static str {
        "sc8"
    }
}

impl Sample for Complex<i16> {
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
