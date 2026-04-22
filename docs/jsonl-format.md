# JSONL-Format — empirische Dokumentation

**Quelle:** Eigener Claude-Code-Logs-Bestand vom **2026-04-17**, 38 Files, ~13 MB,
Versionen 2.1.84 – 2.1.112. Diese Doku ist **beschreibend**, nicht spekulativ. Was hier
nicht steht, ist nicht beobachtet worden — auftauchende neue Felder werden ergänzt, keine
aus der Luft erfunden.

> **Scope-Warnung.** Claude Code ist Closed-Source, das JSONL-Format ist undokumentiert
> und kann sich zwischen Minor-Versionen ändern. Der Scanner muss **additiv und
> tolerant** lesen: unbekannte `type`-Werte werden geloggt und übersprungen, unbekannte
> Felder innerhalb bekannter Typen ignoriert.

---

## 1. Verzeichnisstruktur

```
%USERPROFILE%\.claude\projects\
├── D---claude-projects-winusage\
│   ├── 35bf8651-bda1-471b-9117-87f8a6817ca0.jsonl
│   ├── 537c7647-0cc5-4bc0-9880-8b77f4e41017.jsonl
│   └── memory\            ← Auto-Memory (kein Session-Log, ignorieren)
├── D---claude-projects-hotline\
│   ├── <sessionId>.jsonl
│   └── <sessionId>\       ← File-Backups pro Snapshot (ignorieren)
├── subagents\             ← Sub-Agent-Logs, gleiches JSONL-Format
└── …
```

**Projekt-Directory-Naming.** Claude Code kodiert den `cwd` als Directory-Name, indem es
jeden Pfad-Trenner (`:`, `\`, `/`) und jeden führenden `.` durch einen einzelnen `-`
ersetzt. Beispiele:

| `cwd`                                    | Directory-Name                        |
|------------------------------------------|---------------------------------------|
| `D:\.claude\projects\winusage`           | `D---claude-projects-winusage`        |
| `D:\.claude\projects\hotline`            | `D---claude-projects-hotline`         |
| `C:\Users\Justin-Strittmatter\AppData\Roaming\Claude` | `C--Users-Justin-Strittmatter-AppData-Roaming-Claude` |
| `R:\001-Rechnungen-LS-Kreditoren\Nespresso` | `R--001-Rechnungen-LS-Kreditoren-Nespresso` |

Diese Kodierung ist **nicht reversibel** (mehrere Pfade können denselben Directory-Namen
ergeben). **Für Ignis gilt: der Projekt-Pfad wird ausschließlich aus dem `cwd`-Feld
der JSONL-Zeilen gelesen, nie aus dem Directory-Namen.** Der Directory-Name ist nur ein
Grouping-Key für den Scanner.

**Das `subagents/`-Directory** enthält JSONL-Files von Sub-Agenten, angelegt beim
ersten Sidechain-Call. Format identisch zu Top-Level-Sessions. Für Usage-Aggregation
gelten sie als Teil der Eltern-Session (siehe §5).

**Das `memory/`-Unterverzeichnis unter Projekt-Dirs** ist das Auto-Memory-System (nicht
Session-bezogen). Keine `*.jsonl`-Files — wird vom Scanner ignoriert.

**Backup-Unterordner `<sessionId>/`** enthalten `file-history-snapshot`-Nutzdaten
(Tool-Backups). Der Scanner braucht sie nicht.

---

## 2. Datei-Konvention

- Name: `{sessionId}.jsonl` — der UUID-Basename entspricht exakt dem `sessionId`-Feld
  in jeder Zeile.
- Ein File = eine Session. Append-only: Claude Code schreibt Zeilen an, bearbeitet oder
  löscht nie. (Wird das beobachtet, ist das ein kritischer Bug im Parser — Position-
  Tracking würde Off-by-Bytes laufen.)
- Encoding: UTF-8, eine JSON-Zeile pro physikalischer Zeile, `\n` als Separator.

---

## 3. Gemeinsame Felder (fast alle Zeilen)

| Feld              | Typ              | Pflicht? | Bedeutung                                    |
|-------------------|------------------|----------|----------------------------------------------|
| `type`            | string           | ja       | Zeilentyp (siehe §4).                        |
| `uuid`            | string (UUID)    | fast immer | Unique-ID der Zeile.                      |
| `parentUuid`      | string \| null   | meistens | Referenz auf die vorherige Zeile in der Kette. |
| `timestamp`       | string (RFC 3339)| ja       | UTC-Timestamp, Millisekunden-Präzision.     |
| `sessionId`       | string (UUID)    | meistens | Gleich dem Filename-Basename.                |
| `cwd`             | string           | meistens | Windows-Pfad mit `\\`-Trennern.              |
| `gitBranch`       | string           | meistens | Branch zum Zeitpunkt der Zeile (kann `""` sein). |
| `version`         | string           | meistens | Claude-Code-Version, z.B. `"2.1.84"`.        |
| `entrypoint`      | string           | meistens | Meist `"cli"`.                               |
| `userType`        | string           | meistens | Meist `"external"`.                          |
| `isSidechain`     | bool             | meistens | `true` für Sub-Agent-Messages (§5).          |
| `isMeta`          | bool             | optional | `true` markiert injected Text (§5).          |
| `slug`            | string           | häufig   | Kurz-Identifier der "Turn-Phase"; nicht billing-relevant. |

**Was fehlt, darf fehlen.** File-level Meta-Zeilen wie `file-history-snapshot` oder
`pr-link` haben oft nur einen Bruchteil der obigen Felder.

---

## 4. Zeilentypen

Aus 38 Files empirisch beobachtet (Häufigkeit absteigend):

| `type`                 | Häufigkeit | Billing-relevant? | Rolle                                  |
|------------------------|------------|-------------------|----------------------------------------|
| `assistant`            | 1946       | **JA** (einzige)  | API-Response von Claude                |
| `user`                 | 1334       | nein              | Nutzer-Input oder Tool-Result          |
| `progress`             | 828        | nein              | Hook-/Tool-Fortschritt                 |
| `file-history-snapshot`| 180        | nein              | Tool-Backup-Metadaten                  |
| `system`               | 76         | nein              | Turn-Dauer, Compact-Boundary, etc.     |
| `queue-operation`      | 50         | nein              | Task-Queue-Events                      |
| `last-prompt`          | 14         | nein              | Pointer auf letzten Prompt             |
| `permission-mode`      | 13         | nein              | Permission-Mode-Wechsel                |
| `attachment`           | 12         | nein              | Datei-Attachments des Users            |
| `pr-link`              | 8          | nein              | GitHub-PR-Referenz                     |
| `custom-title`         | 3          | nein              | Session-Titel-Override                 |
| `agent-name`           | 3          | nein              | Sub-Agent-Name-Deklaration             |

**Konsequenz:** Für Token-Zählung reicht es, ausschließlich `assistant`-Zeilen zu
parsen. Alle anderen Typen liefern Kontext (Projekt, Session, Zeit), kein Geld.

### 4.1 `assistant`

Der einzig billing-relevante Typ. Kern-Layout:

```jsonc
{
  "type": "assistant",
  "uuid": "369b3445-…",
  "parentUuid": "b6d16a92-…",
  "timestamp": "2026-04-13T08:58:36.395Z",
  "sessionId": "2cce88f0-…",
  "cwd": "D:\\.claude\\projects\\hotline",
  "gitBranch": "main",
  "version": "2.1.84",
  "requestId": "req_011Ca1VbX3FhHAxWsmdzCog3",
  "isSidechain": false,
  "message": {
    "id": "msg_017w6T5a5egNZaZedrsFt8PY",
    "type": "message",
    "role": "assistant",
    "model": "claude-opus-4-6",
    "content": [
      { "type": "thinking", "thinking": "...", "signature": "..." },
      { "type": "text",     "text": "..." },
      { "type": "tool_use", "id": "toolu_...", "name": "Read", "input": { … } }
    ],
    "stop_reason": "tool_use",   // "end_turn" | "tool_use" | "stop_sequence"
    "stop_sequence": null,
    "stop_details": null,
    "usage": {
      "input_tokens": 3,
      "output_tokens": 36,
      "cache_creation_input_tokens": 5611,
      "cache_read_input_tokens": 11390,
      "cache_creation": {
        "ephemeral_5m_input_tokens": 0,
        "ephemeral_1h_input_tokens": 5611
      },
      "service_tier": "standard",
      "inference_geo": "not_available",
      "server_tool_use": { "web_search_requests": 0, "web_fetch_requests": 0 },
      "iterations": null,
      "speed": null
    }
  }
}
```

**Invariante:** In unserem Datensatz hatten **alle 1946 `assistant`-Zeilen** das
`message.usage`-Objekt. Wir verlassen uns darauf nicht blind — Parser gibt
`input_tokens = 0` als Default, wenn das Feld fehlt, und loggt den Vorfall.

**Content-Block-Typen (beobachtet):** `text`, `thinking`, `tool_use`. Alle sind
billing-irrelevant, die Tokens stehen bereits im `usage`-Objekt aggregiert.

### 4.2 `user`

Trägt Nutzer-Prompts **und** Tool-Results. Kein `usage` — User-Input wird über das
nächste `assistant.usage.input_tokens` gebillt (inkl. Cache-Read der Historie).

`message.content` kann sein:
- **`string`** — plain text, z.B. `"Schritt 2"`.
- **Array von Blocks** — typisch `{ "type": "tool_result", … }`, gelegentlich
  `{ "type": "text", … }` oder `{ "type": "document", … }` (File-Uploads).

### 4.3 `system`

`subtype` beobachtet: `turn_duration` (häufig), `stop_hook_summary`, `local_command`,
`compact_boundary`, `away_summary`.

Besonders interessant: `compact_boundary` markiert eine **Kontext-Kompaktion**. Nach
einer Kompaktion gibt die nächste `assistant`-Zeile hohe `input_tokens` aus (der
kompakte Kontext wird bezahlt, aber nicht mehr aus Cache-Read). Der Scanner markiert
dies nicht speziell — Token-Zählung ist monoton und wird durch Compact-Boundaries nicht
gestört.

### 4.4 Sonder-Zeilen

- **`file-history-snapshot`** — pro `messageId` ein Eintrag der Tool-Backup-Metadaten.
  Ignorieren.
- **`pr-link`, `custom-title`, `agent-name`** — reiner Kontext, keine Tokens.
- **`progress`** — Hook-Callbacks, Tool-Fortschritts-Meldungen. Ignorieren.

---

## 5. Wichtige Flags

### `isSidechain: true` → Sub-Agent-Message

Sub-Agenten (Task-Tool) führen eigene Inner-Chats. Deren `assistant`-Zeilen haben
`isSidechain: true` und oft `agentId`. **Token zählen zur User-Session** (der Nutzer
bezahlt, unabhängig von der Agent-Schicht). Der Scanner mergt sie deshalb
in die Session-Aggregation ein, ohne Sonderbehandlung.

Einzige Unterscheidung, die Ignis später anbietet: ein optionaler Breakdown
"Main vs. Sidechain" pro Session (post-MVP, in v0.2 ggf.).

### `isMeta: true` → injected Text, keine echte User-Interaktion

Markiert Zeilen, deren Inhalt automatisch eingefügt wurde (Skill-Bodys,
`<local-command-caveat>`-Hinweise). Diese `user`-Zeilen haben kein `usage` und sollten
für **Session-Counting** (z.B. "Wie viele Prompts hast du heute gesendet?") nicht
mitzählen. Ignorieren in allen Aggregationen ausgenommen Debugging.

### `<synthetic>`-Modell → Error-Pseudo-Message

Beobachtet: `message.model === "<synthetic>"` zusammen mit `isApiErrorMessage: true`
und einem `error`-Feld (z.B. `"authentication_failed"`). Alle Token-Werte sind `0`.

**Verhalten:** Zeilen mit `"model": "<synthetic>"` oder `"isApiErrorMessage": true`
werden **nicht** gebillt, nicht im Model-Breakdown gezählt, und **auch nicht in einer
"Error"-Statistik** im MVP — stumm überspringen, Warning-Log.

### `service_tier: null` & `isApiErrorMessage`

Nur bei synthetischen Error-Messages. Nicht als Billing-Tier behandeln.

---

## 6. Modelle (empirisch beobachtet)

| Model-ID                         | Count | Anmerkung                                    |
|----------------------------------|-------|----------------------------------------------|
| `claude-sonnet-4-6`              | 821   |                                              |
| `claude-opus-4-6`                | 754   |                                              |
| `claude-haiku-4-5-20251001`      | 266   | **mit Datums-Suffix** — andere ohne!         |
| `claude-opus-4-7`                | 104   |                                              |
| `<synthetic>`                    |   1   | Error-Pseudo, nicht billen.                  |

**Konsequenz für Pricing-Lookup:** Die Map `model_id → price` muss Einträge **sowohl mit
als auch ohne Datum-Suffix** akzeptieren. Strategie: Lookup zuerst mit der kompletten
ID, bei Miss nach erstem Split-Prefix (alles vor dem letzten `-YYYYMMDD`) erneut. Bei
zweitem Miss: Warning im UI, Usage wird weiter erfasst (Tokens), Kosten erscheinen als
"—" oder "unpriced".

---

## 7. `usage`-Feld — Detail

Alle Werte sind Integers oder `null`. Pro `assistant`-Zeile genau **einmal** — keine
Stream-Chunks im persistierten JSONL.

| Feld                            | Typ   | Bedeutung                                          |
|---------------------------------|-------|----------------------------------------------------|
| `input_tokens`                  | int   | Frische Input-Tokens (excl. Cache-Read).           |
| `output_tokens`                 | int   | Vom Modell erzeugte Tokens.                        |
| `cache_read_input_tokens`       | int   | Tokens aus Cache gelesen (≈ 10 % Preis).           |
| `cache_creation_input_tokens`   | int   | Tokens beim Erstellen des Cache geschrieben.       |
| `cache_creation.ephemeral_5m_input_tokens` | int | Davon: 5-min-TTL-Cache.                        |
| `cache_creation.ephemeral_1h_input_tokens` | int | Davon: 1h-TTL-Cache (höherer Write-Preis).     |
| `service_tier`                  | string| Praktisch immer `"standard"`.                      |
| `inference_geo`                 | string| `"not_available"` oder Region-Code.                |
| `server_tool_use.web_search_requests` | int | Web-Search-Calls durch Claude.                |
| `server_tool_use.web_fetch_requests`  | int | Web-Fetch-Calls durch Claude.                 |
| `iterations`                    | int\|null | Nur in neueren Versionen; Rolle unklar.        |
| `speed`                         | null/num  | Nur in neueren Versionen; Rolle unklar.        |

**Rechnungs-Regel (MVP, einfach):**

```
cost = input_tokens         * input_price
     + output_tokens        * output_price
     + cache_read_input_tokens            * cache_read_price
     + cache_creation.ephemeral_5m_input_tokens * cache_write_5m_price
     + cache_creation.ephemeral_1h_input_tokens * cache_write_1h_price
```

Fallback falls `cache_creation.*` fehlt: `cache_creation_input_tokens` → `cache_write_5m_price`
(verhält sich wie 5m in Pricing-Tabellen ohne 1h-Unterscheidung).

`server_tool_use` wird im MVP gezählt (Anzeige "Web-Searches: N"), aber nicht monetär
gebillt — Pricing für Server-Tool-Calls ist nicht Teil der Standard-Token-Tabelle.

---

## 8. Sessions und Session-Blocks

- Eine **Session** = ein JSONL-File = ein `sessionId`. Start = Timestamp der ersten
  Zeile, Ende = Timestamp der letzten. "Aktiv" = File wurde in den letzten N Minuten
  modifiziert (Schwelle konfigurierbar, Default 5 min).
- **Session-Blocks** (5-Stunden-Billing-Windows von Anthropic-Plänen) sind **nicht
  direkt im JSONL erkennbar**. Es gibt keine expliziten "Block-Start"- oder
  "Block-Reset"-Zeilen. Eine Heuristik (gleitende 5h-Fenster ab erster Assistant-
  Zeile nach einer langen Pause) wurde im MVP **ausgeschlossen** (siehe ADR-010).
  Implementation wandert nach v0.2, nachdem wir echtes Verhalten über mehrere Block-
  Zyklen beobachtet haben.

---

## 9. Konsequenzen für Scanner-Design

1. **Ein File = Append-only.** Position-Tracking via Byte-Offset ist sicher (ADR-011).
2. **Nur `assistant`-Zeilen müssen voll geparst werden.** Andere Typen können im
   Fast-Pfad nach `type`-Feld geprüft und übersprungen werden (kein voller JSON-Parse
   nötig, wenn Performance relevant wird — erst in v0.2 optimieren).
3. **Session-State** = ein Record pro File: `(session_id, project_cwd, first_ts,
   last_ts, byte_offset, aggregated_usage)`.
4. **Globaler State** = `HashMap<PathBuf, SessionState>` für alle offenen
   JSONL-Files.
5. **File-Watching** via `notify` crate. Auf `Modify`-Event: Δ-Scan ab gespeichertem
   Offset. Auf `Create`: neue Session, ab Byte 0.
6. **Fehlerhafte Zeilen** werden pro Zeile abgefangen (defekter JSON → Warning, Zeile
   überspringen, Offset trotzdem weiterzählen). Nie abbrechen.
7. **`<synthetic>` und `isApiErrorMessage: true`** → beim Aggregieren überspringen.
8. **Projekt-Pfad** ausschließlich aus `cwd` des ersten `assistant`-Events im File
   bestimmen. Directory-Name nur als Grouping-Key im Dateisystem-Scan.

---

## 10. Beobachtete Version-Schwankungen

| Version    | Zeilen beobachtet | Auffälligkeiten                     |
|------------|-------------------|-------------------------------------|
| `2.1.84`   | 3704              | Baseline.                           |
| `2.1.87`   | 218               | Identisches Schema.                 |
| `2.1.101`  | 74                | `iterations`/`speed` häufiger gesetzt. |
| `2.1.112`  | 200               | Keine Breaking Changes in Usage.    |

Keine Migrations-Logik nötig. Neue Felder werden additiv toleriert.

---

## 11. Offene Fragen (für spätere Phasen)

- Was genau markieren `iterations` und `speed`? (Irrelevant für Kosten.)
- Gibt es Fälle, wo `cache_creation_input_tokens > 0`, aber beide
  `ephemeral_*_input_tokens === 0`? — bisher nicht beobachtet; Fallback aus §7 decks ab.
- Block-Reset-Detection: empirische Studie in v0.2.
- Wie lange bleibt eine JSONL offen, bevor Claude Code "schließt"? (Scanner braucht das
  nicht zu wissen — append-only reicht — aber für das "Active Session"-Flag relevant.)

---

*Letzte Aktualisierung: 2026-04-17 (Phase 0, initiale Untersuchung).*
