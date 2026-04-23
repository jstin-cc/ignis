# Ignis

Windows-native local usage tracker for Claude Code. Reads JSONL logs from
`%USERPROFILE%\.claude\projects\` and surfaces token consumption, cost and
session status via three interfaces:

1. **System-Tray app** (Tauri 2 + React 18.3) — primary UI. Spawns the HTTP API
   internally; nothing else to start.
2. **Terminal CLI** (`ignis daily`, `ignis monthly`, `ignis session`,
   `ignis export`).
3. **Live TUI** (`ignis-watch`) — ratatui-based dashboard with burn-rate,
   billing-block progress and by-model breakdown.
4. **Local HTTP API** on `127.0.0.1:7337` for statuslines, editor plugins and scripts.

All data stays local. No cloud, no telemetry, no account.

## Status

**v1.1.0 (2026-04-23).** Tray-UI redesigned: TabBar navigation, Design-System
tokens (IBM Plex fonts), CSS-based progress bars with colour thresholds,
and port-conflict safety for the auto-spawned API.

| Module | State |
|---|---|
| `src/model.rs` · `parser.rs` · `pricing.rs` · `aggregate.rs` | ✅ |
| `src/scanner.rs` — full + delta scan, NTFS file-identity | ✅ |
| `src/provider.rs` — `Provider` trait + `ClaudeCodeProvider` (ADR-012) | ✅ |
| CLI: `ignis daily / monthly / session / scan / export` | ✅ |
| Live TUI: `ignis-watch` (burn-rate, 5 h billing blocks) | ✅ |
| HTTP API: `/health`, `/v1/summary`, `/v1/sessions`, `/v1/heatmap` + CORS | ✅ |
| Tray app: TabBar (Today / Month / Projects / Heatmap) | ✅ |
| Tray: Usage-Balken (5h-Block, Woche, Extra) via Anthropic OAuth | ✅ |
| Tray: Plan-Settings (Pro / Max5 / Max20 / Custom) im ⚙-Overlay | ✅ |
| Tray: auto-spawns `ignis-api`, Port-7337-Konflikt-Check | ✅ |
| Tray: notifications (80 % / 100 % block), auto-start, auto-update | ✅ |
| Installer (MSI + NSIS via Tauri Bundler) | ✅ |

See `PROGRESS.md` for the full phase breakdown,
`DECISIONS.md` for architecture decisions (ADR-001 – ADR-013), and
`CHANGELOG.md` for release notes.

## Quick start (dev)

```powershell
# Requires Rust 1.75+
cargo test                          # 57 tests, all green
cargo run --example scan            # full scan → JSON summary on stdout
cargo run --bin ignis -- daily      # today's usage as ASCII table
cargo run --bin ignis-watch         # live TUI (q to quit, d/m for views)
cargo run --bin ignis-api           # start HTTP API on 127.0.0.1:7337
```

## Tray app (Windows)

```powershell
cd tray
npm install
npm run build                        # build the React frontend
cd src-tauri
cargo build --release                # or: npm run tauri build  (adds MSI + NSIS)
```

The resulting `ignis-tray.exe` spawns `ignis-api.exe` as a child process
on startup and terminates it on quit — no manual backend launch required
(see ADR-013). `ignis-api.exe` is looked up next to the tray binary or under
`target/release` during development.

Requires [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2, Rust, Node).

### Tray controls

- **Left-click tray icon** — toggle panel visibility
- **Right-click tray icon → Quit** — terminate the app (the window ✕-button only hides)
- **Header drag-region** — reposition the panel
- **Dashboard button** — launches `ignis-watch` in a new console
- **CLI button** — copies `ignis` to the clipboard

## Configuration

`%APPDATA%\ignis\config.json` — holds the auto-generated `api_token` (32 hex
chars, `getrandom`) and the Claude-projects directory. Created on first run of
either `ignis-api` or the tray.

## Repository

Private (`jstin-cc/ignis`). Tech stack, constraints and design decisions
are documented in `CLAUDE.md` and `docs/`.
