# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2 abschließen: Release-Tag `v0.2.0` setzen.**

### Kontext

Alle Phase-2-Features sind implementiert:
- `winusage watch` Live-TUI
- 5-Stunden-Billing-Windows + Burn-Rate
- Tray: BlockPanel, ProjectsPanel, Notifications, Auto-Start

### Schritte

1. **`CHANGELOG.md`** — Eintrag für `v0.2.0` schreiben (alle Phase-2-Features auflisten).
2. **`tray/src-tauri/tauri.conf.json`** — `version` auf `"0.2.0"` setzen.
3. **`Cargo.toml`** (Root) — `version` auf `"0.2.0"` setzen.
4. Commit: `chore: bump version to v0.2.0`.
5. Git-Tag: `git tag v0.2.0 && git push --tags`.

Danach: Phase 3 planen (Provider-Plugin-Trait, CSV/JSON-Export, Heatmap).
