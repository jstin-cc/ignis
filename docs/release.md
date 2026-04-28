# Release-Prozess

## Einen Release erstellen

1. Version in `tray/src-tauri/tauri.conf.json` erhöhen (`"version": "x.y.z"`).
2. `PROGRESS.md` Changelog-Abschnitt `[Unreleased]` → `[x.y.z]` umbenennen, Datum eintragen.
3. Committen und pushen:
   ```
   git add tray/src-tauri/tauri.conf.json PROGRESS.md
   git commit -m "chore: release vx.y.z"
   git push
   ```
4. Tag setzen und pushen — löst den Release-Workflow aus:
   ```
   git tag vx.y.z
   git push origin vx.y.z
   ```
5. GitHub Actions baut die Installer und erstellt einen GitHub Release mit `latest.json`.

## Signing-Schlüssel einrichten (einmalig)

```bash
cargo tauri signer generate -w ~/.tauri/ignis.key
```

Ausgabe enthält den öffentlichen Schlüssel. Diesen in `tray/src-tauri/tauri.conf.json`
unter `plugins.updater.pubkey` eintragen.

In GitHub → Repository Settings → Secrets → Actions zwei Secrets anlegen:

| Secret                            | Wert                                     |
|-----------------------------------|------------------------------------------|
| `TAURI_SIGNING_PRIVATE_KEY`       | Inhalt von `~/.tauri/ignis.key`          |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Passwort (leer lassen = kein Passwort) |

## Rollback-Prozedur

Falls ein fehlerhafter Release gepusht wurde:

### Schritt 1 — Tag löschen

```bash
git tag -d vx.y.z
git push origin :refs/tags/vx.y.z
```

### Schritt 2 — GitHub Release archivieren

Den fehlerhaften Release auf GitHub als „Draft" markieren oder löschen
(GitHub Web UI: Releases → Edit → als Draft speichern oder Delete).

### Schritt 3 — latest.json korrigieren

Das Update-Manifest `latest.json` im letzten funktionierenden Release auf GitHub
muss der aktuelle Stand sein. Tauri-Action überschreibt es beim nächsten guten Release
automatisch.

Falls sofort ein Hotfix nötig ist:
1. `latest.json` des letzten guten Releases herunterladen.
2. Als neues Asset unter dem letzten guten Release hochladen.

### Schritt 4 — Hotfix-Release

```bash
git checkout main
# Fix einspielen …
git commit -m "fix: ..."
git push
git tag vx.y.(z+1)
git push origin vx.y.(z+1)
```

## SemVer-Regeln

Ignis folgt [Semantic Versioning 2.0.0](https://semver.org/).

| Versions-Typ | Wann |
|---|---|
| **MAJOR** (`x+1.0.0`) | Breaking Changes an `/v1/*`-API-Schema, Entfernen von CLI-Subcommands oder -Flags, Wechsel des Config-Formats ohne Migrations-Pfad |
| **MINOR** (`x.y+1.0`) | Neue API-Endpoints, neue CLI-Subcommands/-Flags, neue UI-Features — stets rückwärtskompatibel |
| **PATCH** (`x.y.z+1`) | Bug-Fixes, Security-Patches, Dokumentations-Updates ohne Verhaltensänderung |

### Changelog-Workflow

1. Neue Einträge kommen unter `### [Unreleased]` in `PROGRESS.md`.
2. Beim Taggen wird `[Unreleased]` → `[x.y.z] — YYYY-MM-DD` umbenannt.
3. Der `[Unreleased]`-Link am Ende der Datei wird auf den neuen Tag angepasst.

### Was ist ein Breaking Change?

- Feld aus API-Response entfernt oder umbenannt
- HTTP-Statuscode eines Endpoints geändert
- CLI-Flag entfernt oder Semantik geändert
- `config.json`-Schlüssel entfernt ohne automatische Migration

Additive Änderungen (neues Feld, neuer optionaler Parameter, neue Zeile in CLI-Output)
sind **kein** Breaking Change und erfordern nur ein Minor-Bump.

---

## SmartScreen-Hinweis (bis v2.0)

Installer sind mit Ed25519 für den Tauri-Updater signiert, aber **nicht** mit
einem Authenticode-Zertifikat. Windows SmartScreen zeigt beim Erstinstall eine
Warnung. Nutzer müssen „Weitere Infos → Trotzdem ausführen" klicken.

Hintergrund: ADR-016 — Authenticode wird erst ab v2.0 eingeführt.
