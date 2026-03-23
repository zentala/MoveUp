//! height_stabilizer.rs — Smooths raw sensor readings for stable UI display.
//!
//! Three algorithms work together:
//! 1. **Moving average** — averages last N readings for smooth display
//! 2. **Round to cm** — converts mm to nearest whole cm for UI
//! 3. **Trend lock** — locks display when readings are stable, unlocks on significant change

use std::collections::VecDeque;

/// Default number of readings for the moving average window.
const DEFAULT_WINDOW_SIZE: usize = 10;

/// Tolerance in mm for trend lock: readings within this range count as "stable".
const TREND_LOCK_TOLERANCE_MM: f32 = 3.0;

/// Number of stable readings required before locking the display value.
const TREND_LOCK_COUNT: usize = 5;

/// Deviation in mm that unlocks a locked display value.
const TREND_UNLOCK_DEVIATION_MM: f32 = 5.0;

/// Smooths raw sensor readings for flicker-free UI display.
///
/// Feed raw mm readings via [`push`]; read the stabilized cm value via [`stabilized_cm`].
/// The state machine should still use raw values for its own debounce logic.
pub struct HeightStabilizer {
    /// Circular buffer of recent raw readings (in mm).
    buffer: VecDeque<i32>,
    /// Maximum number of readings to keep.
    window_size: usize,
    /// Locked display value in mm (when trend is stable).
    locked_mm: Option<f32>,
    /// Count of consecutive readings within tolerance of the current average.
    stable_count: usize,
}

impl HeightStabilizer {
    /// Creates a new stabilizer with the default window size.
    pub fn new() -> Self {
        Self::with_window_size(DEFAULT_WINDOW_SIZE)
    }

    /// Creates a new stabilizer with a custom window size.
    pub fn with_window_size(window_size: usize) -> Self {
        let size = window_size.max(1);
        Self {
            buffer: VecDeque::with_capacity(size),
            window_size: size,
            locked_mm: None,
            stable_count: 0,
        }
    }

    /// Pushes a raw sensor reading (in mm) into the stabilizer.
    pub fn push(&mut self, mm: i32) {
        if self.buffer.len() >= self.window_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(mm);
        self.update_trend_lock();
    }

    /// Returns the stabilized height in whole centimeters (rounded).
    ///
    /// Uses trend-locked value if stable, otherwise the moving average.
    /// Returns `None` if no readings have been pushed yet.
    pub fn stabilized_cm(&self) -> Option<u32> {
        let avg_mm = self.average_mm()?;
        let effective_mm = self.locked_mm.unwrap_or(avg_mm);
        Some(mm_to_cm_rounded(effective_mm))
    }

    /// Returns the stabilized desk height in whole cm, accounting for desk thickness.
    ///
    /// Subtracts `desk_thickness_cm` from the stabilized floor distance before rounding.
    /// Returns `None` if no readings have been pushed yet.
    pub fn stabilized_height_cm(&self, desk_thickness_cm: f32) -> Option<f32> {
        let avg_mm = self.average_mm()?;
        let effective_mm = self.locked_mm.unwrap_or(avg_mm);
        let height_mm = effective_mm - desk_thickness_cm * 10.0;
        Some(mm_to_cm_rounded(height_mm) as f32)
    }

    /// Returns the moving average in mm, or `None` if the buffer is empty.
    pub fn average_mm(&self) -> Option<f32> {
        if self.buffer.is_empty() {
            return None;
        }
        let sum: i64 = self.buffer.iter().map(|&v| v as i64).sum();
        Some(sum as f32 / self.buffer.len() as f32)
    }

    /// Returns whether the trend lock is currently engaged.
    pub fn is_locked(&self) -> bool {
        self.locked_mm.is_some()
    }

    /// Resets all internal state (e.g., on device reconnect).
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.locked_mm = None;
        self.stable_count = 0;
    }

    /// Updates the trend lock based on the latest reading vs. current average.
    fn update_trend_lock(&mut self) {
        let avg = match self.average_mm() {
            Some(a) => a,
            None => return,
        };

        let latest = match self.buffer.back() {
            Some(&v) => v as f32,
            None => return,
        };

        // Check if the latest reading deviates enough to unlock.
        if let Some(locked) = self.locked_mm {
            if (latest - locked).abs() > TREND_UNLOCK_DEVIATION_MM {
                self.locked_mm = None;
                self.stable_count = 0;
                return;
            }
        }

        // Check stability: is the latest reading within tolerance of the average?
        if (latest - avg).abs() <= TREND_LOCK_TOLERANCE_MM {
            self.stable_count += 1;
        } else {
            self.stable_count = 0;
            // Also unlock if we were locked and readings are shifting.
            if self.locked_mm.is_some() {
                self.locked_mm = None;
            }
        }

        // Engage lock after enough stable readings.
        if self.stable_count >= TREND_LOCK_COUNT && self.locked_mm.is_none() {
            self.locked_mm = Some(avg);
        }
    }
}

impl Default for HeightStabilizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts mm to the nearest whole cm using standard rounding.
///
/// Examples: 721mm -> 72cm, 725mm -> 73cm (round half up), 719mm -> 72cm.
pub fn mm_to_cm_rounded(mm: f32) -> u32 {
    (mm / 10.0).round().max(0.0) as u32
}

#[cfg(test)]
mod tests;
