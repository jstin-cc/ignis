# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**`src/config.rs` + `examples/scan.rs` implementieren.**

### 1. `src/config.rs`

Lädt Laufzeit-Konfiguration — Pfade und Auth-Token.

```rust
pub struct Config {
    /// Root-Verzeichnis der Claude-Logs, z.B. %USERPROFILE%\.claude\projects
    pub claude_projects_dir: PathBuf,
    /// Bearer-Token für HTTP-API (zufällig generiert, in config-Datei gespeichert)
    pub api_token: String,
}

impl Config {
    /// Lädt aus `%APPDATA%\winusage\config.toml` oder legt Defaults an.
    pub fn load() -> Result<Self, ConfigError>;
}
```

- Kein extra Crate nötig — einfaches TOML-ähnliches Format via `serde_json` oder manuell.
  Alternativ: `toml = "0.8"` hinzufügen (prüfen ob passt).
- `claude_projects_dir` default: `%USERPROFILE%\.claude\projects`
- `api_token` default: zufälliger 32-Byte-Hex-String (einmalig generiert, danach aus Datei gelesen).
- Fehlender `%USERPROFILE%`-Envvar → `ConfigError`.

### 2. `examples/scan.rs`

Dev-CLI für schnelle Verifikation:

```
cargo run --example scan
```

Gibt JSON-Dump auf stdout aus:

```json
{
  "scanned_files": 12,
  "total_events": 87,
  "today_cost_usd": "1.23",
  "pricing_warnings": []
}
```

Verwendet `scan_all(config.claude_projects_dir)` + `build_snapshot()` + `PricingTable`.

### Tests

- `Config::load()` mit `WINUSAGE_PROJECTS_DIR` env-override (statt hardcodiertem Pfad).
- `Config` → fehlender Envvar → sinnvoller Fehler.

Danach: HTTP-API (`src/api.rs`) mit Axum.
