# WinUsage

Windows-native local usage tracker for Claude Code. Reads JSONL logs from
`%USERPROFILE%\.claude\projects\` and surfaces token consumption, cost and
session status via three interfaces:

1. **System-Tray app** (Tauri 2 + React 18.3) — primary UI. Spawns the HTTP API
   internally; nothing else to start.
2. **Terminal CLI** (`winusage daily`, `winusage monthly`, `winusage session`,
   `winusage export`).
3. **Live TUI** (`winusage-watch`) — ratatui-based dashboard with burn-rate,
   billing-block progress and by-model breakdown.
4. **Local HTTP API** on `127.0.0.1:7337` for statuslines, editor plugins and scripts.

All data stays local. No cloud, no telemetry, no account.

## Status

**Phase 3 complete (`v1.0.0`, 2026-04-20).** Post-v1.0 hotfix pass on 2026-04-21
closed the remaining end-to-end gaps (auto-spawned API, CORS, drag-region,
scrollable panel, Dashboard/CLI buttons).

| Module | State |
|---|---|
| `src/model.rs` · `parser.rs` · `pricing.rs` · `aggregate.rs` | ✅ |
| `src/scanner.rs` — full + delta scan, NTFS file-identity | ✅ |
| `src/provider.rs` — `Provider` trait + `ClaudeCodeProvider` (ADR-012) | ✅ |
| CLI: `winusage daily / monthly / session / scan / export` | ✅ |
| Live TUI: `winusage-watch` (burn-rate, 5 h billing blocks) | ✅ |
| HTTP API: `/health`, `/v1/summary`, `/v1/sessions`, `/v1/heatmap` + CORS | ✅ |
| Tray app: Today / Month / Block / Projects / Heatmap / Active Session | ✅ |
| Tray: auto-spawns `winusage-api` as child process (ADR-013) | ✅ |
| Tray: notifications (80 % / 100 % block), auto-start, auto-update | ✅ |
| Installer (MSI + NSIS via Tauri Bundler) | ✅ |

See `PROGRESS.md` for the full phase breakdown, `NEXT.md` for the next concrete step,
`DECISIONS.md` for architecture decisions (ADR-001 – ADR-013), and
`CHANGELOG.md` for release notes.

## Quick start (dev)

```powershell
# Requires Rust 1.75+
cargo test                          # 57 tests, all green
cargo run --example scan            # full scan → JSON summary on stdout
cargo run --bin winusage -- daily   # today's usage as ASCII table
cargo run --bin winusage-watch      # live TUI (q to quit, d/m for views)
cargo run --bin winusage-api        # start HTTP API on 127.0.0.1:7337
```

## Tray app (Windows)

```powershell
cd tray
npm install
npm run build                        # build the React frontend
cd src-tauri
cargo build --release                # or: npm run tauri build  (adds MSI + NSIS)
```

The resulting `winusage-tray.exe` spawns `winusage-api.exe` as a child process
on startup and terminates it on quit — no manual backend launch required
(see ADR-013). `winusage-api.exe` is looked up next to the tray binary or under
`target/release` during development.

Requires [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2, Rust, Node).

### Tray controls

- **Left-click tray icon** — toggle panel visibility
- **Right-click tray icon → Quit** — terminate the app (the window ✕-button only hides)
- **Header drag-region** — reposition the panel
- **Dashboard button** — launches `winusage-watch` in a new console
- **CLI button** — copies `winusage` to the clipboard

## Configuration

`%APPDATA%\winusage\config.json` — holds the auto-generated `api_token` (32 hex
chars, `getrandom`) and the Claude-projects directory. Created on first run of
either `winusage-api` or the tray.

## Repository

Private (`jstin-cc/winusage`). Tech stack, constraints and design decisions
are documented in `CLAUDE.md` and `docs/`.
