extern crate chrono;

use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

pub fn get_elapsed_seconds(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + (0.000_000_001 * f64::from(duration.subsec_nanos()))
}

pub(crate) struct Clock {
    instant: Instant,
    manual_duration: Option<Duration>,
    manual_now: Option<DateTime<Utc>>,
}

impl Clock {
    pub(crate) fn new() -> Self {
        Clock {
            instant: Instant::now(),
            manual_duration: None,
            manual_now: None,
        }
    }
    #[inline]
    pub(crate) fn now(&self) -> DateTime<Utc> {
        match self.manual_now {
            Some(x) => x,
            None => Utc::now(),
        }
    }
    #[inline]
    pub(crate) fn elapsed(&self) -> Duration {
        match self.manual_duration {
            Some(d) => d,
            None => self.instant.elapsed(),
        }
    }
    #[cfg(test)]
    pub(crate) fn set_manual_duration(&mut self, duration: Duration) {
        self.manual_duration = Some(duration);
    }
    #[cfg(test)]
    pub(crate) fn set_manual_now(&mut self, now: DateTime<Utc>) {
        self.manual_now = Some(now);
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
