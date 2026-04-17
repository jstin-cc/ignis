# WinUsage

Windows-native local usage tracker for Claude Code. Reads JSONL logs from
`%USERPROFILE%\.claude\projects\` and surfaces token consumption, cost and
session status via three interfaces:

1. **System-Tray app** (Tauri 2 + React 18.3) — primary UI.
2. **Terminal CLI / TUI** (`winusage daily`, `winusage monthly`, `winusage session`).
3. **Local HTTP API** on `127.0.0.1:7337` for statuslines, editor plugins and scripts.

All data stays local. No cloud, no telemetry, no account.

## Status

**Phase 1 in progress** — core library complete, CLI and API pending.

| Module | State |
|---|---|
| `src/model.rs` — data types | ✅ |
| `src/parser.rs` — JSONL parser | ✅ |
| `src/pricing.rs` + `pricing.json` | ✅ |
| `src/aggregate.rs` — rolling windows | ✅ |
| `src/scanner.rs` — full + delta scan | ✅ |
| `src/config.rs` — paths + auth token | ✅ |
| `examples/scan.rs` — dev CLI | ✅ |
| CLI subcommands (`daily` / `monthly` / `session`) | ⏳ |
| HTTP API (`/health`, `/v1/summary`, `/v1/sessions`) | ⏳ |
| Tray app (Tauri 2 + React 18.3) | ⏳ |
| Installer (MSI) | ⏳ |

See `PROGRESS.md` for full phase breakdown, `NEXT.md` for the next concrete step,
`DECISIONS.md` for architecture decisions (ADR-001 – ADR-011).

## Quick start (dev)

```powershell
# Requires Rust 1.75+
cargo test                    # 32 tests, all green
cargo run --example scan      # full scan → JSON summary on stdout
```

## Repository

Private (`jstin-cc/winusage`). Tech stack, constraints and design decisions
are documented in `CLAUDE.md` and `docs/`.
