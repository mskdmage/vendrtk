use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    max_attempts: u32,
    interval: Duration,
}

impl Config {
    pub fn new(max_attempts: u32, interval: Duration) -> Self {
        Self {
            max_attempts,
            interval,
        }
    }

    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_attempts: 120,
            interval: Duration::from_secs(1),
        }
    }
}