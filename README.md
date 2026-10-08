# Openany – die App

**[⬇ APK herunterladen](../../releases/latest)** (Android) – fertig gebaut und signiert, selbst bauen ist nicht nötig.

Notizen, Kalender, Speicher, Galerie, Adressbuch, Nachrichten und Projekte –
**auf deinem Gerät, ohne Konto und ohne Server.** Eigene Geräte gleichen
sich im selben WLAN direkt untereinander ab, und eine verschlüsselte
Sicherungsdatei nimmt alles auf ein anderes Gerät mit.

*English: Openany is a local-first app for notes, calendar, files, photos,
contacts, messages and projects. It works without an account and without a
server.*

## Herunterladen

Fertige Pakete stehen unter [Releases](../../releases): Beim neuesten Release
unter „Assets" die Datei `openany-android-….apk` laden und auf dem Gerät
öffnen; Android fragt einmal, ob Installationen aus dieser Quelle erlaubt
sind. Jede Plattform hat ihre eigene Versionsnummer; ein Release heißt
deshalb etwa „Android 1.0.0". Neben jeder APK steht ihre SHA-256-Prüfsumme.

## Was drin ist

| Ordner | Inhalt |
|---|---|
| `app/` | die App: Oberfläche (Vue) und Programm (Rust, Tauri 2) |
| `app/crates/` | die Bausteine – Speicher, Abgleich, Geräte in der Nähe, E-Mail/OpenPGP, Sicherung, Anmeldung (`anyid-client`) |
| `packages/oberflaeche/` | die Oberfläche, die App und Webapp von openany.de teilen |
| `matrix/core/` | der Matrix-Kern (Nachrichten, Ende-zu-Ende verschlüsselt) |

## Selbst bauen

Gebraucht werden Rust (stable), Node.js 22, für Android das Android-SDK mit
NDK und Java 21 sowie die Tauri-Kommandozeile 2 (`cargo install tauri-cli --version "^2"`).

```bash
cd app
npm ci
cargo test                                   # die Bausteine prüfen
cargo tauri android build --apk --target aarch64
```

Ohne eigenen Signaturschlüssel entsteht eine unsignierte APK. Wie die
Signatur eingebunden wird, steht in
`app/src-tauri/gen/android/app/build.gradle.kts`.

## Lizenz

Copyright © 2026 openany.de

Die App steht unter der **Elastic License 2.0** (SPDX: `Elastic-2.0`), siehe
[LICENSE](LICENSE). Kurz gefasst: Lesen, nutzen, verändern und weitergeben
ist erlaubt. Nicht erlaubt ist, sie als gehosteten oder verwalteten Dienst
anderen anzubieten, Lizenzschlüssel-Funktionen zu umgehen oder Lizenz- und
Urheberhinweise zu entfernen. Maßgeblich ist allein der Text in `LICENSE`.

## Kontakt

info@openany.de

Entwickelt wird in einem nicht öffentlichen Repository; hier erscheint zu
jedem Release der Stand der App.
