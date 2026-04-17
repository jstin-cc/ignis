# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 0 fortsetzen: JSONL-Format empirisch untersuchen.**

Konkret:

1. Auflisten, welche Dateien unter `%USERPROFILE%\.claude\projects\` liegen.
2. Eine repräsentative, nicht zu kleine JSONL-Datei auswählen.
3. Zeilentypen (Assistant-Messages, Tool-Uses, Meta-Events) inventarisieren.
4. Token-Usage-Felder identifizieren (input / output / cache_creation / cache_read).
5. Schema in `docs/jsonl-format.md` dokumentieren — inkl. Quirks (fehlende Felder,
   variable Reihenfolgen, Stream-Chunks).
6. 2–3 **anonymisierte** Sample-Lines in `fixtures/` ablegen (Pfade/Projektnamen
   scrubben).

Blockiert: `docs/architecture.md` kann erst danach final geschrieben werden — das
Datenmodell hängt am tatsächlichen Schema.

## Danach

- `docs/architecture.md` (inkl. Position-Tracking-Design pro File).
- `docs/api.md` (Schema für `/v1/summary`, `/v1/sessions`, `/health`).
- `docs/design-system.md` (Farbpalette aus `INITIAL_PROMPT.md` übertragen).
- 3 Agent-Definitionen in `.claude/agents/`.
- Git-Init + Initial-Commit + `gh repo create winusage --private --source=. --push`.
- `PROGRESS.md`/`NEXT.md` final auf Phase 1 ausrichten.
