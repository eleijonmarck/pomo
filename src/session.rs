use fmt::{Display, Formatter};
use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};

use confy;
mod config;

lazy_static! {
    pub static ref SESSION_DURATIONS: HashMap<SessionMode, (u64, u64, u64)> = {
        let mut durations: HashMap<SessionMode, (u64, u64, u64)> = HashMap::new();

        // ugly but working way of using the config
        let cfg: config::PomoConfig = confy::load("pomo").expect("yo, config not working");

        durations.insert(SessionMode::LongSession, (0, cfg.long_session_minutes, 0));
        durations.insert(SessionMode::ShortBreak, (0, cfg.short_break_minutes, 0));
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

impl Display for SessionMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub trait IntoRepresentation {
    fn into_representation(self) -> String;
}

impl IntoRepresentation for Duration {
    fn into_representation(self) -> String {
        let s = self.as_secs();
        let (hours, minutes, seconds) = (s / 3600, (s % 3600) / 60, s % 60);
        if hours == 0 {
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
    }
}

#[derive(Debug)]
pub struct Session {
    pub duration: Duration,
    pub mode: SessionMode,

    elapsed_time: Instant,
    paused_time: Option<Instant>,
}

impl Session {
    pub fn new(mode: SessionMode) -> Session {
        let (h, m, s) = SESSION_DURATIONS[&mode];
        Session {
            duration: Duration::new(3600 * h + 60 * m + s, 0),
            mode,
            elapsed_time: Instant::now(),
            paused_time: None,
        }
    }

    pub fn is_ended(&self) -> bool {
        self.duration < self.elapsed_time.elapsed()
    }

    pub fn is_paused(&self) -> bool {
        self.paused_time.is_some()
    }

    pub fn remaining(&self) -> Duration {
        self.duration - self.elapsed_time.elapsed()
    }

    pub fn toggle_pause(&mut self) {
        if let Some(paused) = self.paused_time {
            if let Some(elapsed_sum) = self.elapsed_time.checked_add(paused.elapsed()) {
                self.elapsed_time = elapsed_sum
            }
            self.paused_time = None;
        } else {
            self.paused_time = Some(Instant::now());
        }
    }
}
