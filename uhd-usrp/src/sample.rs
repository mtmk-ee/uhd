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
