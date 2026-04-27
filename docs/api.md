# HTTP API

Lokaler HTTP-Server, damit Statuslines, Editor-Plugins oder eigene Scripts den aktuellen
Ignis-Zustand auslesen können.

- **Bind:** `127.0.0.1:7337` (Port in Config anpassbar). Niemals `0.0.0.0` — das
  Server-Binary verweigert den Start, falls ein anderer Host konfiguriert wird (ADR-005).
- **Auth:** Bearer-Token + Origin-Check (§3).
- **Lesen only.** Es gibt keine mutierenden Endpoints im MVP.

---

## 1. Endpoints (MVP)

| Method | Path              | Response                     |
|--------|-------------------|------------------------------|
| GET    | `/health`         | 200 `{"ok": true, …}`        |
| GET    | `/v1/summary`     | 200 `Summary`                |
| GET    | `/v1/sessions`    | 200 `{ "sessions": […] }`    |

Alle Responses `Content-Type: application/json; charset=utf-8`.

### 1.1 `GET /health`

Ungeschützt (kein Auth nötig). Nur für Liveness-Checks von Plugins.

```json
{
  "ok": true,
  "version": "0.1.0",
  "snapshot_age_ms": 742,
  "warnings": ["unknown_model:claude-opus-5-0"]
}
```

- `snapshot_age_ms` = `now - snapshot.taken_at`. Als Proxy dafür, wie aktuell die
  Daten sind.
- `warnings` listet Pricing-Misses aggregiert; leeres Array, wenn alle Modelle
  bepreist sind.

### 1.2 `GET /v1/summary`

**Auth erforderlich** (§3).

Query-Parameter:

| Name    | Werte                             | Default   | Bedeutung             |
|---------|-----------------------------------|-----------|-----------------------|
| `range` | `today` \| `week` \| `month` \| `30days` \| `all` | `today`   | Zeitfenster. `30days` = rolling 30 Tage (heute + 29 vorangegangene); `month` = Kalender-Monat. |

Response:

```json
{
  "range": "today",
  "taken_at": "2026-04-17T12:30:55Z",
  "total_cost_usd": "2.4312",
  "total_tokens": 1218432,
  "event_count": 96,
  "by_model": [
    {
      "model": "claude-opus-4-6",
      "input_tokens": 1800,
      "output_tokens": 41230,
      "cache_read_tokens": 890000,
      "cache_creation_tokens": 56200,
      "cost_usd": "1.8274",
      "event_count": 54
    },
    {
      "model": "claude-sonnet-4-6",
      "input_tokens": 620,
      "output_tokens": 18200,
      "cache_read_tokens": 210000,
      "cache_creation_tokens": 1300,
      "cost_usd": "0.6038",
      "event_count": 42
    }
  ],
  "by_project": [
    {
      "project_path": "D:\\.claude\\projects\\winusage",
      "total_tokens": 612000,
      "total_cost_usd": "1.4812",
      "session_count": 3
    }
  ],
  "active_session": {
    "session_id": "35bf8651-bda1-471b-9117-87f8a6817ca0",
    "project_path": "D:\\.claude\\projects\\winusage",
    "git_branch": "main",
    "first_seen": "2026-04-17T11:58:01Z",
    "last_seen": "2026-04-17T12:30:48Z",
    "event_count": 42,
    "total_cost_usd": "0.71"
  },
  "pricing_warnings": []
}
```

**Geldbeträge sind Strings, nicht floats.** Server serialisiert `rust_decimal::Decimal`
als String, Clients parsen sie als Decimal/BigDecimal. Grund: JSON-Number-Floats
verlieren Präzision bei < 0.01 USD.

**Token-Zählungen sind Integers** (u64).

### 1.3 `GET /v1/sessions`

**Auth erforderlich.**

Query-Parameter:

| Name     | Werte                             | Default   | Bedeutung                       |
|----------|-----------------------------------|-----------|---------------------------------|
| `active` | `true` \| `false`                 | —         | Wenn gesetzt, filtert auf (in-)active. |
| `limit`  | 1..500                            | 100       | Max. Anzahl.                    |

Response:

```json
{
  "taken_at": "2026-04-17T12:30:55Z",
  "sessions": [
    {
      "session_id": "35bf8651-bda1-471b-9117-87f8a6817ca0",
      "project_path": "D:\\.claude\\projects\\winusage",
      "git_branch": "main",
      "first_seen": "2026-04-17T11:58:01Z",
      "last_seen": "2026-04-17T12:30:48Z",
      "is_active": true,
      "event_count": 42,
      "total_cost_usd": "0.71",
      "by_model": [
        { "model": "claude-opus-4-6", "cost_usd": "0.62", "tokens": 298000 },
        { "model": "claude-sonnet-4-6", "cost_usd": "0.09", "tokens": 14000 }
      ]
    }
  ]
}
```

---

## 2. Fehler-Responses

Alle Fehler folgen dem Shape:

```json
{
  "error": {
    "code": "auth_required",
    "message": "Bearer token missing or invalid."
  }
}
```

| HTTP | `code`              | Bedeutung                                        |
|------|---------------------|--------------------------------------------------|
| 400  | `bad_request`       | Ungültige Query-Parameter.                       |
| 401  | `auth_required`     | Token fehlt oder ungültig.                       |
| 403  | `origin_rejected`   | `Origin`-Header nicht in Allowlist.              |
| 404  | `not_found`         | Unbekannter Pfad.                                |
| 500  | `internal`          | Unerwarteter Server-Fehler.                      |
| 503  | `no_snapshot_yet`   | Initial-Scan läuft noch.                         |

---

## 3. Authentifizierung und Origin-Check (ADR-005)

### 3.1 Bearer-Token

- Beim ersten Start generiert das Binary ein Token (32 Bytes, base64-url) und legt es
  unter `%APPDATA%\ignis\auth-token.txt` mit restriktiven ACLs ab:
  - Eigentümer: aktueller User, Inherit: aus.
  - ACL: nur Owner hat Read/Write; SYSTEM optional.
  - Implementation via `icacls` im Installer + defensiver Runtime-Check (`GetFileSecurity`).
- Clients setzen `Authorization: Bearer <token>`.
- Header fehlt oder ungleich → 401.
- Rotation: CLI-Kommando `ignis token rotate` überschreibt die Datei (Phase 1,
  optional; im allerersten MVP nur manuelles Löschen + Neustart).

### 3.2 Origin-Allowlist

- Default-Allowlist: leer (`[]`). Das bedeutet: Requests, die **keinen** `Origin`-Header
  senden (CLI-curl, Editor-Extensions im Node-Runtime), sind erlaubt. Requests mit
  `Origin`-Header werden gegen die Allowlist geprüft; Mismatch → 403.
- Zweck: Browser-basierte CSRF-Attacks (bösartige Webseite fetcht den lokalen API)
  werden abgefangen, ohne den Plugin-Use-Case zu brechen.
- Konfiguration per Config-Datei:
  ```
  %APPDATA%\ignis\config.toml
    [api]
    origin_allowlist = ["vscode-webview://*", "cursor://*"]
  ```

### 3.3 CORS

`/health` sendet `Access-Control-Allow-Origin: *` (unkritisch, nur Liveness).
`/v1/*` sendet **keine** CORS-Header — Browser-Clients sind nicht der Zielkanal. Wer
einen Browser-Client bauen will, setzt in der Config eine explizite `origin_allowlist`;
erst dann werden entsprechende CORS-Header pro Allowed-Origin ausgeliefert.

---

## 4. Versionierung

- Path-Präfix `/v1/`. Breaking Changes ziehen `/v2/` nach; `/v1/` bleibt so lange
  parallel erhalten, bis keine Clients mehr darauf zugreifen (wir loggen Hits).
- Additive Änderungen (neue Felder) sind innerhalb `/v1/` erlaubt — Clients müssen
  unbekannte Felder tolerieren.

---

## 5. Nicht-Ziele (MVP)

- Keine WebSocket/SSE-Push-Kanäle. Clients pollen.
- Keine `POST/PUT/DELETE`. Pricing-Overrides etc. laufen ausschließlich über Dateien.
- Keine Auth-Schemes außer Bearer. Kein OAuth, kein mTLS.
- Keine Response-Caches mit `ETag`/`If-None-Match`. (Snapshot ist ohnehin billig zu
  serialisieren.)

---

## 6. Implementation-Hinweise

- Framework: **Axum** auf Tokio.
- Shared State: `Arc<ArcSwap<Snapshot>>` + `Arc<Config>`.
- Tests: jeder Endpoint hat mindestens einen `tower::ServiceExt`-Integration-Test,
  gefüttert mit fixture-basierten Snapshots.
- Graceful-Shutdown bei `Ctrl+C` / Service-Stop-Signal.
