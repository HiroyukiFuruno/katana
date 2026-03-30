/* WHY: OS-level theme detection.

Detects whether the operating system is running in dark mode.
Currently only macOS is supported.  Other platforms return `None`. */

/* WHY: Returns `Some(true)` if the OS is in dark mode, `Some(false)` for light mode,
and `None` if the platform does not provide theme information.

Detection uses the `NSApp.effectiveAppearance` API on macOS (Objective-C runtime). */
pub fn is_dark_mode() -> Option<bool> {
    detect_dark_mode_impl()
}

// WHY: ── Platform implementations ──────────────────────────────────────────

#[cfg(target_os = "macos")]
fn detect_dark_mode_impl() -> Option<bool> {
    /* WHY: Query NSApplication.effectiveAppearance via the Objective-C runtime.
    The appearance name contains "Dark" for dark mode variants. */
    use std::ffi::CStr;
    let name = unsafe { katana_macos_appearance_name() };
    if name.is_null() {
        return None;
    }
    // SAFETY: katana_macos_appearance_name returns a valid C string or NULL.
    let cstr = unsafe { CStr::from_ptr(name) };
    let appearance = cstr.to_string_lossy();
    Some(appearance.contains("Dark"))
}

#[cfg(not(target_os = "macos"))]
fn detect_dark_mode_impl() -> Option<bool> {
    // WHY: Dark mode detection is not supported on non-macOS platforms.
    None
}

// WHY: ── macOS FFI ─────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
extern "C" {
    /* WHY: Returns the name of the NSApplication effective appearance as a C string.

    # Safety
    Returns a valid, null-terminated UTF-8 C string, or NULL if the runtime call fails.
    The returned pointer is valid only for the lifetime of the current autorelease pool. */
    fn katana_macos_appearance_name() -> *const std::ffi::c_char;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_some_bool_or_none_on_macos() {
        let result = is_dark_mode();
        /* WHY: On macOS: result is Some(true/false) when NSApp is available, or None when
        running under a test binary (no NSApplication main loop started).
        On other platforms: always None. */
        #[cfg(not(target_os = "macos"))]
        assert!(result.is_none(), "non-macOS must return None");
        #[cfg(target_os = "macos")]
        // WHY: Either None (test env, no NSApp) or Some(bool) (running inside the app).
        let _ = result; // WHY: any value is acceptable in test environment
    }
}