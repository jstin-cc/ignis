# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2: Notifications bei Limit-Schwellen.**

### Kontext

Der aktive Block hat `percent_elapsed`. Wenn 80% oder 100% erreicht werden, soll eine
Windows-Benachrichtigung ausgelöst werden. Tauri 2 stellt `tauri-plugin-notification`
bereit.

### Schritte

1. **`tray/src-tauri/Cargo.toml`** — `tauri-plugin-notification` hinzufügen.

2. **`tray/src-tauri/src/main.rs`** — Plugin registrieren:
   `tauri::Builder::default().plugin(tauri_plugin_notification::init())`

3. **`tray/src/hooks/useBlockNotifications.ts`** — neuer Hook:
   - Speichert welche Schwellen (80%, 100%) für den aktuellen Block bereits gefeuert wurden
     (via `useRef` — kein Re-Render nötig).
   - Wenn `percent_elapsed >= 80` und noch nicht gemeldet → Notification.
   - Wenn `percent_elapsed >= 100` / Block abgelaufen → Notification.
   - Beim Block-Wechsel (neue `start`-Zeit) → fired-Set zurücksetzen.

4. **`tray/src/App.tsx`** — `useBlockNotifications(activeBlock)` aufrufen.

Danach: Auto-Start bei Windows-Login (optional, Tauri `autostart`-Plugin).
