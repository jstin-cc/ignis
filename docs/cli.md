# CLI Reference — ignis

Lokales CLI-Binary, das denselben Scan-Kern wie die Tray-App nutzt und die Rohdaten
aus `%USERPROFILE%\.claude\projects\` auswertet.

---

## Subcommands

| Subcommand | Beschreibung                                     |
|------------|--------------------------------------------------|
| `daily`    | Token/Kosten-Tabelle für heute                   |
| `monthly`  | Token/Kosten-Tabelle für den Kalendermonat       |
| `session`  | Aktive Session anzeigen (falls vorhanden)        |
| `scan`     | Vollständiger JSON-Dump (Dev-Werkzeug)           |
| `export`   | Nutzungsdaten als JSON oder CSV ausgeben         |

---

## `ignis export`

```
ignis export [OPTIONS]

Options:
  -f, --format <FORMAT>   json | csv  [default: json]
  -p, --period <PERIOD>   today | week | month  [default: month]
  -o, --output <FILE>     Zieldatei (Standard: stdout)
      --force             Bestehende Datei überschreiben
  -h, --help              Diese Hilfe anzeigen
```

### Verhalten

- **Stdout** (kein `--output`): Ausgabe direkt auf stdout — pipes direkt weiter.
- **Datei** (`--output <FILE>`):
  - Schreibt zunächst in eine temporäre Datei (`<FILE>.ignis_tmp`), dann atomarer
    Rename auf `<FILE>`. Crash mitten im Write hinterlässt kein beschädigtes Ziel.
  - Existiert `<FILE>` bereits, bricht der Befehl mit einem Fehler ab — es sei denn,
    `--force` ist gesetzt.
  - Das übergeordnete Verzeichnis muss existieren; `ignis export` legt es **nicht** an.

### Beispiele

```powershell
# Monatsbericht als JSON auf stdout
ignis export

# Heutigen Report in Datei schreiben
ignis export --period today --output "$env:USERPROFILE\Desktop\ignis-today.json"

# Wochenbericht als CSV, überschreibe vorhandene Datei
ignis export --period week --format csv --output report.csv --force

# Pipe in jq
ignis export | jq '.by_model[].cost_usd'
```

### JSON-Schema (Kurzform)

```json
{
  "period":         "month",
  "total_cost_usd": "3.14",
  "total_tokens":   1000000,
  "event_count":    42,
  "by_model": [
    { "model": "claude-sonnet-4-6", "input_tokens": 620, "output_tokens": 18200, "cost_usd": "0.60" }
  ],
  "by_project": [
    { "project": "D:\\.claude\\projects\\winusage", "total_tokens": 612000, "cost_usd": "1.48" }
  ]
}
```

Geldbeträge sind Strings (keine JSON-Number-Floats) — identisches Schema wie `/v1/summary`.

---

## Hinweise

- Der Scan liest immer alle JSONL-Dateien neu ein; es gibt keinen laufenden Daemon.
  Für kontinuierliches Monitoring → `ignis-api` + HTTP-Polling.
- `ignis scan` ist ein Dev-Werkzeug und gibt interne Zähler aus; das Format ist
  nicht stabil.
