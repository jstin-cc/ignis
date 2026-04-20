# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**v1.0.0 ist getaggt. Alle drei Phasen sind abgeschlossen.**

Mögliche nächste Schritte (kein fester Plan, Priorität nach Bedarf):

- **Pricing-Update** — `src/pricing.json` mit aktuellen Preisen refreshen.
- **Echter Update-Kanal** — Minisign-Keypair generieren, `tauri.conf.json` und
  GitHub-Release-Workflow einrichten (`latest.json` im Release-Artifact).
- **Provider-Implementierung** — zweiter konkreter Provider (Cursor, Codex) sobald
  JSONL-Format bekannt; `Provider`-Trait ist bereit (ADR-012).
- **Light-Mode** — `prefers-color-scheme: light` CSS-Variablen ergänzen (v0.3+).
- **Export-Erweiterungen** — `winusage export --output <file>` für Dateiausgabe.
