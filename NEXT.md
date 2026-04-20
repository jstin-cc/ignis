# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 3, Schritt 4: Auto-Update via Tauri Updater — dann v1.0.0.**

### Kontext

`tauri-plugin-updater` prüft beim Start gegen einen Endpunkt, ob eine neuere Version
verfügbar ist, und lädt das Update im Hintergrund herunter.

Weil kein öffentlicher Server vorhanden ist, reicht für v1.0.0 eine minimale
Implementierung: Plugin einbinden, Command `check_update` anlegen, im UI einen
"Nach Updates suchen"-Button im Settings-Panel ergänzen.

### Schritte

1. **`tray/src-tauri/Cargo.toml`** — `tauri-plugin-updater = "2"` hinzufügen.
2. **`tray/src-tauri/capabilities/default.json`** — `"updater:default"` ergänzen.
3. **`tray/src-tauri/src/main.rs`** — Plugin registrieren; Command `check_for_update`
   — ruft `app.updater()?.check().await` auf, gibt `{ available: bool, version: String }`
   zurück.
4. **`tray/src/hooks/useUpdater.ts`** — Hook: `checkForUpdate()` Funktion,
   `updateInfo: { available: bool, version: string } | null` State.
5. **`tray/src/App.tsx`** — Settings-Panel: Button "Updates prüfen" + Statuszeile.
6. **`tray/src-tauri/tauri.conf.json`** — `updater.endpoints` auf Platzhalter setzen
   (z.B. `https://github.com/jstin-cc/winusage/releases/latest/download/latest.json`).

Danach: CHANGELOG v1.0.0 schreiben, Tag setzen.
