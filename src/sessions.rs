use std::{collections::HashMap, fmt, time};

lazy_static! {
    pub static ref SESSION_DURATIONS: HashMap<SessionMode, (u64, u64, u64)> = {
        let mut durations: HashMap<SessionMode, (u64, u64, u64)> = HashMap::new();
        durations.insert(SessionMode::LongSession, (0, 0, 5));
        durations.insert(SessionMode::ShortBreak, (0, 5, 0));
        durations.insert(SessionMode::LongBreak, (0, 15, 0));
        durations
    };
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum SessionMode {
    LongSession,
    ShortBreak,
    LongBreak,
}

impl fmt::Display for SessionMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct Session {
    pub duration: time::Duration,
    pub mode: SessionMode,

    elapsed_time: time::Instant,
    paused_time: Option<time::Instant>,
}

impl Session {
    pub fn init(mode: SessionMode) -> Session {
        let (h, m, s) = SESSION_DURATIONS[&mode];
        Session {
            duration: time::Duration::new(3600 * h + 60 * m + s, 0),
            mode,
            elapsed_time: time::Instant::now(),
            paused_time: None,
        }
    }

    pub fn is_ended(&self) -> bool {
        self.duration < self.elapsed_time.elapsed()
    }

    pub fn is_paused(&self) -> bool {
        self.paused_time.is_some()
    }

    pub fn remaining(&self) -> time::Duration {
        self.duration - self.elapsed_time.elapsed()
    }

    pub fn toggle_pause(&mut self) {
        if let Some(paused) = self.paused_time {
            if let Some(elapsed_sum) = self.elapsed_time.checked_add(paused.elapsed()) {
                self.elapsed_time = elapsed_sum
            }
            self.paused_time = None;
        } else {
            self.paused_time = Some(time::Instant::now());
        }
    }
}
