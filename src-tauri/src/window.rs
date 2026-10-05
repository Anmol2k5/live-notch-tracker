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
///   `WS_EX_TRANSPARENT` is intentionally NOT set — M6a uses `SetWindowRgn`
///   for per-pixel hit-testing so the silhouette stays interactive while the
///   transparent margin passes through. Previously `TRANSPARENT|LAYERED` made
///   the whole window click-through.
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
            // M6a: remove TRANSPARENT entirely — region handles per-pixel hit-testing.
            // Keep TOOLWINDOW, clear both TRANSPARENT and LAYERED (region works with or without
            // LAYERED; keeping LAYERED caused the earlier whole-window click-through and can
            // interfere with SetWindowRgn on some DWM configurations). LAYERED is not needed
            // for a shaped window with per-pixel region.
            let new_style = (style | (WS_EX_TOOLWINDOW.0 as isize))
                & !(WS_EX_TRANSPARENT.0 as isize)
                & !(WS_EX_LAYERED.0 as isize);
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

// ---------------------------------------------------------------------------
// M6a: Win32 region from notchPath — makes the window itself the silhouette
// ---------------------------------------------------------------------------

/// Returns the notch silhouette polygon in logical pixels, matching
/// `src/components/SideNotchShape.tsx`'s `notchPath(width,height)` exactly.
///
/// Order: top (edgeX,0) → bottom (edgeX,height) → innerBottom → bezier samples
/// → innerTop → back to top. The bezier is the `Q` segment from innerBottom
/// to innerTop via control (0, height/2).
pub fn notch_polygon_logical(width: f64, height: f64) -> Vec<(f64, f64)> {
    let edge_x = width;
    // TS: Math.min(38.5 * (44/117)*(117/44), width, height/2) → 38.5
    let curl = 38.5_f64.min(width).min(height / 2.0);
    let flare = (width * 0.55).min(curl);
    let inner_top = (flare * 0.2, flare);
    let inner_bottom = (flare * 0.2, height - flare);
    let apex = (edge_x - width, height / 2.0); // control point, also sample at t=0.5 if needed
    let _ = apex; // keep for clarity, actual bezier uses (0, h/2) as control
    let top = (edge_x, 0.0);
    let bottom = (edge_x, height);

    let mut pts: Vec<(f64, f64)> = Vec::new();
    pts.push(top);
    pts.push(bottom);
    pts.push(inner_bottom);
    // Sample quadratic bezier from innerBottom → innerTop via (0, h/2)
    let p0 = inner_bottom;
    let c = (edge_x - width, height / 2.0); // (0, h/2)
    let p2 = inner_top;
    // Increase density for smooth curve — 32 segments gives no visible faceting
    let steps = 32usize;
    for i in 1..steps {
        let t = i as f64 / steps as f64;
        let inv = 1.0 - t;
        let x = inv * inv * p0.0 + 2.0 * inv * t * c.0 + t * t * p2.0;
        let y = inv * inv * p0.1 + 2.0 * inv * t * c.1 + t * t * p2.1;
        pts.push((x, y));
    }
    pts.push(inner_top);
    // Close implicitly — CreatePolygonRgn closes to first point
    pts
}

/// Convert logical polygon to physical-pixel integer points for `CreatePolygonRgn`.
/// Uses the same `roundToPhysical` semantics as `notchGeometry.ts` and `round_to_physical`.
pub fn notch_polygon_physical(
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
) -> Vec<(i32, i32)> {
    notch_polygon_logical(logical_width, logical_height)
        .into_iter()
        .map(|(x, y)| {
            let px = (x * scale_factor).round() as i32;
            let py = (y * scale_factor).round() as i32;
            (px, py)
        })
        .collect()
}

#[cfg(windows)]
fn apply_notch_region_inner(
    window: Option<tauri::WebviewWindow>,
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::Graphics::Gdi::{
        CreatePolygonRgn, DeleteObject, SetWindowRgn, CREATE_POLYGON_RGN_MODE,
    };

    let window = window.ok_or("notch window not found for SetWindowRgn")?;
    let handle = window.window_handle()?.as_raw();
    if let RawWindowHandle::Win32(win32) = handle {
        let hwnd = HWND(win32.hwnd.get() as *mut _);
        let sf = if scale_factor > 0.0 && scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        let physical_pts = notch_polygon_physical(logical_width, logical_height, sf);
        if physical_pts.len() < 3 {
            return Err("notch polygon has <3 points".into());
        }
        let points: Vec<POINT> = physical_pts
            .into_iter()
            .map(|(x, y)| POINT { x, y })
            .collect();

        unsafe {
            // ALTERNATE fill mode = 1
            let hrgn = CreatePolygonRgn(&points, CREATE_POLYGON_RGN_MODE(1));
            if hrgn.is_invalid() {
                return Err("CreatePolygonRgn failed".into());
            }
            let ok = SetWindowRgn(hwnd, Some(hrgn), true);
            if ok == 0 {
                let _ = DeleteObject(hrgn.into());
                return Err("SetWindowRgn failed".into());
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
#[tauri::command]
pub fn apply_notch_region_cmd(
    app: tauri::AppHandle,
    width: f64,
    height: f64,
    scale_factor: Option<f64>,
    #[allow(non_snake_case)] scaleFactor: Option<f64>,
) -> Result<(), String> {
    let sf = scale_factor.or(scaleFactor).unwrap_or(1.0);
    apply_notch_region_inner(app.get_webview_window("notch"), width, height, sf)
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn apply_notch_region_cmd(
    _app: tauri::AppHandle,
    _width: f64,
    _height: f64,
    _scale_factor: Option<f64>,
    #[allow(non_snake_case)] _scaleFactor: Option<f64>,
) -> Result<(), String> {
    Ok(())
}

/// Snap a logical-pixel length to whole physical pixels for `scale_factor`,
/// expressed back in logical pixels. The OS places windows on physical pixels,
/// so a fractional physical edge gets rounded by the compositor and the notch
/// floats a hairline off the screen edge (the macOS 0.7pt flush-gap bug).
#[allow(dead_code)]
pub fn round_to_physical(logical: f64, scale_factor: f64) -> f64 {
    let sf = if scale_factor > 0.0 && scale_factor.is_finite() {
        scale_factor
    } else {
        1.0
    };
    (logical * sf).round() / sf
}

#[cfg(test)]
mod tests {
    use super::{notch_polygon_logical, notch_polygon_physical, round_to_physical};

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

    #[test]
    fn notch_polygon_matches_ts_samples() {
        // TS notchPath(70, 300) samples: top (70,0), innerTop (7.7,38.5), apex (0,150), innerBottom (7.7,261.5), bottom (70,300)
        let pts = notch_polygon_logical(70.0, 300.0);
        assert_eq!(pts[0], (70.0, 0.0));
        assert_eq!(pts[1], (70.0, 300.0));
        assert!((pts[2].0 - 7.7).abs() < 1e-9);
        assert!((pts[2].1 - 261.5).abs() < 1e-9);
        let last = pts.last().unwrap();
        assert!((last.0 - 7.7).abs() < 1e-9);
        assert!((last.1 - 38.5).abs() < 1e-9);
        let has_apex = pts.iter().any(|(x, y)| (*x - 3.85).abs() < 0.5 && (*y - 150.0).abs() < 1.0);
        assert!(has_apex, "bezier samples should include near-apex (3.85,150), got {:?}", pts);
        assert_eq!(pts.len(), 35);
    }

    #[test]
    fn notch_polygon_physical_at_100() {
        let phys = notch_polygon_physical(70.0, 300.0, 1.0);
        assert_eq!(phys[0], (70, 0));
        assert_eq!(phys[1], (70, 300));
        assert_eq!(phys[2], (8, 262)); // 7.7 rounded to 8, 261.5 rounded to 262
        let last = phys.last().unwrap();
        assert_eq!(*last, (8, 39)); // 7.7 -> 8, 38.5 -> 39
    }

    #[test]
    fn notch_polygon_physical_rounding_at_125() {
        // Logical 70x300 at 1.25 -> physical 88x375 (70*1.25=87.5->88, 300*1.25=375)
        let phys = notch_polygon_physical(70.0, 300.0, 1.25);
        assert_eq!(phys[0], (88, 0));
        assert_eq!(phys[1], (88, 375));
        // innerBottom (7.7,261.5) * 1.25 = (9.625->10, 326.875->327)
        assert_eq!(phys[2], (10, 327));
        // innerTop (7.7,38.5) * 1.25 = (10, 48)
        let last = phys.last().unwrap();
        assert_eq!(*last, (10, 48));
    }

    #[test]
    fn notch_polygon_physical_at_150() {
        let phys = notch_polygon_physical(70.0, 300.0, 1.5);
        assert_eq!(phys[0], (105, 0)); // 70*1.5=105
        assert_eq!(phys[1], (105, 450)); // 300*1.5=450
        // innerBottom (7.7, 261.5) * 1.5 = (11.55->12, 392.25->392)
        assert_eq!(phys[2], (12, 392));
        let last = phys.last().unwrap();
        assert_eq!(*last, (12, 58)); // 38.5 * 1.5 = 57.75 -> 58
    }

    #[test]
    fn notch_polygon_physical_at_175() {
        let phys = notch_polygon_physical(70.0, 300.0, 1.75);
        assert_eq!(phys[0], (123, 0)); // 70 * 1.75 = 122.5 -> 123
        assert_eq!(phys[1], (123, 525)); // 300 * 1.75 = 525
        assert_eq!(phys.len(), 35);
    }

    #[test]
    fn notch_polygon_physical_at_200() {
        let phys = notch_polygon_physical(70.0, 300.0, 2.0);
        assert_eq!(phys[0], (140, 0)); // 70 * 2.0 = 140
        assert_eq!(phys[1], (140, 600)); // 300 * 2.0 = 600
        assert_eq!(phys[2], (15, 523)); // 7.7 * 2 = 15.4 -> 15, 261.5 * 2 = 523
        let last = phys.last().unwrap();
        assert_eq!(*last, (15, 77)); // 38.5 * 2 = 77
        assert_eq!(phys.len(), 35);
    }

    #[test]
    fn notch_polygon_noop_for_small_height() {
        // height/2 < curl -> curl clamps to height/2
        let pts = notch_polygon_logical(70.0, 40.0);
        assert!((pts[2].1 - 20.0).abs() < 1e-9 || (pts[2].1 - (40.0 - 20.0)).abs() < 1e-9);
    }
}
