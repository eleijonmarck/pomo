use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PomoConfig {
    pub long_session_minutes: u64,
    pub short_break_minutes: u64,
    pub long_break_minutes: u64,
}

impl Default for PomoConfig {
    fn default() -> Self {
        PomoConfig {
            long_session_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
        }
    }
}
