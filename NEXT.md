# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2: 5-Stunden-Billing-Windows / Session-Blocks (ADR-010).**

### Kontext

Claude Code rechnet in 5-Stunden-Blöcken ab (Rolling Window). Ein neuer Block beginnt
5 Stunden nach dem ersten Event des laufenden Blocks. Das ist für die Burn-Rate-Anzeige
und Limit-Warnungen notwendig.

### Schritte

1. **ADR-010 ausarbeiten** (sofern noch nicht geschehen) — Definitionen:
   - Block-Start: Timestamp des ersten Events nach einer Pause > 5 h (oder Programm-Start).
   - Block-Ende: Block-Start + 5 h.
   - Aktiver Block: der Block, dessen Fenster `Utc::now()` enthält.

2. **`src/aggregate.rs` erweitern**:
   - `SessionBlock { start: DateTime<Utc>, cost_usd: Decimal, token_count: u64 }`
   - `billing_blocks(events: &[UsageEvent]) -> Vec<SessionBlock>` — gruppiert Events
     in 5-h-Fenster.
   - `active_block(blocks: &[SessionBlock], now: DateTime<Utc>) -> Option<&SessionBlock>`
   - `Snapshot` um `active_block: Option<SessionBlock>` erweitern.

3. **`winusage watch` — Burn-Rate-Panel befüllen** (`src/bin/winusage-watch.rs`):
   - Aktuellen Block-Fortschritt als Balken: `elapsed / 5h`.
   - Token/Kosten-Summe im aktuellen Block.
   - Einfache Burn-Rate: `cost_usd / elapsed_hours` → `$/h`.

4. Tests in `src/aggregate.rs` für `billing_blocks()`.
