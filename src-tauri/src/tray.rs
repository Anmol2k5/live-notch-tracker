//! Adapted from vinzdg/codenotch's Windows tray menu/event pattern, pinned at
//! 6522e9f3ee0fe35d46e428b441e6e80b586eacbb. MIT; see third-party/codenotch.
//! Uses our Desk frontend and refresh entry point, not upstream's settings or account model.
use tauri::{Emitter, Manager};

pub fn request_desk(app: &tauri::AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Err(error) = crate::window::show_desk(handle.clone()) {
            eprintln!("Could not open Desk: {error}");
        }
        let _ = handle.emit("open-desk", ());
    });
}

pub fn install(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    let show = MenuItemBuilder::with_id("desk", "Open Desk").build(app)?;
    let refresh = MenuItemBuilder::with_id("refresh", "Refresh AI usage").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Live Notch Tracker").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&refresh)
        .separator()
        .item(&quit)
        .build()?;
    let mut tray = TrayIconBuilder::with_id("desk-tray")
        .tooltip("Live Notch Tracker — Open Desk")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "desk" => request_desk(app),
            "refresh" => crate::refresh_usage(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                request_desk(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

pub fn close_to_tray(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        // Retain a visible window if the tray could not be created.
        if window.app_handle().tray_by_id("desk-tray").is_some() && window.hide().is_ok() {
            api.prevent_close();
        }
    }
}
