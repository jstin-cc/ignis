# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Usage-Balken (5h / Woche / Extra) via Anthropic OAuth abgeschlossen (2026-04-21).**

Tray-Panel zeigt drei echte Auslastungsbalken direkt von Anthropic. Plan-Dropdown im
Settings-Panel als Fallback wenn Credentials fehlen oder Nutzer offline ist.

Mögliche nächste Schritte:

- **v1.1.0 taggen** — CHANGELOG `[Unreleased]` → `[1.1.0]`, `git tag v1.1.0`, Release-Notes auf GitHub
- **Port-Konflikt-Handling** — prüfen ob Port 7337 schon belegt ist, bevor Child gespawnt wird
- **README.md** aktualisieren — Auto-Spawn, Usage-Balken, Plan-Settings dokumentieren
- **Token-Ablauf UX** — wenn `do_refresh` fehlschlägt, Hinweis im Panel statt stiller Leerfallback
