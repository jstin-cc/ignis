// Tauri frameless tray-panel host for Ignis.
//
// Window behaviour:
// - Frameless, always-on-top, no taskbar entry, initially hidden.
// - Tray-icon left-click toggles visibility.
// - Right-click context menu has a "Quit" item.
// - Spawns `ignis-api` as a child process; kills it on exit.

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

// ── Anthropic OAuth Usage API ─────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct UsageWindow {
    utilization: u8,
    resets_at: String,
}

#[derive(serde::Serialize)]
struct ExtraUsage {
    is_enabled: bool,
    used_usd: String,
    monthly_limit_usd: String,
    /// true wenn monthly_limit == 0 (kein Limit gesetzt)
    is_unlimited: bool,
    pct: u8,
}

#[derive(serde::Serialize)]
struct AnthropicUsageDto {
    five_hour: Option<UsageWindow>,
    seven_day: Option<UsageWindow>,
    extra_usage: Option<ExtraUsage>,
}

#[derive(serde::Deserialize)]
struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OAuthEntry,
}

#[derive(serde::Deserialize)]
struct OAuthEntry {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: i64, // unix milliseconds
}

fn credentials_path() -> Result<std::path::PathBuf, String> {
    let profile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(profile)
        .join(".claude")
        .join(".credentials.json"))
}

fn read_credentials() -> Result<OAuthEntry, String> {
    let path = credentials_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let creds: Credentials = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(creds.claude_ai_oauth)
}

fn write_tokens(access_token: &str, refresh_token: &str, expires_at: i64) -> Result<(), String> {
    let path = credentials_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if let Some(oauth) = val.get_mut("claudeAiOauth") {
        oauth["accessToken"] = serde_json::json!(access_token);
        oauth["refreshToken"] = serde_json::json!(refresh_token);
        oauth["expiresAt"] = serde_json::json!(expires_at);
    }
    let json = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

async fn do_refresh(client: &reqwest::Client, refresh_token: &str) -> Result<OAuthEntry, String> {
    const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
    const SCOPE: &str = "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload";

    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": CLIENT_ID,
        "scope": SCOPE,
    });

    let resp = client
        .post("https://platform.claude.com/v1/oauth/token")
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Token-Refresh fehlgeschlagen: {e}"))?;

    if !resp.status().is_success() {
        return Err("Token abgelaufen. Bitte `claude` neu starten.".to_owned());
    }

    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let new_access = val["access_token"]
        .as_str()
        .ok_or("access_token fehlt in Refresh-Response")?
        .to_owned();
    let new_refresh = val["refresh_token"]
        .as_str()
        .unwrap_or(refresh_token)
        .to_owned();
    let expires_in = val["expires_in"].as_i64().unwrap_or(3600);
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
        + expires_in * 1_000;

    write_tokens(&new_access, &new_refresh, expires_at)?;

    Ok(OAuthEntry {
        access_token: new_access,
        refresh_token: new_refresh,
        expires_at,
    })
}

#[tauri::command]
async fn get_anthropic_usage() -> Result<AnthropicUsageDto, String> {
    let client = reqwest::Client::new();

    let mut creds = read_credentials()?;

    // Refresh if token expires within 5 minutes.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    if creds.expires_at < now_ms + 5 * 60 * 1_000 {
        creds = do_refresh(&client, &creds.refresh_token).await?;
    }

    let resp = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", creds.access_token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "ignis/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Anthropic API nicht erreichbar: {e}"))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        // Try one refresh, then retry.
        creds = do_refresh(&client, &creds.refresh_token).await?;
        let retry = client
            .get("https://api.anthropic.com/api/oauth/usage")
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("User-Agent", "ignis/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Anthropic API nicht erreichbar: {e}"))?;
        if !retry.status().is_success() {
            return Err(format!("Anthropic API Fehler ({})", retry.status()));
        }
        return parse_usage_response(retry).await;
    }

    if !resp.status().is_success() {
        return Err(format!("Anthropic API Fehler ({})", resp.status()));
    }

    parse_usage_response(resp).await
}

async fn parse_usage_response(resp: reqwest::Response) -> Result<AnthropicUsageDto, String> {
    let val: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Antwort nicht parsebar: {e}"))?;

    let five_hour = parse_window(&val["five_hour"]);
    let seven_day = parse_window(&val["seven_day"]);
    let extra_usage = parse_extra(&val["extra_usage"]);

    Ok(AnthropicUsageDto {
        five_hour,
        seven_day,
        extra_usage,
    })
}

fn parse_window(v: &serde_json::Value) -> Option<UsageWindow> {
    // utilization may be integer or float in the JSON response
    let utilization = v
        .get("utilization")?
        .as_f64()
        .map(|f| f.round().clamp(0.0, 100.0) as u8)?;
    let resets_at = v.get("resets_at")?.as_str()?.to_owned();
    Some(UsageWindow {
        utilization,
        resets_at,
    })
}

fn parse_extra(v: &serde_json::Value) -> Option<ExtraUsage> {
    if v.is_null() || !v.is_object() {
        return None;
    }
    let is_enabled = v.get("is_enabled").and_then(|x| x.as_bool()).unwrap_or(false);
    // Credits may be integer or float in the JSON response.
    let used_cents = v.get("used_credits").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let limit_cents = v.get("monthly_limit").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let is_unlimited = limit_cents <= 0.0;
    let pct = if !is_unlimited && limit_cents > 0.0 {
        ((used_cents / limit_cents * 100.0).clamp(0.0, 100.0)) as u8
    } else {
        0
    };
    let used_usd = format!("{:.2}", used_cents / 100.0);
    let monthly_limit_usd = format!("{:.2}", limit_cents / 100.0);
    Some(ExtraUsage {
        is_enabled,
        used_usd,
        monthly_limit_usd,
        is_unlimited,
        pct,
    })
}

fn find_api_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    let same_dir = dir.join("ignis-api.exe");
    if same_dir.exists() {
        return Some(same_dir);
    }

    for rel in [
        "../../../../target/release/ignis-api.exe",
        "../../../../target/debug/ignis-api.exe",
    ] {
        let candidate = dir.join(rel);
        if candidate.exists() {
            return candidate.canonicalize().ok();
        }
    }

    None
}

fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn spawn_api() -> Option<Child> {
    if !port_is_free(7337) {
        // Port already in use — existing ignis-api instance running, skip spawn.
        return None;
    }
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
            eprintln!("failed to spawn ignis-api at {path:?}: {e}");
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
        .join("ignis")
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
    usage_poll_interval_secs: u32,
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
    let usage_poll_interval_secs = plan
        .and_then(|p| p.get("usage_poll_interval_secs"))
        .and_then(|v| v.as_u64())
        .unwrap_or(60) as u32;
    Ok(PlanConfigDto {
        kind,
        custom_token_limit,
        usage_poll_interval_secs,
    })
}

#[tauri::command]
fn set_plan_config(
    kind: String,
    custom_token_limit: Option<u64>,
    usage_poll_interval_secs: Option<u32>,
) -> Result<(), String> {
    let path = config_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut plan = serde_json::json!({ "kind": kind });
    if let Some(limit) = custom_token_limit {
        plan["custom_token_limit"] = serde_json::json!(limit);
    }
    if let Some(secs) = usage_poll_interval_secs {
        plan["usage_poll_interval_secs"] = serde_json::json!(secs);
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
        dir.join("ignis-watch.exe"),
        dir.join("../../../../target/release/ignis-watch.exe"),
        dir.join("../../../../target/debug/ignis-watch.exe"),
    ];

    // canonicalize() returns \\?\ UNC paths on Windows which cmd.exe rejects;
    // strip the prefix to get a plain absolute path.
    let watch_path_str = candidates
        .iter()
        .find_map(|p| {
            p.canonicalize().ok().filter(|cp| cp.exists()).map(|cp| {
                let s = cp.to_string_lossy().into_owned();
                s.strip_prefix(r"\\?\").unwrap_or(&s).to_owned()
            })
        })
        .unwrap_or_else(|| "ignis-watch".to_owned()); // PATH fallback

    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "Ignis Dashboard", &watch_path_str])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        Command::new(&watch_path_str).spawn().map_err(|e| e.to_string())?;
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
            get_anthropic_usage,
        ])
        .setup(|app| {
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Ignis").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("no app icon configured")?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Ignis")
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
        .expect("error while building Ignis tray application");

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
