use tauri::WebviewWindow;

/// Apply capture exclusion / stealth protection to a window so screen share tools
/// (Zoom, Teams, Google Meet, OBS, Discord) cannot capture it.
pub fn apply_stealth_to_window(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        // On macOS: [NSWindow setSharingType:0] (NSWindowSharingNone)
        // Invoked directly via the macOS Objective-C runtime ABI
        if let Ok(ns_win) = window.ns_window() {
            unsafe {
                unsafe extern "C" {
                    fn sel_registerName(name: *const std::ffi::c_char) -> *const std::ffi::c_void;
                    fn objc_msgSend(receiver: *mut std::ffi::c_void, sel: *const std::ffi::c_void, ...) -> *mut std::ffi::c_void;
                }
                let sel = sel_registerName(b"setSharingType:\0".as_ptr() as *const _);
                let _: *mut std::ffi::c_void = objc_msgSend(ns_win as *mut _, sel, 0isize);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows: SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE = 0x00000011)
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                unsafe extern "system" {
                    fn SetWindowDisplayAffinity(hwnd: *mut std::ffi::c_void, affinity: u32) -> i32;
                }
                const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
                let _ = SetWindowDisplayAffinity(hwnd.0 as *mut std::ffi::c_void, WDA_EXCLUDEFROMCAPTURE);
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
    }
}
