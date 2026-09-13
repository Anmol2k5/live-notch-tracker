use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[tauri::command]
pub fn get_work_area(app: AppHandle) -> Result<WorkArea, String> {
    let window = app
        .get_webview_window("notch")
        .ok_or_else(|| "notch window not found".to_string())?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no monitor for notch window".to_string())?;
    let area = monitor.work_area();
    Ok(WorkArea {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width,
        height: area.size.height,
        scale_factor: monitor.scale_factor(),
    })
}

/// Apply Win32 extended styles to the notch window at launch:
/// - `WS_EX_TOOLWINDOW`: hides the window from the Alt-Tab switcher
/// - `WS_EX_TRANSPARENT | WS_EX_LAYERED`: makes the window click-through
///   (clicks in transparent margins pass to whatever is behind). `WS_EX_LAYERED`
///   is required alongside `WS_EX_TRANSPARENT` on Windows — `TRANSPARENT` alone
///   does not make `WindowFromPoint` skip the window (verified with a live
///   production build; `TRANSPARENT` alone left margin hits on the WebView,
///   `TRANSPARENT|LAYERED` makes them hit the window behind).
///
/// Called from the Tauri `setup` hook in `lib.rs` and again after the window
/// is shown (Tauri may reset exstyle on `show()`).
#[cfg(windows)]
pub fn apply_notch_styles(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    apply_notch_styles_inner(app.get_webview_window("notch"))
}

#[cfg(windows)]
fn apply_notch_styles_inner(
    window: Option<tauri::WebviewWindow>,
) -> Result<(), Box<dyn std::error::Error>> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SET_WINDOW_POS_FLAGS,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_LAYERED,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };

    let window = window.ok_or("notch window not found during setup")?;
    let handle = window.window_handle()?.as_raw();
    if let RawWindowHandle::Win32(win32) = handle {
        let hwnd = HWND(win32.hwnd.get() as *mut _);
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_style = style
                | (WS_EX_TOOLWINDOW.0 as isize)
                | (WS_EX_TRANSPARENT.0 as isize)
                | (WS_EX_LAYERED.0 as isize);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SET_WINDOW_POS_FLAGS(SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOZORDER.0 | SWP_FRAMECHANGED.0),
            );
        }
    }
    Ok(())
}

#[cfg(windows)]
#[tauri::command]
pub fn apply_notch_styles_cmd(app: tauri::AppHandle) -> Result<(), String> {
    apply_notch_styles_inner(app.get_webview_window("notch")).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
pub fn apply_notch_styles(_app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn apply_notch_styles_cmd(_app: tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

/// Snap a logical-pixel length to whole physical pixels for `scale_factor`,
/// expressed back in logical pixels. The OS places windows on physical pixels,
/// so a fractional physical edge gets rounded by the compositor and the notch
/// floats a hairline off the screen edge (the macOS 0.7pt flush-gap bug).
#[allow(dead_code)] // No Rust call-site yet: the live path rounds in TS (notchGeometry.ts). Kept as the tested reference implementation; a Rust call-site lands with runtime re-anchor work.
pub fn round_to_physical(logical: f64, scale_factor: f64) -> f64 {
    (logical * scale_factor).round() / scale_factor
}

#[cfg(test)]
mod tests {
    use super::round_to_physical;

    #[test]
    fn whole_physical_pixels_at_125_percent() {
        let rounded = round_to_physical(70.0, 1.25);
        assert_eq!((rounded * 1.25).round(), 88.0);
    }

    #[test]
    fn whole_physical_pixels_at_150_percent() {
        let rounded = round_to_physical(70.0, 1.5);
        assert_eq!((rounded * 1.5).round(), 105.0);
    }

    #[test]
    fn identity_at_100_percent() {
        assert_eq!(round_to_physical(70.0, 1.0), 70.0);
    }
}
