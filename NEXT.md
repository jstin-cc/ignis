# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2: Auto-Start bei Windows-Login (optional via Tray-Einstellungen).**

### Kontext

Die Tray-App soll optional beim Windows-Login automatisch starten. Tauri 2 bietet
`tauri-plugin-autostart` dafür. Der Nutzer soll Auto-Start im Tray ein-/ausschalten
können (Settings-Button ⚙ in der Header-Leiste).

### Schritte

1. **`tray/src-tauri/Cargo.toml`** — `tauri-plugin-autostart = "2"` hinzufügen.

2. **`tray/src-tauri/src/main.rs`** — Plugin registrieren:
   `.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))`
   Tauri-Command `toggle_autostart` → `enable()` / `disable()` / `is_enabled()` wrappen.

3. **`tray/src-tauri/capabilities/default.json`** — `"autostart:allow-enable"`,
   `"autostart:allow-disable"`, `"autostart:allow-is-enabled"` hinzufügen.

4. **`tray/src/hooks/useAutoStart.ts`** — Hook:
   - `isEnabled: boolean` State
   - `toggle()` Funktion (ruft Tauri-Command auf)
   - Initialer Zustand beim Mount abfragen

5. **`tray/src/App.tsx`** — Settings-Button (⚙) öffnet ein kleines Inline-Panel
   mit Auto-Start-Toggle (Checkbox oder Button).

Danach: Phase 2 abschließen, v0.2.0 Tag setzen.
