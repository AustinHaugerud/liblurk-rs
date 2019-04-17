use std::time::{Duration, Instant};

pub struct Clock {
    last_time: Instant,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            last_time: Instant::now(),
        }
    }

    pub fn get_elapsed(&mut self) -> Duration {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();
        elapsed
    }
}
