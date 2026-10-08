# @oberflaeche — die gemeinsame Oberfläche

Was die Webapp (`frontend/`) und das Programm (`app/`) **gleich** haben
sollen: Grundbausteine, Sprachen, netzfreie Helfer. Plan und Begründung:
[`docs/plan-app-neuaufsatz.md`](../../docs/plan-app-neuaufsatz.md).

## Die eine Regel

**Nichts hier spricht mit einem Server, einem Router oder einer Plattform.**
Kein `services/api`, kein `vue-router`, kein `@tauri-apps/api`. Was Daten
braucht, bekommt sie hineingereicht (eine `dataSource`, ein Rückruf). Nur so
kann dieselbe Datei im Browser, auf dem Schreibtisch und auf dem Telefon
laufen — und für Android, iOS, Windows, macOS und Linux gleich gebaut werden.

## Wie es eingebunden wird

Kein npm-Paket, sondern ein Vite-Alias. `npm ci` und `node_modules` bleiben,
wie sie sind; das Paket hat **keine eigenen Abhängigkeiten**.

```js
// vite.config.js
resolve: {
  alias: { '@oberflaeche': path.resolve(import.meta.dirname, '../packages/oberflaeche') },
  dedupe: ['vue', 'vue-i18n', 'lucide-vue-next'],
}
```

Drei Stellen, die sonst still brechen:

| | wenn es fehlt |
|---|---|
| `dedupe` | Das Paket findet `vue` nicht — oder zieht eine zweite Instanz, und gemeinsame refs greifen nicht |
| `@source "…/packages/oberflaeche"` im CSS | Klassen, die nur hier vorkommen, fehlen im Bündel. Kein Fehler, nur falsches Aussehen |
| `server.fs.allow` | Der Entwicklungsserver liefert die Dateien nicht aus |

Im Docker-Entwicklungscontainer der Webapp ist `./packages` zusätzlich nach
`/packages` eingehängt (`docker-compose.yml`).

## Prüfen

Tests und Lint laufen aus `frontend/` mit:

```bash
cd frontend && npx vitest run && npm run lint
```
