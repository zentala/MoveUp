//! activity.rs — System idle-time detection.
//!
//! Uses the Windows `GetLastInputInfo` API to determine how long the user
//! has been idle (no keyboard or mouse input). Falls back to treating the
//! user as active on non-Windows platforms.

/// Threshold below which a user is considered active (seconds).
const ACTIVE_THRESHOLD_SECS: u64 = 60;

/// Returns the number of seconds since the last keyboard or mouse input.
///
/// On Windows this calls `GetLastInputInfo` via the `windows` crate.
/// On other platforms it always returns 0 (always active).
pub fn get_idle_seconds() -> u64 {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
        use windows::Win32::System::SystemInformation::GetTickCount;

        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        // SAFETY: lii is properly initialised with the correct cbSize.
        let ok = unsafe { GetLastInputInfo(&mut lii) };
        if !ok.as_bool() {
            return 0;
        }

        // GetTickCount wraps at ~49.7 days; the subtraction naturally handles
        // the wrapping via u32 arithmetic (then we widen to u64).
        let tick_now = unsafe { GetTickCount() };
        let elapsed_ms = tick_now.wrapping_sub(lii.dwTime) as u64;
        elapsed_ms / 1000
    }

    #[cfg(not(windows))]
    {
        0
    }
}

/// Returns `true` if the user has been active within the last 60 seconds.
pub fn is_active() -> bool {
    get_idle_seconds() < ACTIVE_THRESHOLD_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_seconds_returns_u64() {
        // Just ensure the call does not panic and returns a sane value.
        let secs = get_idle_seconds();
        // On a fresh test runner the machine should have had recent input.
        // We only assert it doesn't overflow or panic.
        let _ = secs;
    }

    #[test]
    fn is_active_consistent_with_idle_seconds() {
        let idle = get_idle_seconds();
        let active = is_active();
        if idle < 60 {
            assert!(active);
        } else {
            assert!(!active);
        }
    }
}
