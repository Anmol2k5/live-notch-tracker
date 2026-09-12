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

pub fn is_claude_running() -> bool {
    // A simplified active check, returning true so it polls every 60s
    true
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
            get_usage,
            get_cursor,
            get_codex,
            get_antigravity,
            refresh_usage
        ])
        .setup(|app| {
            window::apply_notch_styles(app)?;
            
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
