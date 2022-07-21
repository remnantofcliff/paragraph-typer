use std::time::{Duration, Instant};

pub struct Timer {
    start_time: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
