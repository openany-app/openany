# Geräte in der Nähe: LocalSend statt Wi-Fi Aware

**Stand 13.09.2026 — geprüft, nicht gebaut.** Erst wird die Webapp
abgeschlossen, danach geht es hier weiter. Versionen vor dem Bau gegenprüfen.

## Worum es geht

Geräte, auf denen dieses Programm läuft, sollen sich ohne Server
untereinander finden — für Dateiaustausch und Chat innerhalb eines Projekts.
**Nur hier, nicht in der Webapp:** Die Webapp ist immer am Netz, offline
ergibt sie wenig Sinn, also braucht sie kein P2P.

## Warum nicht Wi-Fi Aware

Am 12.09.2026 zurückgestellt, frühestens Herbst 2027 neu bewerten:
iOS beantwortet nach dem Pairing die NDP-Requests von Android nicht (Apple-
Tickets FB19568037, FB19570341, FB19683706), das Spec-4.0-Pairing fehlt auf
vielen aktuellen Android-Geräten, und Apple vergibt das Entitlement einzeln.

## Warum LocalSend passt

LocalSend setzt nur ein gemeinsames WLAN voraus — damit verstehen sich iOS
und Android, ohne Hardware-Lotterie.

| | |
|---|---|
| Protokoll | [v2.2](https://github.com/localsend/protocol), offen dokumentiert |
| Finden | UDP-Multicast `224.0.0.167:53317`, Rückfall `POST /api/localsend/v2/register` an alle Adressen im Subnetz |
| Übertragen | HTTPS/REST: `prepare-upload`, `upload`, `cancel` |
| Vertrauen | Fingerabdruck = SHA-256 des selbst erstellten Zertifikats, optional PIN (`?pin=`) |
| Code | [localsend/localsend](https://github.com/localsend/localsend) v1.18.2 (21.08.2026), Apache-2.0 |
| **Rust** | Crate `localsend` unter `packages/core`, Features `discovery`, `http`, `multicast` — **benutzen statt nachbauen** |

Nebeneffekt: Dateien gingen dann auch mit jeder gewöhnlichen LocalSend-App hin
und her.

## Was LocalSend nicht löst

* **Kein Chat.** Das Protokoll kennt keinen Verlauf, keine Gruppen, keinen
  Projektbezug. Chat und Projektdaten laufen über den eigenen Abgleich;
  LocalSend liefert nur das Finden und das Vertrauen.
* **Nur eine Gegenstelle.** Ein Gerät in der Nähe ist für den Abgleich einfach
  eine weitere `base_url`. Die echte Lücke liegt drüben: `SyncRunner::basis()`
  kennt nur *eine* Gegenstelle, obwohl `sync_state` mit
  `unique(user_id, base_url)` mehrere könnte.
* **Abgeschottete WLANs.** AP-Isolation (Gäste-, Schul-WLAN) sperrt alles;
  dann bleibt ein Hotspot.
* **Telefon nur im Vordergrund.** Der kleine Server läuft, solange die App
  offen ist.

## Plattformgrenzen

Gehört hinter einen Trait, nicht in `crates/`:

* **Android:** `WifiManager.MulticastLock`, sonst kommen keine
  Multicast-Pakete an.
* **iOS:** Entitlement `com.apple.developer.networking.multicast` bei Apple
  beantragen (bis ~2 Wochen), dazu `NSLocalNetworkUsageDescription` und die
  Local-Network-Abfrage. Ohne Entitlement bleibt nur der HTTP-Rückfall.

## Erster Schritt, wenn es dran ist

Ein Versuch, kein Feature: zwei Geräte — ein iPhone, ein Android — finden sich
per Multicast und melden sich an. Das zeigt schnell, ob es in echten WLANs
trägt.
