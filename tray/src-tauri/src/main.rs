// Tauri frameless tray-panel host for WinUsage.
//
// Window behaviour:
// - Frameless, always-on-top, no taskbar entry, initially hidden.
// - Tray-icon left-click toggles visibility.
// - Right-click context menu has a "Quit" item.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_updater::UpdaterExt;

fn toggle_panel<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_autostart_enabled(app: tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())
    } else {
        mgr.disable().map_err(|e| e.to_string())
    }
}

#[derive(serde::Serialize)]
struct UpdateCheckResult {
    available: bool,
    version: String,
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    let updater = app
        .updater_builder()
        .build()
        .map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => Ok(UpdateCheckResult {
            available: true,
            version: update.version,
        }),
        None => Ok(UpdateCheckResult {
            available: false,
            version: String::new(),
        }),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_autostart_enabled,
            set_autostart_enabled,
            check_for_update,
        ])
        .setup(|app| {
            // Build the right-click context menu.
            let quit_item = MenuItemBuilder::with_id("quit", "Quit WinUsage").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

            // Tray icon — uses a 1×1 transparent placeholder; replace with a real
            // ICO asset once the icon is available (tauri.conf.json `icon` field).
            let _tray = TrayIconBuilder::new()
                .icon(Image::new_owned(vec![0u8, 0, 0, 0], 1, 1))
                .menu(&menu)
                .tooltip("WinUsage")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_panel(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WinUsage tray application");
}
