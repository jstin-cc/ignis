# WinUsage

Windows-native local usage tracker for Claude Code. Reads JSONL logs from
`%USERPROFILE%\.claude\projects\` and surfaces token consumption, cost and
session status via three interfaces:

1. **System-Tray app** (Tauri 2 + React 18.3) — primary UI.
2. **Terminal CLI / TUI** (`winusage watch`, ratatui).
3. **Local HTTP API** on `127.0.0.1:7337` for statuslines, editor plugins and scripts.

All data stays local. No cloud, no telemetry, no account.

## Status

**Phase 0** — scaffolding, documentation, architecture. No product code yet.
See `PROGRESS.md` for current state, `NEXT.md` for the next concrete step,
`DECISIONS.md` for architecture decisions.

## Repository

Private. Later-public checklist lives in `INITIAL_PROMPT.md` §Git & GitHub-Workflow.
