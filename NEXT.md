# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Post-v1.0 Hotfixes vom 2026-04-21 abgeschlossen.**

Tray-App ist jetzt End-to-End nutzbar: API spawnt automatisch, CORS funktioniert,
Release-Build lädt Frontend korrekt, Fenster lässt sich verschieben und scrollen,
Dashboard/CLI-Buttons sind verdrahtet.

Siehe `PROGRESS.md` → „Post-v1.0 Hotfixes (2026-04-21)" und
`DECISIONS.md` → ADR-013.

Mögliche nächste Schritte:

- **v1.0.1 taggen** (CHANGELOG `[Unreleased]` → `[1.0.1]`, Release-Notes, `git tag`)
- **README.md** auf Auto-Spawn-Flow aktualisieren (Abschnitt „Tray benutzt eine mitgelieferte API" statt „starte beide Prozesse")
- **Port-Konflikt-Handling** in Tray: prüfen, ob Port 7337 schon belegt ist, bevor der Child gespawnt wird — sonst crasht das Child still und die Tray spricht mit einer fremden Instanz.
- Nächste Feature-Phase planen (Provider-Support für Cursor/Codex, Light-Mode, …)
