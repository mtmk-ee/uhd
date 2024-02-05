use std::{ops::Add, time::Duration};

#[derive(Clone, Copy, Debug)]
pub struct DeviceTime(Duration);

impl DeviceTime {
    pub const ZERO: DeviceTime = DeviceTime(Duration::ZERO);

    pub fn new(seconds: f64) -> Self {
        DeviceTime(Duration::from_secs_f64(seconds))
    }

    pub const fn from_duration(duration: Duration) -> Self {
        DeviceTime(duration)
    }

    pub fn from_parts(full_seconds: u64, fractional_seconds: f64) -> Self {
        DeviceTime(Duration::from_secs(full_seconds) + Duration::from_secs_f64(fractional_seconds))
    }

    pub fn seconds(&self) -> f64 {
        self.0.as_secs_f64()
    }

    pub fn full_seconds(&self) -> u64 {
        self.0.as_secs()
    }

    pub fn fractional_seconds(&self) -> f64 {
        self.0.as_secs_f64() - self.0.as_secs() as f64
    }
}

impl Add<Duration> for DeviceTime {
    type Output = DeviceTime;

    fn add(self, rhs: Duration) -> Self::Output {
        DeviceTime(self.0 + rhs)
    }
}
