# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**`src/pricing.rs` + `src/pricing.json` implementieren.**

1. `src/pricing.json` mit aktuellen Anthropic-Preisen anlegen (Format: `docs/pricing.md` §2).
   Preise aus der offiziellen Preisliste für:
   - `claude-opus-4-7`, `claude-opus-4-6`
   - `claude-sonnet-4-6`
   - `claude-haiku-4-5`, `claude-haiku-4-5-20251001`
2. `src/pricing.rs` mit `PricingTable`:
   - `include_str!("pricing.json")` → bei Startup einmalig deserialisieren.
   - `lookup(model: &ModelId) -> Option<ModelPricing>`: exakt → Datum-Suffix-Strip → None.
   - `compute_cost(event: &UsageEvent) -> (Decimal, Option<ModelId>)`: Token × Preis;
     `Some(model)` wenn unbekannt (für `pricing_warnings`).
3. Unit-Tests in `pricing.rs`:
   - Exakter Match.
   - Datum-Suffix-Fallback (z.B. `claude-haiku-4-5-20251001` → `claude-haiku-4-5`).
   - Unbekanntes Modell → `None`, Warning-Pfad.
   - Kostenkalkulation für eine fixture-ähnliche Event-Konstante.

Danach: `src/aggregate.rs` (Rolling-Windows → `Snapshot` bauen).
