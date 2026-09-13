pub mod usage;
pub mod cursor;
pub mod codex;
pub mod antigravity;
mod window;

use std::sync::Mutex;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub usage: Mutex<usage::UsageSnapshot>,
    pub codex: Mutex<usage::UsageSnapshot>,
    pub cursor: Mutex<usage::UsageSnapshot>,
    pub antigravity: Mutex<usage::UsageSnapshot>,
}

pub fn data_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("")).join("codenotch-win")
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
    state.usage.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn get_cursor(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state.cursor.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn get_codex(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state.codex.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn get_antigravity(state: tauri::State<AppState>) -> usage::UsageSnapshot {
    state.antigravity.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn refresh_usage(app: AppHandle) {
    {
        let st = app.state::<AppState>();
        let mut u = st.usage.lock().unwrap_or_else(|e| e.into_inner());
        u.backoff_until = 0;
    }
    usage::request_refresh();
    codex::request_refresh();
    cursor::request_refresh();
    antigravity::request_refresh();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = std::fs::create_dir_all(data_dir());
    
    tauri::Builder::default()
        .manage(AppState {
            usage: Mutex::new(usage::load_persisted()),
            codex: Mutex::new(codex::load_persisted()),
            cursor: Mutex::new(cursor::load_persisted()),
            antigravity: Mutex::new(antigravity::load_persisted()),
        })
        .invoke_handler(tauri::generate_handler![
            window::get_work_area,
            window::apply_notch_styles_cmd,
            get_usage,
            get_cursor,
            get_codex,
            get_antigravity,
            refresh_usage
        ])
        .setup(|app| {
            window::apply_notch_styles(app)?;
            // Tauri may reset exstyle on show(); re-apply shortly after the
            // frontend's `win.show()` (the frontend also invokes
            // `apply_notch_styles_cmd` after `show()`, this is a safety net
            // for the production build where the console is not visible).
            #[cfg(windows)]
            {
                let h = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    let _ = window::apply_notch_styles_cmd(h.clone());
                    std::thread::sleep(std::time::Duration::from_millis(900));
                    let _ = window::apply_notch_styles_cmd(h);
                });
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
