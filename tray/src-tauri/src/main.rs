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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Build the right-click context menu.
            let quit_item = MenuItemBuilder::with_id("quit", "Quit WinUsage").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

            // Tray icon — uses a 1×1 transparent placeholder; replace with a real
            // ICO asset once the icon is available (tauri.conf.json `icon` field).
            let _tray = TrayIconBuilder::new()
                .icon(Image::from_rgba(
                    vec![0u8, 0, 0, 0],
                    1,
                    1,
                )?)
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
