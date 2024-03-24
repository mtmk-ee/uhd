use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    time::Duration,
};

use crate::error::TimeError;

/// Creates a new TimeSpec from a number of seconds and a unit.
///
/// Possible units are:
/// - `m` for minutes
/// - `s` for seconds
/// - `ms` for milliseconds
/// - `us` for microseconds
/// - `ns` for nanoseconds
///
/// # Panics
///
/// Panics if:
/// - the value cannot be represented without overflow
/// - the value is NaN or infinite
///
/// # Examples
///
/// ```rust
/// use uhd_usrp::{TimeSpec, timespec};
///
/// assert_eq!(timespec!(0), TimeSpec::ZERO);
/// assert_eq!(timespec!(0.5 s), TimeSpec::from_parts(0, 0.5));
/// assert_eq!(timespec!(0.5 s), TimeSpec::from_parts(0, 0.5));
/// ```
///
/// Variables can also be used:
///
/// ```rust
/// use uhd_usrp::{TimeSpec, timespec};
///
/// let t = 0.5;
/// assert_eq!(timespec!(t s), TimeSpec::from_parts(0, 0.5));
/// ```
///
/// Expressions cannot be used for the numeric portion.
#[macro_export]
macro_rules! timespec {
    (0) => {{
        TimeSpec::ZERO
    }};
    ($val:literal m) => {{
        TimeSpec::from_secs_f64($val as f64 * 60.0)
    }};
    ($val:literal s) => {{
        TimeSpec::from_secs_f64($val as f64)
    }};
    ($val:literal ms) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e3)
    }};
    ($val:literal us) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e6)
    }};
    ($val:literal ns) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e9)
    }};
    ($val:ident m) => {{
        TimeSpec::from_secs_f64($val as f64 * 60.0)
    }};
    ($val:ident s) => {{
        TimeSpec::from_secs_f64($val as f64)
    }};
    ($val:ident ms) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e3)
    }};
    ($val:ident us) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e6)
    }};
    ($val:ident ns) => {{
        TimeSpec::from_secs_f64($val as f64 / 1e9)
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TimeSpec {
    /// The number of full seconds.
    ///
    /// For negative timespecs the number of full seconds may be one less
    /// than expected. Note that `full_secs + frac_secs` will still yield
    /// the expected value.
    full_secs: i64,
    /// The number of fractional seconds. For valid timespecs this is always
    /// in the range `[0, 1)`.
    frac_secs: f64,
}

impl TimeSpec {
    /// A special value that signifies immediate execution.
    pub const ZERO: TimeSpec = TimeSpec::from_parts_unchecked(0, 0.0);
    pub const MAX: TimeSpec = TimeSpec::from_parts_unchecked(i64::MAX, 1.0 - f64::EPSILON);
    pub const MIN: TimeSpec = TimeSpec::from_parts_unchecked(i64::MIN, 0.0);

    /// Create a new TimeSpec using the number of full and fractional seconds.
    ///
    /// `frac_seconds` need not be in the range `[0, 1)`; it will be normalized
    /// to that range internally.
    ///
    /// Either or both values may be negative. The representation regardless
    /// is `full_secs + frac_secs`.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - the given values cannot be represented without overflow
    /// - the given values are NaN or infinite
    pub fn from_parts(full_secs: i64, frac_secs: f64) -> Self {
        Self::try_from_parts(full_secs, frac_secs)
            .expect("the given time cannot be represented without overflow")
    }

    /// Create a new TimeSpec without checking for overflow or normalizing the values.
    ///
    /// Care should be taken to ensure `frac_secs` is in the range `[0, 1)` and is not
    /// NaN or infinite. Using invalid values may lead to unexpected results.
    pub const fn from_parts_unchecked(full_secs: i64, frac_secs: f64) -> Self {
        Self {
            full_secs,
            frac_secs,
        }
    }

    /// Tries to create a new TimeSpec using the number of full and fractional seconds.
    ///
    /// `frac_seconds` need not be in the range `[0, 1)`; it will be normalized
    /// to that range internally.
    ///
    /// Either or both values may be negative. The representation regardless
    /// is `full_secs + frac_secs`.
    ///
    /// `None` is returned if:
    /// - the given values cannot be represented without overflow
    /// - the given values are NaN or infinite
    pub fn try_from_parts(mut full_secs: i64, mut frac_secs: f64) -> Option<Self> {
        if frac_secs.is_nan() || frac_secs.is_infinite() {
            return None;
        }
        full_secs = full_secs.checked_add(frac_secs.trunc() as i64)?;
        frac_secs = frac_secs.fract();
        if frac_secs < 0.0 {
            frac_secs += 1.0;
            full_secs = full_secs.checked_sub(1)?;
        }
        Some(Self {
            full_secs,
            frac_secs,
        })
    }

    pub fn from_ticks(ticks: i64, tick_rate: f64) -> Self {
        let rate_i = tick_rate as i64;
        let rate_f = tick_rate - rate_i as f64;
        let secs_full = ticks / rate_i;
        let ticks_error = ticks - secs_full * rate_i;
        let ticks_frac = ticks_error as f64 - secs_full as f64 * rate_f;
        Self::from_parts(secs_full, ticks_frac / tick_rate)
    }

    pub fn from_secs(secs: i64) -> Self {
        Self::from_parts(secs, 0.0)
    }

    pub fn from_secs_f32(secs: f32) -> Self {
        Self::from_parts(0, secs as f64)
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self::from_parts(0, secs)
    }

    pub fn from_millis(millis: i64) -> Self {
        let full = millis / 1_000;
        let frac = (millis % 1_000) as f64 / 1_000.0;
        Self::from_parts(full, frac)
    }

    pub fn from_micros(micros: i64) -> Self {
        let full = micros / 1_000_000;
        let frac = (micros % 1_000_000) as f64 / 1_000_000.0;
        Self::from_parts(full, frac)
    }

    pub fn from_nanos(nanos: i64) -> Self {
        let full = nanos / 1_000_000_000;
        let frac = (nanos % 1_000_000_000) as f64 / 1_000_000_000.0;
        Self::from_parts(full, frac)
    }

    /// Get the number of full seconds in the TimeSpec.
    ///
    /// Note that for negative TimeSpecs the value may not be as expected.
    /// For example, `-0.3 s` is represnted as `full_secs = -1`, `frac_secs = 0.7`.
    pub fn full_secs(&self) -> i64 {
        self.full_secs
    }

    /// Get the number of fractional seconds in the TimeSpec.
    ///
    /// This will always be in the range `[0, 1)`.
    ///
    /// Note that for negative TimeSpecs the value may not be as expected.
    /// For example, `-0.3 s` is represnted as `full_secs = -1`, `frac_secs = 0.7`.
    pub const fn frac_secs(&self) -> f64 {
        self.frac_secs
    }
}

/// Conversions
impl TimeSpec {
    /// Convert the TimeSpec to the total number of seconds it represents.
    ///
    /// For large times the result may result in lowered precision.
    pub fn as_secs(&self) -> f64 {
        self.full_secs as f64 + self.frac_secs
    }

    /// Convert the TimeSpec to a [`Duration`].
    ///
    /// This will return `None` if the TimeSpec is negative.
    pub fn as_duration(&self) -> Option<Duration> {
        if self.is_negative() {
            None
        } else {
            let secs = u64::try_from(self.full_secs).ok()?;
            let nanos = (self.frac_secs as f64 * 1_000_000_000.0) as u32;
            Some(Duration::new(secs, nanos))
        }
    }

    /// Convert to clock ticks.
    pub fn to_ticks(&self, tick_rate: f64) -> i64 {
        let rate_i = tick_rate as i64;
        let rate_f = tick_rate - rate_i as f64;
        let ticks_full = self.full_secs * rate_i;
        let ticks_error = self.full_secs as f64 * rate_f;
        let ticks_frac = self.frac_secs * tick_rate;
        ticks_full + (ticks_error + ticks_frac).round() as i64
    }
}

/// Properties
impl TimeSpec {
    /// Check if the time represented by the TimeSpec is negative.
    pub const fn is_negative(self) -> bool {
        self.full_secs < 0
    }

    /// Check if the time represented by the TimeSpec is exactly zero.
    pub fn is_zero(self) -> bool {
        self.full_secs == 0 && self.frac_secs == 0.0
    }

    /// Get the sign of the time represented by the TimeSpec.
    ///
    /// The result is either `+1`, `-1`, or `0`.
    pub fn sign(self) -> i64 {
        if self.full_secs == 0 {
            // either +1 or 0
            self.frac_secs.signum() as i64
        } else {
            // either -1 or +1
            self.full_secs.signum()
        }
    }

    /// Get the absolute (non-negative) time represented by the TimeSpec.
    pub fn abs(self) -> Self {
        if self.is_negative() {
            -self
        } else {
            self
        }
    }
}

/// Math
impl TimeSpec {
    /// Add two TimeSpecs while checking for overflow/underflow.
    #[must_use]
    pub fn checked_add(self, rhs: TimeSpec) -> Option<Self> {
        let full_secs = self.full_secs.checked_add(rhs.full_secs)?;
        let frac_secs = self.frac_secs + rhs.frac_secs;
        Self::try_from_parts(full_secs, frac_secs)
    }

    /// Subtracts two TimeSpecs while checking for overflow/underflow.
    #[must_use]
    pub fn checked_sub(self, rhs: TimeSpec) -> Option<Self> {
        let full_secs = self.full_secs.checked_sub(rhs.full_secs)?;
        let frac_secs = self.frac_secs - rhs.frac_secs;
        Self::try_from_parts(full_secs, frac_secs)
    }

    /// Multiplies a TimeSpec while checking for overflow/underflow.
    #[must_use]
    pub fn checked_mul(self, rhs: f64) -> Option<Self> {
        let full_secs = self.full_secs as f64 * rhs;
        let frac_secs = self.frac_secs * rhs + full_secs.fract();
        if full_secs > i64::MAX as f64 || full_secs < i64::MIN as f64 {
            return None;
        }
        Self::try_from_parts(full_secs.trunc() as i64, frac_secs)
    }

    /// Multiplies a TimeSpec while checking for overflow/underflow and division by zero.
    #[must_use]
    pub fn checked_div(self, rhs: f64) -> Option<Self> {
        if rhs == 0.0 {
            return None;
        }
        let full_secs = self.full_secs as f64 / rhs;
        let frac_secs = self.frac_secs / rhs + full_secs.fract();
        if full_secs > i64::MAX as f64 || full_secs < i64::MIN as f64 {
            return None;
        }
        Self::try_from_parts(full_secs.trunc() as i64, frac_secs)
    }

    /// Divides two TimeSpecs while checking for division by zero.
    #[must_use]
    pub fn checked_div_timespec(self, rhs: TimeSpec) -> Option<f64> {
        if rhs.is_zero() {
            return None;
        }
        Some(self.as_secs() / rhs.as_secs())
    }
}

impl TryFrom<Duration> for TimeSpec {
    type Error = TimeError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        let full_secs = i64::try_from(value.as_secs()).or(Err(TimeError::Overflow))?;
        let frac_secs = value.subsec_nanos() as f64 / 1e9;
        TimeSpec::try_from_parts(full_secs, frac_secs).ok_or(TimeError::Overflow)
    }
}

impl Display for TimeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_secs as f64 + self.frac_secs)
    }
}

impl Neg for TimeSpec {
    type Output = TimeSpec;

    fn neg(self) -> Self::Output {
        TimeSpec::from_parts(-self.full_secs, -self.frac_secs)
    }
}

impl Add for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: TimeSpec) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow or underflow when adding time specs")
    }
}

impl AddAssign for TimeSpec {
    fn add_assign(&mut self, rhs: TimeSpec) {
        *self = *self + rhs;
    }
}

impl Sub for TimeSpec {
    type Output = TimeSpec;

    fn sub(self, rhs: TimeSpec) -> Self::Output {
        self.checked_sub(rhs)
            .expect("overflow or underflow when subtracting time specs")
    }
}

impl SubAssign for TimeSpec {
    fn sub_assign(&mut self, rhs: TimeSpec) {
        *self = *self - rhs;
    }
}

impl Div for TimeSpec {
    type Output = f64;

    fn div(self, rhs: TimeSpec) -> Self::Output {
        self.checked_div_timespec(rhs).expect("division by zero")
    }
}

macro_rules! mul_impl {
    ($t:ty) => {
        impl Mul<$t> for TimeSpec {
            type Output = TimeSpec;

            fn mul(self, rhs: $t) -> Self::Output {
                self.checked_mul(rhs as f64)
                    .expect("overflow during multiplication")
            }
        }

        impl MulAssign<$t> for TimeSpec {
            fn mul_assign(&mut self, rhs: $t) {
                *self = *self * rhs;
            }
        }
    };
}

macro_rules! div_impl {
    ($t:ty) => {
        impl Div<$t> for TimeSpec {
            type Output = TimeSpec;

            fn div(self, rhs: $t) -> Self::Output {
                self.checked_div(rhs as f64).expect("division by zero")
            }
        }

        impl DivAssign<$t> for TimeSpec {
            fn div_assign(&mut self, rhs: $t) {
                *self = *self / rhs;
            }
        }
    };
}

mul_impl!(i8);
mul_impl!(i16);
mul_impl!(i32);
mul_impl!(i64);
mul_impl!(isize);
mul_impl!(u8);
mul_impl!(u16);
mul_impl!(u32);
mul_impl!(u64);
mul_impl!(usize);
mul_impl!(f32);
mul_impl!(f64);

div_impl!(i8);
div_impl!(i16);
div_impl!(i32);
div_impl!(i64);
div_impl!(isize);
div_impl!(u8);
div_impl!(u16);
div_impl!(u32);
div_impl!(u64);
div_impl!(usize);
div_impl!(f32);
div_impl!(f64);

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_close_enough {
        ($a:expr, $b:literal) => {
            if ($a - $b).abs() > 1e-9 {
                panic!("expected {:?} is not close enough to {:?}", $a, $b);
            }
        };
        ($a:expr, $b:expr) => {
            if ($a - $b).as_secs().abs() > 1e-9 {
                panic!("expected {:?} is not close enough to {:?}", $a, $b);
            }
        };
    }

    #[test]
    fn timespec_macro() {
        assert_eq!(timespec!(0), TimeSpec::ZERO);
        assert_close_enough!(timespec!(0 s), TimeSpec::from_parts(0, 0.0));
        assert_close_enough!(timespec!(1 s), TimeSpec::from_parts(1, 0.0));
        assert_close_enough!(timespec!(1.5 s), TimeSpec::from_parts(1, 0.5));
        assert_close_enough!(timespec!(-0.5 s), TimeSpec::from_parts(-1, 0.5));
        assert_close_enough!(timespec!(-1234.5 ms), TimeSpec::from_parts(-2, 0.7655));
        assert_close_enough!(timespec!(1.5 us), TimeSpec::from_parts(0, 0.000_001_5));
        assert_close_enough!(timespec!(1.5 ns), TimeSpec::from_parts(0, 0.000_000_001_5));
    }

    #[test]
    fn addition() {
        assert_eq!(TimeSpec::MAX.checked_add(timespec!(1 ns)), None);
        assert_eq!(timespec!(0) + timespec!(0), timespec!(0));
        assert_close_enough!(timespec!(0.5 s) + timespec!(0.6 s), timespec!(1.1 s));
        assert_close_enough!(timespec!(0.5 s) + timespec!(-0.6 s), timespec!(-0.1 s));
        assert_close_enough!(timespec!(-0.5 s) + timespec!(0.6 s), timespec!(0.1 s));
        assert_close_enough!(timespec!(-0.5 s) + timespec!(-0.6 s), timespec!(-1.1 s));
    }

    #[test]
    fn subtraction() {
        assert_eq!(TimeSpec::MIN.checked_sub(timespec!(1 ns)), None);
        assert_eq!(timespec!(0) - timespec!(0), timespec!(0));
        assert_close_enough!(timespec!(0.5 s) - timespec!(0.6 s), timespec!(-0.1 s));
        assert_close_enough!(timespec!(0.5 s) - timespec!(-0.6 s), timespec!(1.1 s));
        assert_close_enough!(timespec!(-0.5 s) - timespec!(0.6 s), timespec!(-1.1 s));
        assert_close_enough!(timespec!(-0.5 s) - timespec!(-0.6 s), timespec!(0.1 s));
    }

    #[test]
    fn multiplication() {
        assert_eq!(TimeSpec::MAX.checked_mul(2.0), None);
        assert_eq!(TimeSpec::MIN.checked_mul(2.0), None);
        assert_eq!(timespec!(0) * 10.0, timespec!(0));
        assert_close_enough!(timespec!(0.5 s) * 2u32, timespec!(1.0 s));
        assert_close_enough!(timespec!(0.5 s) * -2.5f64, timespec!(-1.25 s));
        assert_close_enough!(timespec!(-1.5 s) * 3i8, timespec!(-4.5 s));
    }

    #[test]
    fn division_by_number() {
        assert_eq!(timespec!(0.5 s).checked_div(0.0), None);
        assert_eq!(timespec!(0) / 10.0, timespec!(0));
        assert_close_enough!(timespec!(0.5 s) / 2u32, timespec!(250 ms));
        assert_close_enough!(timespec!(5 s) / -2.5f64, timespec!(-2 s));
        assert_close_enough!(timespec!(-1.5 s) / 3i8, timespec!(-0.5 s));
    }

    #[test]
    fn division_by_timespec() {
        assert_eq!(timespec!(0.5 s).checked_div_timespec(timespec!(0)), None);
        assert_eq!(timespec!(0) / timespec!(2 s), 0.0);
        assert_close_enough!(timespec!(0.5 s) / timespec!(2 s), 0.25);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", timespec!(0 s)), "0");
        assert_eq!(format!("{}", timespec!(-1.5 s)), "-1.5");
        assert_eq!(format!("{}", timespec!(0.123456789 s)), "0.123456789");
    }
}
