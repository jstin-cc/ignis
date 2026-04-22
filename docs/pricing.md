# Pricing

Regeln und Datenformat für die Umrechnung Tokens → USD.

Verknüpfte Entscheidung: **ADR-004** — Pricing-Daten werden als `pricing.json`
**embedded** ins Binary ausgeliefert. Updates nur via App-Update. Kein Remote-Fetch,
kein User-editierbares Override im MVP.

---

## 1. Speicherung

- `src/pricing.json` im Repository.
- Ins Binary via `include_str!("pricing.json")` eingebettet.
- Zur Laufzeit einmalig beim Start deserialisiert (`serde_json::from_str` → `PricingTable`).

Ein späterer Override-Mechanismus (`%APPDATA%\ignis\pricing.local.json` mergt über
die Defaults) steht im Backlog, ist aber **nicht MVP** — zu wenig Use-Cases, zu viel
Oberfläche für Inkonsistenzen.

---

## 2. Datenformat

```jsonc
{
  "version": 1,
  "updated": "2026-04-17",
  "models": {
    "claude-opus-4-7": {
      "input_per_mtok":        "15.00",
      "output_per_mtok":       "75.00",
      "cache_read_per_mtok":    "1.50",
      "cache_write_5m_per_mtok":"18.75",
      "cache_write_1h_per_mtok":"30.00"
    },
    "claude-sonnet-4-6": {
      "input_per_mtok":         "3.00",
      "output_per_mtok":       "15.00",
      "cache_read_per_mtok":    "0.30",
      "cache_write_5m_per_mtok":"3.75",
      "cache_write_1h_per_mtok":"6.00"
    }
    // weitere Modelle …
  }
}
```

- Alle Preise sind **Strings** (Decimal-friendly). Einheit: USD pro 1 000 000 Tokens.
- `cache_write_5m` vs. `cache_write_1h` spiegeln die unterschiedlichen TTL-Preise
  (siehe `docs/jsonl-format.md` §7).
- `version` ist ein Integer-Zähler für Format-Migrationen (aktuell `1`).

> **Platzhalter-Werte im Repository.** Die konkreten Zahlen oben sind **Beispielwerte**
> zur Schema-Illustration. Das tatsächliche `pricing.json` wird vor dem ersten echten
> Release (v0.1.0-mvp) mit Werten aus der offiziellen Anthropic-Preisliste befüllt
> und bei jedem Release reviewt.

---

## 3. Lookup-Regeln

Reihenfolge beim Nachschlagen einer Model-ID aus einem `UsageEvent`:

1. **Exakter Match** gegen `models`-Key (z.B. `"claude-haiku-4-5-20251001"`).
2. **Fallback ohne Datums-Suffix** — strip eines abschließenden `-YYYYMMDD`
   (Regex `/-\d{8}$/`) und erneut suchen
   (z.B. `"claude-haiku-4-5-20251001"` → `"claude-haiku-4-5"`).
3. **Miss** — Token werden **gezählt**, aber **nicht gebillt**. Die Model-ID landet in
   `Snapshot.pricing_warnings`. UI zeigt einen Warning-Hinweis neben der Gesamtsumme
   ("Unbekannte Modelle in diesem Zeitraum: N").

Dieses Fallback-Schema ist in `docs/jsonl-format.md` §6 empirisch motiviert — wir haben
beobachtet, dass nur `claude-haiku-*` ein Datum-Suffix mitführt, der Rest nicht.

---

## 4. Kostenformel pro Event

Aus `docs/jsonl-format.md` §7 übernommen:

```
cost_usd =
      input_tokens                                * input_price
    + output_tokens                               * output_price
    + cache_read_input_tokens                     * cache_read_price
    + cache_creation.ephemeral_5m_input_tokens    * cache_write_5m_price
    + cache_creation.ephemeral_1h_input_tokens    * cache_write_1h_price
```

Fallback, falls `cache_creation.*` fehlt und nur das Top-Level-`cache_creation_input_tokens`
gesetzt ist: der Gesamtbetrag wird als 5m-Write verrechnet (konservativ, da 5m billiger
als 1h).

`server_tool_use.*` wird **nicht** in die Kosten eingerechnet — diese Preise sind
nicht Teil der Standard-Tabelle. Ignis zeigt die Counts informativ an.

---

## 5. Präzision

- Alle Preise und Zwischenergebnisse in `rust_decimal::Decimal` (28 signifikante Stellen).
- **Kein** f64 an keiner Stelle.
- Serialisierung in der API als String (`"1.8274"`). JSON-Number würde bei < 0.01 USD
  Präzision verlieren.

---

## 6. Warning-Flow

```
Event mit Model "claude-opus-5-99"
  ↓
PricingTable.lookup() → None
  ↓
Aggregator:
  - addiert Tokens normal,
  - cost_usd bleibt unverändert für diesen Event,
  - fügt "claude-opus-5-99" zu Snapshot.pricing_warnings hinzu.
  ↓
UI / API:
  - Tray zeigt Warnung unter der Summen-Zeile.
  - CLI (`ignis daily`) zeigt eine Zeile "⚠ unpriced models: claude-opus-5-99".
  - `/v1/summary` gibt "pricing_warnings": ["claude-opus-5-99"] zurück.
  - `/health` listet die kumulierten Warnungen.
```

Der Nutzer erkennt sofort, dass ein App-Update fällig ist, und wird nicht mit stiller
Unter-Buchung konfrontiert.

---

## 7. Release-Pflicht

Pricing-Updates sind Teil jedes App-Releases:

- Vor jedem Release `src/pricing.json` gegen die offizielle Anthropic-Preisliste abgleichen.
- Neue Modelle: ADR oder zumindest `CHANGELOG.md`-Eintrag.
- Entfernte Modelle: **nicht entfernen** — historische Events brauchen weiterhin ihre
  Preise. Modelle markieren wir höchstens als `"deprecated": true` (optional, Format-
  Erweiterung).

---

## 8. Nicht-Ziele (MVP)

- Kein Remote-Fetch von `pricing.json`.
- Keine User-eigene `pricing.local.json` (Backlog).
- Kein Volumen-Rabatt, keine Enterprise-Preise.
- Keine Währungsumrechnung — USD ist die einzige unterstützte Währung.
