//! gap_handler.rs — Detects time gaps on app restart and applies appropriate state changes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of gap detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GapResult {
    /// Gap < 5 min — assume last state continued.
    Continue { gap_secs: i64 },
    /// Gap 5-10 min — treat as short Away break.
    ShortBreak { gap_secs: i64 },
    /// Gap >= 10 min — treat as long break / full reset.
    LongBreak { gap_secs: i64 },
    /// No previous timestamp — first launch or data lost.
    NoPreviousData,
}

/// Detects the gap between last tick and now, returns appropriate handling.
pub fn detect_gap(last_tick_ts: Option<DateTime<Utc>>) -> GapResult {
    let now = Utc::now();
    match last_tick_ts {
        None => GapResult::NoPreviousData,
        Some(last) => {
            let gap_secs = (now - last).num_seconds().max(0);
            if gap_secs < 300 {
                GapResult::Continue { gap_secs }
            } else if gap_secs < 600 {
                GapResult::ShortBreak { gap_secs }
            } else {
                GapResult::LongBreak { gap_secs }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn no_previous_data() {
        assert_eq!(detect_gap(None), GapResult::NoPreviousData);
    }

    #[test]
    fn continue_short_gap() {
        let ts = Utc::now() - Duration::seconds(60);
        match detect_gap(Some(ts)) {
            GapResult::Continue { gap_secs } => {
                assert!(gap_secs >= 55 && gap_secs <= 65, "gap_secs={gap_secs}");
            }
            other => panic!("Expected Continue, got {:?}", other),
        }
    }

    #[test]
    fn short_break_gap() {
        let ts = Utc::now() - Duration::seconds(400);
        match detect_gap(Some(ts)) {
            GapResult::ShortBreak { gap_secs } => {
                assert!(gap_secs >= 395 && gap_secs <= 405, "gap_secs={gap_secs}");
            }
            other => panic!("Expected ShortBreak, got {:?}", other),
        }
    }

    #[test]
    fn long_break_gap() {
        let ts = Utc::now() - Duration::seconds(700);
        match detect_gap(Some(ts)) {
            GapResult::LongBreak { gap_secs } => {
                assert!(gap_secs >= 695 && gap_secs <= 705, "gap_secs={gap_secs}");
            }
            other => panic!("Expected LongBreak, got {:?}", other),
        }
    }
}
