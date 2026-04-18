# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Installer (MSI via Tauri Bundler) + Release-Tag `v0.1.0-mvp`.**

### Schritte

1. Tray-Icon-Assets erzeugen: Platzhalterpng (32x32, 128x128, 128x128@2x) in
   `tray/src-tauri/icons/` — Tauri Bundler erwartet diese für MSI/NSIS.
   Quickest path: `tauri icon` CLI-Befehl auf einem Quell-PNG.

2. Vor dem Bundle sicherstellen, dass `tray/src-tauri/tauri.conf.json` korrekte Pfade hat
   und `bundle.active = true` gesetzt ist.

3. `cargo tauri build` aus dem `tray/`-Verzeichnis (Windows-Runner oder CI) ausführen.
   Erzeugt `.msi` und `.exe`-Installer unter `tray/src-tauri/target/release/bundle/`.

4. GitHub Release erstellen:
   ```
   git tag v0.1.0-mvp
   git push origin v0.1.0-mvp
   gh release create v0.1.0-mvp --title "v0.1.0-mvp" --notes "MVP: Scanner, CLI, HTTP-API, Tray"
   ```
   Installer-Artefakte als Release-Assets hochladen.

5. CHANGELOG.md aktualisieren (Keep-a-Changelog, Abschnitt `[0.1.0-mvp]`).

**Voraussetzung:** Schritt 1–3 müssen auf einem Windows-Build-Runner laufen
(lokal oder via GitHub Actions `windows-latest`). Auf dem aktuellen Linux-Host
sind GTK-Systemlibs für den Tauri-Linux-Build nicht installiert — das ist kein
Problem, da das Deployment-Target Windows 11 ist.
