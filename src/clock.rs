use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

pub fn get_elapsed_seconds(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + (0.000_000_001 * f64::from(duration.subsec_nanos()))
}

pub(crate) struct Clock {
    instant: Instant,
    duration_override: Option<Duration>,
    now_override: Option<DateTime<Utc>>,
}

impl Clock {
    pub(crate) fn new() -> Self {
        Clock {
            instant: Instant::now(),
            duration_override: None,
            now_override: None,
        }
    }
    #[inline]
    pub(crate) fn now(&self) -> DateTime<Utc> {
        match self.now_override {
            Some(x) => x,
            None => Utc::now(),
        }
    }
    #[inline]
    pub(crate) fn elapsed(&self) -> Duration {
        match self.duration_override {
            Some(d) => d,
            None => self.instant.elapsed(),
        }
    }
    #[cfg(test)]
    pub(crate) fn set_duration_override(&mut self, duration: Duration) {
        self.duration_override = Some(duration);
    }
    #[cfg(test)]
    pub(crate) fn set_now_override(&mut self, now: DateTime<Utc>) {
        self.now_override = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elapsed_whole_seconds() {
        let d = Duration::new(5, 0);
        let result = get_elapsed_seconds(&d);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_elapsed_fractional_seconds() {
        let d = Duration::new(42, 123);
        let result = get_elapsed_seconds(&d);
        assert_eq!(result, 42.000000123);
    }
}
