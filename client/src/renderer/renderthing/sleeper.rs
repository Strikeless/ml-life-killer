use std::time::{Duration, Instant};

pub struct Sleeper {
    pub target_delta_time: Duration,
    pub last_instant: Option<Instant>,
}

impl Sleeper {
    pub fn new(target_delta_time: Duration) -> Self {
        Self {
            target_delta_time,
            last_instant: None,
        }
    }

    pub fn sleep(&mut self) -> bool {
        let this_instant = Instant::now();
        self.last_instant = Some(this_instant);

        let slept = if !self.in_time() {
            false
        } else {
            // SAFETY: We will never be in time if last_instant is None, thus unwrap should never be a problem here.
            let delta_time = this_instant - self.last_instant.unwrap();

            if self.target_delta_time > delta_time {
                let draw_target_delta = self.target_delta_time - delta_time;
                spin_sleep::sleep(draw_target_delta);

                true
            } else {
                false
            }
        };

        self.last_instant = Some(this_instant);
        slept
    }

    pub fn in_time(&self) -> bool {
        if let Some(last_instant) = self.last_instant {
            let this_instant = Instant::now();
            let delta_time = this_instant - last_instant;

            self.target_delta_time > delta_time
        } else {
            // Assume we're late if we've never slept yet.
            false
        }
    }
}
