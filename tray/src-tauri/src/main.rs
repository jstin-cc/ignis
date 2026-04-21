// Tauri frameless tray-panel host for WinUsage.
//
// Window behaviour:
// - Frameless, always-on-top, no taskbar entry, initially hidden.
// - Tray-icon left-click toggles visibility.
// - Right-click context menu has a "Quit" item.
// - Spawns `winusage-api` as a child process; kills it on exit.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_updater::UpdaterExt;

struct ApiChild(Mutex<Option<Child>>);

fn find_api_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    let same_dir = dir.join("winusage-api.exe");
    if same_dir.exists() {
        return Some(same_dir);
    }

    for rel in [
        "../../../../target/release/winusage-api.exe",
        "../../../../target/debug/winusage-api.exe",
    ] {
        let candidate = dir.join(rel);
        if candidate.exists() {
            return candidate.canonicalize().ok();
        }
    }

    None
}

fn spawn_api() -> Option<Child> {
    let path = find_api_binary()?;
    let mut cmd = Command::new(&path);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.spawn() {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("failed to spawn winusage-api at {path:?}: {e}");
            None
        }
    }
}

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
fn get_api_token() -> Result<String, String> {
    let appdata = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/.config")))
        .map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(appdata)
        .join("winusage")
        .join("config.json");
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    val["api_token"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| "api_token not found in config".to_owned())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlanConfigDto {
    kind: String,
    custom_token_limit: Option<u64>,
}

fn config_path() -> Result<std::path::PathBuf, String> {
    let appdata = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/.config")))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(appdata)
        .join("winusage")
        .join("config.json"))
}

#[tauri::command]
fn get_plan_config() -> Result<PlanConfigDto, String> {
    let path = config_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let plan = val.get("plan");
    let kind = plan
        .and_then(|p| p.get("kind"))
        .and_then(|k| k.as_str())
        .unwrap_or("max5")
        .to_owned();
    let custom_token_limit = plan
        .and_then(|p| p.get("custom_token_limit"))
        .and_then(|v| v.as_u64());
    Ok(PlanConfigDto {
        kind,
        custom_token_limit,
    })
}

#[tauri::command]
fn set_plan_config(kind: String, custom_token_limit: Option<u64>) -> Result<(), String> {
    let path = config_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut plan = serde_json::json!({ "kind": kind });
    if let Some(limit) = custom_token_limit {
        plan["custom_token_limit"] = serde_json::json!(limit);
    }
    val["plan"] = plan;
    let json = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_cli_dashboard() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or_else(|| "no parent".to_owned())?;

    let candidates = [
        dir.join("winusage-watch.exe"),
        dir.join("../../../../target/release/winusage-watch.exe"),
        dir.join("../../../../target/debug/winusage-watch.exe"),
    ];

    let watch_path = candidates
        .iter()
        .find_map(|p| p.canonicalize().ok().filter(|p| p.exists()))
        .ok_or_else(|| "winusage-watch.exe not found".to_owned())?;

    #[cfg(windows)]
    {
        let path_str = watch_path.to_string_lossy().into_owned();
        Command::new("cmd")
            .args(["/C", "start", "WinUsage Dashboard", &path_str])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        Command::new(&watch_path).spawn().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
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
    let api_child = spawn_api();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(ApiChild(Mutex::new(api_child)))
        .invoke_handler(tauri::generate_handler![
            get_autostart_enabled,
            set_autostart_enabled,
            check_for_update,
            get_api_token,
            open_cli_dashboard,
            get_plan_config,
            set_plan_config,
        ])
        .setup(|app| {
            let quit_item = MenuItemBuilder::with_id("quit", "Quit WinUsage").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("no app icon configured")?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
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
        .build(tauri::generate_context!())
        .expect("error while building WinUsage tray application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            if let Some(state) = app_handle.try_state::<ApiChild>() {
                if let Ok(mut guard) = state.0.lock() {
                    if let Some(mut child) = guard.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
            }
        }
    });
}
