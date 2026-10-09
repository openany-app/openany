# Openany

## Download

Built and signed – no need to compile anything yourself.

| Platform | Download | |
|---|---|---|
| **Android** 7+ (64-bit ARM) | [⬇ APK](../../releases/download/android-v1.0.0/openany-android-1.0.0.apk) | [all files](../../releases/tag/android-v1.0.0) |
| **Windows** 10/11 (64-bit) | [⬇ Installer](../../releases/download/windows-v1.0.0/openany-windows-1.0.0-setup.exe) | [all files](../../releases/tag/windows-v1.0.0) |
| **macOS** 11+ (Intel and Apple silicon) | [⬇ DMG](../../releases/download/macos-v1.0.1/openany-macos-1.0.1-universal.dmg) | [all files](../../releases/tag/macos-v1.0.1) |
| **Linux** (x86-64) | [⬇ AppImage](../../releases/download/linux-v1.0.0/openany-linux-1.0.0-x86_64.AppImage) | [all files](../../releases/tag/linux-v1.0.0) (also `.deb`) |

Notes, calendar, files, photos, contacts, messages and projects – **on your
device, without an account and without a server.** Your own devices sync
directly over the same Wi-Fi, and an encrypted backup file takes everything
to another device.

## Features

**Notes**
- Markdown editor with toolbar and shortcuts, autosave
- Nested folders, full-text search
- Wikilinks with backlinks, nested #tags, graph view and automatic index
- Citations from a CSL-JSON library (e.g. exported from Zotero)
- Images and files inside notes, PDF export

**Storage**
- Gallery for photos and videos with nested albums, capture date and location, camera buttons
- Files in folders and documents in nested binders
- Photographed documents become searchable PDFs – text recognition runs on the device
- Full-text search in PDFs, Word, OpenDocument, text and Markdown files
- PDF viewer: fill in forms, highlight, draw, add text and comments

**Calendar**
- Multiple calendars with colours, month and week view
- All-day, multi-day and recurring events
- Subscriptions to external ICS calendars

**Projects – nearby, without a server**
- Invite members in person; both devices show the same six digits to confirm
- Project chat, shared note folders (read or edit), folders and albums
- Planning: scheduling, polls, shift plans, bring and guest lists, Kanban boards, roadmaps, places on OpenStreetMap
- School planning: holidays, timetable, care plan, subjects

**Messages**
- Matrix with end-to-end encryption, several accounts
- Email through your own mailbox (IMAP/SMTP) – with OpenPGP encryption and signatures
- Messages to devices nearby, without any server; strangers can only send requests

**Address book**
- Any number of phone numbers, addresses and channels per contact, each with its own label

**Your devices and your data**
- Pair your own devices and sync them directly over Wi-Fi
- Encrypted backup file ([age](https://age-encryption.org)) with a six-word passphrase – restore on another device
- Credentials and keys locked in the Android Keystore
- Instant notifications without Google services
- Seven languages (English, German, Spanish, French, Polish, Portuguese, Ukrainian), light and dark mode, two designs

The full manual is in **[help/README.md](help/README.md)**.

## Installing

- **Android:** open the APK on your device; Android asks once whether
  installs from this source are allowed.
- **Windows:** run the installer. SmartScreen may warn because the app is
  not code-signed yet – choose "More info" → "Run anyway".
- **macOS:** open the DMG and drag Openany to Applications. The app is not
  notarised yet – on first launch right-click it and choose "Open".
- **Linux:** make the AppImage executable and start it, or install the
  `.deb` on Debian/Ubuntu.

The desktop apps update themselves. Each platform has its own version
number, so a release is called e.g. "Windows 1.0.0". Every file comes with
its SHA-256 checksum; see [all releases](../../releases).

## What's inside

| Folder | Contents |
|---|---|
| `app/` | the app: user interface (Vue) and program (Rust, Tauri 2) |
| `app/crates/` | the building blocks – storage, sync, nearby devices, email/OpenPGP, backup, sign-in (`anyid-client`) |
| `packages/oberflaeche/` | the user interface shared with the Openany web app |
| `matrix/core/` | the Matrix core (end-to-end encrypted messages) |
| `help/` | the manual, generated from the app's help texts |

The code and its comments are mostly in German.

## Building it yourself

You need Rust (stable), Node.js 22, and for Android the Android SDK with NDK,
Java 21 and the Tauri CLI 2 (`cargo install tauri-cli --version "^2"`).

```bash
cd app
npm ci
cargo test                                   # check the building blocks
cargo tauri android build --apk --target aarch64
```

Without your own signing key the result is an unsigned APK. How signing is
wired in is described in `app/src-tauri/gen/android/app/build.gradle.kts`.

## Licence

Copyright © 2026 openany.de

Openany is licensed under the **Elastic License 2.0** (SPDX: `Elastic-2.0`),
see [LICENSE](LICENSE). In short: you may read, use, modify and redistribute
it. You may not offer it to others as a hosted or managed service, circumvent
licence key functionality, or remove licence and copyright notices. Only the
text in `LICENSE` is binding.

## Contact

info@openany.de

Development happens in a non-public repository; this repository receives the
state of the app with each release.
