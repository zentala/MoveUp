//! tray_blink.rs — Tray icon dot blinking engine.
//!
//! Plays blink patterns: [on_ms, off_ms] × count, then pause_ms, repeat.
//! Example "3×/10s": 250ms on, 250ms off, ×3, then 8500ms pause.
//!
//! Call [`TrayBlinker::tick`] from a ~50ms interval timer thread.
//! When active, the blinker owns a dedicated background thread that fires
//! a callback to update the tray icon.

/// A blink pattern definition.
///
/// Describes a burst of `count` blinks (each `on_ms` on + `off_ms` off),
/// followed by a `pause_ms` gap before the next burst.
#[derive(Debug, Clone, PartialEq)]
pub struct BlinkPattern {
    /// Duration the dot is visible (ms).
    pub on_ms: u64,
    /// Duration the dot is hidden between blinks in a burst (ms).
    pub off_ms: u64,
    /// Number of blinks per burst.
    pub count: u32,
    /// Gap between bursts (ms).
    pub pause_ms: u64,
}

/// Current phase within the blink cycle.
#[derive(Debug, Clone, PartialEq)]
enum BlinkPhase {
    /// Dot visible — timing on_ms.
    On,
    /// Dot hidden — between blinks within a burst.
    Off,
    /// Dot hidden — gap between bursts.
    Pause,
}

/// Manages tray icon dot blinking state.
///
/// Call [`tick`] with elapsed milliseconds to advance the animation.
/// Returns `true` when the dot should be visible, `false` when hidden.
pub struct TrayBlinker {
    active: bool,
    pattern: BlinkPattern,
    phase: BlinkPhase,
    /// Current blink index within the burst (0..count).
    step: u32,
    /// Time accumulated in the current phase (ms).
    phase_elapsed_ms: u64,
}

impl TrayBlinker {
    /// Create a new inactive blinker with a placeholder pattern.
    pub fn new() -> Self {
        Self {
            active: false,
            pattern: BlinkPattern {
                on_ms: 250,
                off_ms: 250,
                count: 3,
                pause_ms: 8500,
            },
            phase: BlinkPhase::Pause,
            step: 0,
            phase_elapsed_ms: 0,
        }
    }

    /// Start a new blink pattern. Replaces any active pattern and resets state.
    pub fn start(&mut self, pattern: BlinkPattern) {
        self.pattern = pattern;
        self.phase = BlinkPhase::On;
        self.step = 0;
        self.phase_elapsed_ms = 0;
        self.active = true;
    }

    /// Stop blinking immediately. Dot becomes hidden.
    pub fn stop(&mut self) {
        self.active = false;
        self.phase = BlinkPhase::Pause;
        self.step = 0;
        self.phase_elapsed_ms = 0;
    }

    /// Whether the blinker is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advance time by `delta_ms` milliseconds. Returns whether the dot should be visible.
    ///
    /// Call this on a timer at ~50ms intervals for smooth blink animation.
    pub fn tick(&mut self, delta_ms: u64) -> bool {
        if !self.active {
            return false;
        }

        self.phase_elapsed_ms += delta_ms;

        match self.phase {
            BlinkPhase::On => {
                if self.phase_elapsed_ms >= self.pattern.on_ms {
                    self.phase_elapsed_ms = 0;
                    self.phase = BlinkPhase::Off;
                }
                true
            }
            BlinkPhase::Off => {
                if self.phase_elapsed_ms >= self.pattern.off_ms {
                    self.phase_elapsed_ms = 0;
                    self.step += 1;
                    if self.step >= self.pattern.count {
                        self.step = 0;
                        self.phase = BlinkPhase::Pause;
                    } else {
                        self.phase = BlinkPhase::On;
                    }
                }
                false
            }
            BlinkPhase::Pause => {
                if self.phase_elapsed_ms >= self.pattern.pause_ms {
                    self.phase_elapsed_ms = 0;
                    self.phase = BlinkPhase::On;
                }
                false
            }
        }
    }
}

impl Default for TrayBlinker {
    fn default() -> Self {
        Self::new()
    }
}
