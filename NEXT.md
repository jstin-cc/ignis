# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2 starten: `winusage watch` Live-TUI mit ratatui.**

### Schritte

1. Dependencies: `ratatui`, `crossterm` in `[dependencies]`.

2. `src/bin/winusage-watch.rs` — ratatui TUI:
   - Layout wie in `docs/design-system.md` §6 beschrieben
   - Panels: Today, Session, By Model, Burn Rate (Platzhalter)
   - Live-Refresh alle 5 Sekunden via Scanner-Watch-Thread
   - Keys: `q` quit, `r` force-refresh, `d` daily, `m` monthly

3. TUI nutzt `Scanner::start_watching()` (notify crate) für Push-Updates.

4. ANSI-TrueColor mit Hex-Werten aus `docs/design-system.md` §1.
   Fallback: 8-Farben bei `NO_COLOR` oder kein TrueColor-TTY.

Danach: 5-Stunden-Billing-Windows / Session-Blocks (ADR-010).
