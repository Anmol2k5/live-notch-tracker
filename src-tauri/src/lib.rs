pub mod antigravity;
mod autostart;
pub mod codex;
mod codex_rpc;
pub mod cursor;
mod desk_geometry;
mod desktop;
mod github_calendar;
mod launcher;
pub mod storage;
#[cfg(windows)]
mod tray;
pub mod usage;
mod window;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub usage: Mutex<usage::UsageSnapshot>,
    pub codex: Mutex<usage::UsageSnapshot>,
    pub cursor: Mutex<usage::UsageSnapshot>,
    pub antigravity: Mutex<usage::UsageSnapshot>,
}

pub fn data_dir() -> PathBuf {
    storage::app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Whether Claude Code appears to be running on this machine.
/// Used to pick the poll cadence in `usage.rs`: 60 s while active, 300 s otherwise.
/// On Windows this checks for a `Win32_Process` whose name contains "claude"
/// (covers `claude.exe`, `claude-code`, etc.), mirroring the `Get-CimInstance`
/// approach already used in `antigravity.rs` for `language_server` discovery.
pub fn is_claude_running() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let out = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_Process -Filter \"Name LIKE '%claude%'\" | ForEach-Object { $_.ProcessId }",
            ])
            .creation_flags(0x0800_0000)
            .output();
        match out {
            Ok(o) => {
                let s = String::from_utf8_lossy(&o.stdout);
                s.lines().any(|l| l.trim().parse::<u32>().is_ok())
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
fn get_usage(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state
        .usage
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn get_cursor(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state
        .cursor
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn get_codex(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state
        .codex
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn get_antigravity(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state
        .antigravity
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn refresh_usage() {
    // Wake pollers without mutating their persisted Retry-After deadlines.
    usage::request_refresh();
    codex::request_refresh();
    cursor::request_refresh();
    antigravity::request_refresh();
}

#[tauri::command]
fn get_startup_enabled() -> Result<bool, String> {
    autostart::is_enabled()
}

#[tauri::command]
async fn set_startup_enabled(enabled: bool) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if enabled {
            autostart::enable()?;
        } else {
            autostart::disable()?;
        }
        autostart::is_enabled()
    })
    .await
    .map_err(|_| "Windows startup settings were interrupted.".to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = std::fs::create_dir_all(data_dir());

    let builder = tauri::Builder::default();
    #[cfg(windows)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::request_desk(app)
        }))
        .on_window_event(tray::close_to_tray);
    builder
        .manage(AppState {
            usage: Mutex::new(usage::load_persisted()),
            codex: Mutex::new(codex::load_persisted()),
            cursor: Mutex::new(cursor::load_persisted()),
            antigravity: Mutex::new(antigravity::load_persisted()),
        })
        .invoke_handler(tauri::generate_handler![
            window::get_work_area,
            window::apply_notch_styles_cmd,
            window::apply_notch_region_cmd,
            window::show_desk,
            desktop::get_github_calendar,
            desktop::launch_app,
            get_startup_enabled,
            set_startup_enabled,
            get_usage,
            get_cursor,
            get_codex,
            get_antigravity,
            refresh_usage
        ])
        .setup(|app| {
            #[cfg(windows)]
            tray::install(app)?;
            let background = cfg!(windows) && std::env::args().any(|arg| arg == "--background");
            if !background {
                window::show_desk(app.handle().clone()).map_err(std::io::Error::other)?;
            }

            let handle = app.handle().clone();
            usage::start(handle.clone());
            codex::start(handle.clone());
            cursor::start(handle.clone());
            antigravity::start(handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
