//! Was openany von Matrix braucht – und nichts darüber hinaus.
//!
//! **Diese Kiste kennt kein HTTP, kein Laravel und kein Tauri.** Sie nimmt
//! Zugangsdaten entgegen und gibt eine Sitzung zurück; wer sie ruft und
//! woher der Ruf kommt, ist ihr gleichgültig. Genau deshalb kann `openany-app`
//! sie später unverändert einbinden: Dort ruft ein Tauri-Command, wo heute
//! ein Axum-Handler ruft.
//!
//! **Warum so wenig Oberfläche.** Das SDK kann ungleich mehr – Räume
//! verwalten, Mitglieder einladen, Dateien senden. Jede Fähigkeit, die hier
//! durchgereicht wird, ist eine, die zwei Hüllen unterstützen müssen. Was
//! openany nicht anbietet, steht deshalb auch hier nicht.

use std::path::Path;

use anyhow::{Context, Result};
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::{
        events::room::message::{
            MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
        },
        OwnedDeviceId, OwnedEventId, OwnedUserId, UserId,
    },
};

/// Die App haelt den Client zwischen zwei Befehlen -- sie muss ihn also
/// benennen koennen, ohne selbst am SDK zu haengen.
pub use matrix_sdk::Client;

/// Was nach einer Anmeldung aufbewahrt werden muss, um sie wieder aufzunehmen.
///
/// **Kein Passwort.** Es wird einmal benutzt und ist danach vergessen. Was
/// bleibt, gilt für dieses eine Gerät: Wer es entwendet, kann lesen und
/// schreiben, aber das Konto nicht übernehmen.
#[derive(Debug, Clone)]
pub struct Sitzung {
    /// Die Kennung, wie der Homeserver sie nennt – nicht die, die jemand
    /// ins Formular getippt hat. Aus `tiffy` wird `@tiffy:matrix.org`.
    pub mxid: OwnedUserId,
    /// Das Gerät, das openany auf diesem Konto IST.
    pub device_id: OwnedDeviceId,
    pub access_token: String,
}

/// Die Zugangsdaten für genau eine Anmeldung.
pub struct Anmeldung<'a> {
    pub homeserver: &'a str,
    pub benutzer: &'a str,
    pub passwort: &'a str,
}

/// Eine wieder aufzunehmende Sitzung – das Gegenstück zu [`Sitzung`].
pub struct Wiederaufnahme<'a> {
    pub homeserver: &'a str,
    pub mxid: &'a str,
    pub device_id: &'a str,
    pub access_token: &'a str,
}

/// Wie openany sich in der Geräteliste eines Menschen zeigt.
///
/// Dieser Name steht in Element neben dem Gerät, und daran entscheidet
/// jemand, ob er es behält oder abmeldet. „openany" ist deshalb das
/// Mindeste; ein Zufallsname wäre eine Zumutung.
const GERAETENAME: &str = "openany";

/// Meldet ein Konto an und legt dabei ein neues Gerät an.
///
/// **Warum der Speicher hier schon entsteht.** Das SDK legt die
/// Verschlüsselungsschlüssel beim Anmelden an, nicht beim ersten Senden.
/// Ohne Speicher wären sie nach dem Rückgabewert verloren, und das Gerät
/// könnte nie wieder etwas entschlüsseln – auch nicht das, was es selbst
/// geschrieben hat.
pub async fn anmelden(daten: Anmeldung<'_>, speicher: &Path, passphrase: &str) -> Result<Sitzung> {
    let client = klient(daten.homeserver, speicher, passphrase).await?;

    client
        .matrix_auth()
        .login_username(daten.benutzer, daten.passwort)
        .initial_device_display_name(GERAETENAME)
        .await
        .context("Sign-in at the homeserver failed")?;

    sitzung_lesen(&client)
}

/// Nimmt eine bestehende Sitzung wieder auf, ohne ein neues Gerät anzulegen.
///
/// **Der Grund, aus dem `device_id` aufbewahrt wird.** Ohne sie entstünde bei
/// jedem Neustart des Dienstes ein weiteres Gerät, und der Nutzer sähe in
/// Element eine wachsende Liste von Karteileichen – jede davon eine, die er
/// einzeln verifizieren müsste.
pub async fn wiederaufnehmen(
    daten: Wiederaufnahme<'_>,
    speicher: &Path,
    passphrase: &str,
) -> Result<Client> {
    let client = klient(daten.homeserver, speicher, passphrase).await?;

    let mxid: OwnedUserId = daten.mxid.parse().context("unusable Matrix ID")?;
    let device_id: OwnedDeviceId = daten.device_id.into();

    client
        .restore_session(matrix_sdk::authentication::matrix::MatrixSession {
            meta: matrix_sdk::SessionMeta {
                user_id: mxid,
                device_id,
            },
            tokens: matrix_sdk::SessionTokens {
                access_token: daten.access_token.to_owned(),
                refresh_token: None,
            },
        })
        .await
        .context("Session could not be resumed")?;

    Ok(client)
}

/// Schickt einen Text an eine Matrix-Kennung.
///
/// **Die Raumsuche ist der heikle Teil, nicht das Senden.** In Matrix geht
/// eine Nachricht nicht an einen Menschen, sondern in einen Raum. Ein
/// Direktgespräch ist ein Raum mit genau zwei Mitgliedern – und den muss
/// man finden, bevor man einen zweiten daneben stellt.
///
/// **Warum hier ein Sync steht, obwohl niemand etwas empfangen will.**
/// `get_dm_room` schaut in die Räume, die der Client KENNT. Nach einem
/// Neustart des Dienstes kennt er noch keine – der Speicher hat sie zwar,
/// aber was dort fehlt, weil es seit dem letzten Lauf drüben entstanden ist,
/// holt erst ein Sync. Ohne diesen Zwischenschritt legte jedes Senden einen
/// NEUEN Raum an: Der Empfänger bekäme bei jeder Nachricht eine weitere
/// Einladung, und der Verlauf zerfiele in lauter Räume mit je einer Zeile.
///
/// Der Sync steht deshalb zwischen den beiden Versuchen und nicht davor:
/// Kennt der Client den Raum schon, kostet das Senden gar nichts.
pub async fn senden(client: &Client, ziel: &str, text: &str) -> Result<OwnedEventId> {
    let raum = direktraum(client, ziel).await?;

    let antwort = raum
        .send(RoomMessageEventContent::text_plain(text))
        .await
        .context("Message could not be sent")?;

    // `response.event_id` und nicht `antwort.event_id`: Der Rückgabewert
    // trägt daneben noch die Verschlüsselungsangaben des gesendeten
    // Ereignisses – die braucht hier niemand, aber sie sind der Grund für
    // die zusätzliche Ebene.
    Ok(antwort.response.event_id)
}

/// Schickt eine Datei an eine Matrix-Kennung (docs/plan-email-pgp.md,
/// Schritt 1), optional mit einem Text dazu.
///
/// **Die Verschlüsselung macht das SDK.** In einem verschlüsselten Raum lädt
/// `send_attachment` die Datei verschlüsselt hoch und legt den Schlüssel ins
/// Ereignis -- der Homeserver speichert nur Rauschen. Der Text reist als
/// Bildunterschrift (`caption`) im selben Ereignis, nicht als zweite
/// Nachricht.
///
/// **Die Grenze prüft der Aufrufer** ([`hochladegrenze`]), bevor er die
/// Bytes überhaupt einsammelt; hier würde der Homeserver erst nach dem
/// Hochladen ablehnen.
pub async fn anhang_senden(
    client: &Client,
    ziel: &str,
    name: &str,
    mime: &str,
    daten: Vec<u8>,
    text: Option<&str>,
) -> Result<OwnedEventId> {
    use matrix_sdk::attachment::AttachmentConfig;
    use matrix_sdk::ruma::events::room::message::TextMessageEventContent;

    let raum = direktraum(client, ziel).await?;
    let art: mime::Mime = mime.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM);
    let unterschrift = text
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(TextMessageEventContent::plain);

    let antwort = raum
        .send_attachment(
            name,
            &art,
            daten,
            AttachmentConfig::new().caption(unterschrift),
        )
        .await
        .context("Attachment could not be sent")?;

    Ok(antwort.event_id)
}

/// Wie groß eine Datei beim Homeserver sein darf, in Bytes (`m.upload.size`).
/// `None`, wenn er es nicht sagt -- dann gilt nur openanys eigene Grenze.
pub async fn hochladegrenze(client: &Client) -> Option<u64> {
    client
        .load_or_fetch_max_upload_size()
        .await
        .ok()
        .map(u64::from)
}

/// Die Bytes eines empfangenen Anhangs, entschlüsselt.
///
/// `quelle` ist die [`Anhang::quelle`] aus dem Eingang: bei verschlüsselten
/// Räumen samt Schlüssel. Wer sie aufbewahrt, bewahrt also auch den Schlüssel
/// auf -- in der App liegt sie im eigenen Speicher, im Sidecar in Laravels
/// Datenbank, die ohnehin mitlesen darf (Einwilligung).
pub async fn anhang_holen(client: &Client, quelle: &str) -> Result<Vec<u8>> {
    use matrix_sdk::media::{MediaFormat, MediaRequestParameters};
    use matrix_sdk::ruma::events::room::MediaSource;

    let source: MediaSource =
        serde_json::from_str(quelle).context("unusable attachment details")?;
    client
        .media()
        .get_media_content(
            &MediaRequestParameters {
                source,
                format: MediaFormat::File,
            },
            true,
        )
        .await
        .context("Attachment could not be fetched")
}

/// Der Direktraum mit `ziel` -- gefunden oder neu angelegt.
///
/// **Die Raumsuche ist der heikle Teil, nicht das Senden.** Siehe [`senden`].
async fn direktraum(client: &Client, ziel: &str) -> Result<Room> {
    let ziel: &UserId = ziel.try_into().context("unusable Matrix ID")?;

    Ok(match client.get_dm_room(ziel) {
        Some(raum) => raum,
        None => {
            client
                .sync_once(SyncSettings::default())
                .await
                .context("Sync with the homeserver failed")?;

            match client.get_dm_room(ziel) {
                Some(raum) => raum,
                // `create_dm` legt bei eingeschalteter Verschlüsselung einen
                // verschlüsselten Raum an – wir müssen das nicht eigens
                // verlangen und sollten es auch nicht: Was „empfohlen" ist,
                // ändert sich mit der Spezifikation, und das SDK weiss es
                // besser als diese Zeile.
                None => client
                    .create_dm(ziel)
                    .await
                    .context("Direct chat could not be created")?,
            }
        }
    })
}

/// Meldet das Gerät beim Homeserver ab.
///
/// **Warum das mehr ist als das Token wegzuwerfen.** Ein vergessenes Token
/// bleibt drüben gültig, und das Gerät steht weiter in der Liste – für den
/// Menschen sieht „getrennt" dann aus wie „läuft noch". Erst das Abmelden
/// macht den Widerruf für ihn sichtbar.
pub async fn abmelden(client: &Client) -> Result<()> {
    client
        .matrix_auth()
        .logout()
        .await
        .context("Sign-out at the homeserver failed")?;

    Ok(())
}

/// Eine hereingekommene Textnachricht.
#[derive(Debug, Clone)]
pub struct Eingang {
    pub event_id: String,
    /// Die Kennung dessen, der geschrieben hat.
    pub absender: String,
    pub text: String,
    /// Der Raum, in dem sie stand.
    pub raum: String,
    /// Mit wem dieser Raum ein Direktgespraech ist -- `None` bei einem Raum,
    /// der keines ist oder mehr als ein Gegenueber hat.
    ///
    /// **Wozu, wenn doch `absender` da ist.** Fuer das eigene Echo: Eine
    /// Nachricht, die dieses Konto auf einem ANDEREN Geraet schrieb, kommt mit
    /// dem eigenen Namen als Absender. Der Sidecar braucht das nicht (Laravel
    /// verwirft das Echo), die App schon -- sie haelt ihren Verlauf selbst
    /// und muss wissen, an wen die Zeile ging.
    pub gegenueber: Option<String>,
    /// Wann der Homeserver sie annahm, in Millisekunden seit 1970. Die Uhr
    /// des Geraets waere beim Nachholen eines Tages Rueckstand falsch.
    pub zeit_ms: u64,
    /// Eine Datei, ein Bild, ein Video oder eine Tonaufnahme -- `None` bei
    /// reinem Text. `text` ist dann die Bildunterschrift oder leer.
    pub anhang: Option<Anhang>,
}

/// Ein Anhang, wie er in einem Ereignis steht -- ohne seine Bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anhang {
    pub name: String,
    pub mime: String,
    /// In Bytes; 0, wenn der Absender es nicht angab.
    pub groesse: u64,
    /// Wo die Datei liegt, als JSON (`MediaSource`): die mxc-Adresse, bei
    /// verschlüsselten Räumen samt Schlüssel. Für [`anhang_holen`].
    pub quelle: String,
}

impl Eingang {
    /// Hat die Nachricht etwas, das openany zeigen kann?
    pub fn hat_inhalt(&self) -> bool {
        !self.text.is_empty() || self.anhang.is_some()
    }
}

/// Text und Anhang aus der Art der Nachricht.
///
/// **Beim Anhang ist `body` der Dateiname -- es sei denn, `filename` steht
/// daneben.** Dann ist `body` die Bildunterschrift (Matrix 1.10). Beides zu
/// zeigen hiesse, den Dateinamen zweimal zu schreiben.
fn text_und_anhang(art: &MessageType) -> (String, Option<Anhang>) {
    use matrix_sdk::ruma::events::room::MediaSource;

    fn anhang(
        body: &str,
        filename: Option<&str>,
        mime: Option<&str>,
        groesse: Option<u64>,
        quelle: &MediaSource,
    ) -> (String, Option<Anhang>) {
        let (name, text) = match filename {
            Some(f) if !f.is_empty() && f != body => (f.to_string(), body.to_string()),
            _ => (body.to_string(), String::new()),
        };
        let Ok(quelle) = serde_json::to_string(quelle) else {
            return (text, None);
        };
        (
            text,
            Some(Anhang {
                name,
                mime: mime.unwrap_or("application/octet-stream").to_string(),
                groesse: groesse.unwrap_or(0),
                quelle,
            }),
        )
    }

    match art {
        MessageType::Text(t) => (t.body.clone(), None),
        MessageType::File(f) => {
            let info = f.info.as_deref();
            anhang(
                &f.body,
                f.filename.as_deref(),
                info.and_then(|i| i.mimetype.as_deref()),
                info.and_then(|i| i.size).map(u64::from),
                &f.source,
            )
        }
        MessageType::Image(f) => {
            let info = f.info.as_deref();
            anhang(
                &f.body,
                f.filename.as_deref(),
                info.and_then(|i| i.mimetype.as_deref()),
                info.and_then(|i| i.size).map(u64::from),
                &f.source,
            )
        }
        MessageType::Video(f) => {
            let info = f.info.as_deref();
            anhang(
                &f.body,
                f.filename.as_deref(),
                info.and_then(|i| i.mimetype.as_deref()),
                info.and_then(|i| i.size).map(u64::from),
                &f.source,
            )
        }
        MessageType::Audio(f) => {
            let info = f.info.as_deref();
            anhang(
                &f.body,
                f.filename.as_deref(),
                info.and_then(|i| i.mimetype.as_deref()),
                info.and_then(|i| i.size).map(u64::from),
                &f.source,
            )
        }
        // Alles andere (Standort, Umfrage, Emote …) hat kein Gegenstück.
        _ => (String::new(), None),
    }
}

/// Gleicht dauerhaft ab und meldet jede hereinkommende Textnachricht.
///
/// **Kehrt erst zurück, wenn etwas bricht.** Der Aufrufer entscheidet, ob er
/// es erneut versucht – hier wäre die Entscheidung falsch aufgehoben, weil
/// sie vom Aufrufer abhängt: Ein Dienst will neu verbinden, ein
/// Testprogramm will den Fehler sehen.
///
/// **Text und Anhänge, sonst nichts.** Seit dem 28.09.2026 kommen Dateien,
/// Bilder, Videos und Tonaufnahmen mit ([`Eingang::anhang`]). Beitritte,
/// Reaktionen, Zustandsänderungen haben in openany kein Gegenstück; sie
/// kommen mit leerem Text und ohne Anhang an ([`Eingang::hat_inhalt`]), und
/// der Aufrufer verwirft sie.
///
/// **Warum das eigene Echo nicht hier gefiltert wird:** Der Abgleich liefert
/// auch die eigenen Nachrichten zurück. Sie hier wegzulassen hiesse, sich
/// auf die eigene Kennung zu verlassen – und die steht in der Datenbank,
/// nicht hier. Laravel erkennt das Echo und sagt es; diese Funktion meldet,
/// was kam.
/// **`bei_runde` läuft nach jedem geglückten Durchgang**, auch wenn nichts
/// kam. Nur daran ist zu erkennen, dass die Leitung wieder steht: Ein
/// Abgleich, der einmal abgerissen ist, kehrt nie mit `Ok` zurück – er läuft
/// nach dem nächsten Versuch einfach weiter. Ohne dieses Zeichen bliebe eine
/// Störungsmeldung stehen, bis zufällig jemand schreibt.
pub async fn abgleichen<F, Fut, R, RFut>(
    client: &Client,
    bei_nachricht: F,
    bei_runde: R,
) -> Result<()>
where
    F: Fn(Eingang) -> Fut + Send + Sync + 'static,
    // `'static` AUCH AM FUTURE: Das SDK hält den Handler über die gesamte
    // Laufzeit des Abgleichs und ruft ihn irgendwann später auf -- es kann
    // nicht zusagen, dass etwas Geliehenes bis dahin noch existiert.
    Fut: std::future::Future<Output = ()> + Send + 'static,
    R: Fn() -> RFut + Send + Sync,
    RFut: std::future::Future<Output = ()> + Send,
{
    let griff = nachrichten_melden(client, bei_nachricht);

    let ergebnis = client
        .sync_with_callback(SyncSettings::default(), |_| {
            let bei_runde = &bei_runde;
            async move {
                bei_runde().await;
                matrix_sdk::LoopCtrl::Continue
            }
        })
        .await
        .context("Sync with the homeserver aborted");

    client.remove_event_handler(griff);

    ergebnis
}

/// Haengt den Handler an, der jede Textnachricht als [`Eingang`] meldet.
/// Geteilt von [`abgleichen`] (dauerhaft) und [`einmal_abgleichen`].
fn nachrichten_melden<F, Fut>(
    client: &Client,
    bei_nachricht: F,
) -> matrix_sdk::event_handler::EventHandlerHandle
where
    F: Fn(Eingang) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    // ARC, WEIL DAS SDK DEN HANDLER KLONT. Es ruft ihn für jedes passende
    // Ereignis auf und braucht dafür eine eigene Kopie -- ohne diese Hülle
    // verlangt es `Clone` von der übergebenen Funktion, und das kann ein
    // Aufrufer mit gefangenem Zustand nicht immer zusagen.
    let bei_nachricht = std::sync::Arc::new(bei_nachricht);

    // DER GRIFF WIRD AUFBEWAHRT UND AM ENDE GELOEST. Ein Aufrufer, der nach
    // einem Abriss erneut ruft, haengte sonst einen zweiten Handler an
    // denselben Client -- nach zehn Versuchen kaeme jede Nachricht zehnmal
    // beim Aufrufer an.
    client.add_event_handler(move |ereignis: OriginalSyncRoomMessageEvent, raum: Room| {
        // Genau EIN Gegenueber, sonst keines: Bei einem Raum mit dreien
        // waere jede Wahl geraten.
        let ziele = raum.direct_targets();
        let gegenueber = if ziele.len() == 1 {
            ziele.into_iter().next().map(|z| z.to_string())
        } else {
            None
        };

        let (text, anhang) = text_und_anhang(&ereignis.content.msgtype);
        let melden = bei_nachricht(Eingang {
            event_id: ereignis.event_id.to_string(),
            absender: ereignis.sender.to_string(),
            text,
            raum: raum.room_id().to_string(),
            gegenueber,
            zeit_ms: u64::from(ereignis.origin_server_ts.0),
            anhang,
        });

        async move {
            melden.await;
        }
    })
}

/// EIN Durchgang: holt, was seit dem letzten Abgleich kam, meldet die
/// Textnachrichten und kehrt zurueck. Fuer den Wachdienst der App, der von
/// einem Push-Signal geweckt wird und nicht dauerhaft abgleichen soll --
/// das hielte den Funk wach, den das Signal gerade schonen soll.
///
/// Der Handler wird wie bei [`abgleichen`] am Ende wieder geloest.
pub async fn einmal_abgleichen<F, Fut>(client: &Client, bei_nachricht: F) -> Result<()>
where
    F: Fn(Eingang) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let griff = nachrichten_melden(client, bei_nachricht);
    let ergebnis = client
        .sync_once(SyncSettings::default())
        .await
        .map(|_| ())
        .context("Sync with the homeserver aborted");
    client.remove_event_handler(griff);
    ergebnis
}

/// Traegt ein Push-Ziel beim Homeserver ein: Kommt fuer dieses Konto
/// etwas an, schickt der Homeserver eine Meldung an `gateway` (openanys
/// ntfy, `/_matrix/push/v1/notify`), und ntfy reicht sie an das Thema in
/// `pushkey` weiter.
///
/// **`event_id_only`**: Der Homeserver schickt nur Raum und Ereignis-ID,
/// keinen Absender und keinen Text. Den Inhalt holt das Geraet selbst und
/// entschluesselt ihn dort -- ntfy sieht nichts davon.
///
/// Wiederholbar: Dasselbe `(app_id, pushkey)` ersetzt den alten Eintrag.
pub async fn weckruf_eintragen(
    client: &Client,
    app_id: &str,
    pushkey: &str,
    gateway: &str,
    geraetename: &str,
) -> Result<()> {
    use matrix_sdk::ruma::api::client::push::{PusherIds, PusherInit, PusherKind};
    use matrix_sdk::ruma::push::{HttpPusherData, PushFormat};

    let mut daten = HttpPusherData::new(gateway.to_owned());
    daten.format = Some(PushFormat::EventIdOnly);
    let ziel = PusherInit {
        ids: PusherIds::new(pushkey.to_owned(), app_id.to_owned()),
        kind: PusherKind::Http(daten),
        app_display_name: "openany".to_owned(),
        device_display_name: geraetename.to_owned(),
        profile_tag: None,
        lang: "de".to_owned(),
    };
    client
        .pusher()
        .set(ziel.into(), false)
        .await
        .context("Push target could not be registered at the homeserver")
}

/// Das Gegenstueck zu [`weckruf_eintragen`].
pub async fn weckruf_austragen(client: &Client, app_id: &str, pushkey: &str) -> Result<()> {
    use matrix_sdk::ruma::api::client::push::PusherIds;

    client
        .pusher()
        .delete(PusherIds::new(pushkey.to_owned(), app_id.to_owned()))
        .await
        .context("Push target could not be removed at the homeserver")
}

/// Baut einen Client mit eigenem, verschlüsseltem Speicher.
///
/// DAS VERZEICHNIS WIRD HIER SELBST ANGELEGT, obwohl das SDK das auch täte:
/// Nur so lässt sich ein Rechteproblem von einem Netzproblem unterscheiden.
/// Legt es das SDK an und darf nicht, kommt der Fehler aus demselben `build()`
/// wie ein nicht erreichbarer Homeserver – und in der Webapp stünde
/// "Verbindung zum Homeserver", wo ein Verzeichnis nicht schreibbar war.
async fn klient(homeserver: &str, speicher: &Path, passphrase: &str) -> Result<Client> {
    std::fs::create_dir_all(speicher)
        .with_context(|| format!("Storage {} could not be created", speicher.display()))?;

    Client::builder()
        .homeserver_url(homeserver)
        .sqlite_store(speicher, Some(passphrase))
        .build()
        .await
        .context("Connection to the homeserver could not be established")
}

/// Liest aus einem angemeldeten Client, was aufbewahrt werden muss.
fn sitzung_lesen(client: &Client) -> Result<Sitzung> {
    let sitzung = client
        .matrix_auth()
        .session()
        // Nach einem erfolgreichen Login ist die Sitzung da. Steht sie
        // trotzdem nicht, hat das SDK seine Zusage gebrochen – dann ist ein
        // klarer Abbruch besser als ein leeres Token in der Datenbank.
        .context("signed in, but without a session – the SDK behaved differently than promised")?;

    Ok(Sitzung {
        mxid: sitzung.meta.user_id,
        device_id: sitzung.meta.device_id,
        access_token: sitzung.tokens.access_token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(json: serde_json::Value) -> MessageType {
        serde_json::from_value::<RoomMessageEventContent>(json)
            .unwrap()
            .msgtype
    }

    #[test]
    fn eine_datei_ohne_unterschrift_heisst_wie_ihr_body() {
        let (text, anhang) = text_und_anhang(&art(serde_json::json!({
            "msgtype": "m.file",
            "body": "Mietvertrag.pdf",
            "url": "mxc://beispiel.test/abc",
            "info": { "mimetype": "application/pdf", "size": 12345 },
        })));
        assert_eq!(text, "");
        let a = anhang.unwrap();
        assert_eq!(a.name, "Mietvertrag.pdf");
        assert_eq!(a.mime, "application/pdf");
        assert_eq!(a.groesse, 12345);
        assert!(a.quelle.contains("mxc://beispiel.test/abc"));
    }

    #[test]
    fn mit_filename_ist_body_die_unterschrift() {
        let (text, anhang) = text_und_anhang(&art(serde_json::json!({
            "msgtype": "m.image",
            "body": "Schau mal!",
            "filename": "urlaub.jpg",
            "url": "mxc://beispiel.test/bild",
        })));
        assert_eq!(text, "Schau mal!");
        let a = anhang.unwrap();
        assert_eq!(a.name, "urlaub.jpg");
        // Ohne Angabe: der Allzweck-Typ, nicht geraten.
        assert_eq!(a.mime, "application/octet-stream");
        assert_eq!(a.groesse, 0);
    }

    #[test]
    fn eine_verschluesselte_quelle_traegt_ihren_schluessel() {
        let (_, anhang) = text_und_anhang(&art(serde_json::json!({
            "msgtype": "m.file",
            "body": "geheim.txt",
            "file": {
                "url": "mxc://beispiel.test/verschluesselt",
                "key": { "kty": "oct", "key_ops": ["encrypt", "decrypt"], "alg": "A256CTR",
                         "k": "qcHVMSgYg-71CauWBezXI5qkaRb0LuIy-Wx5kIaHMIA", "ext": true },
                "iv": "X85+XgHN+HEAAAAAAAAAAA",
                "hashes": { "sha256": "5qG4fFnbbVdlAB1Q72JDKwCagV6Dbkx9uds4rSak37c" },
                "v": "v2"
            },
        })));
        let quelle = anhang.unwrap().quelle;
        assert!(quelle.contains("verschluesselt"));
        assert!(quelle.contains("\"k\""));
    }

    #[test]
    fn text_bleibt_text() {
        let (text, anhang) = text_und_anhang(&art(serde_json::json!({
            "msgtype": "m.text", "body": "Hallo",
        })));
        assert_eq!(text, "Hallo");
        assert!(anhang.is_none());
    }
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    /// Ein Homeserver, der gerade so viel kann, wie eine Anmeldung braucht.
    ///
    /// Bewusst kein echter: Ein Test, der matrix.org ruft, ist beim ersten
    /// Ausfall dort rot, ohne dass sich hier etwas geändert hätte – und er
    /// legt bei jedem Lauf ein Gerät auf einem fremden Server an.
    async fn homeserver_der_zusagt() -> MockServer {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "versions": ["v1.1", "v1.5"],
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user_id": "@tiffy:beispiel.test",
                "access_token": "geheim-fuer-dieses-geraet",
                "device_id": "OPENANY1",
            })))
            .mount(&server)
            .await;

        server
    }

    #[tokio::test]
    async fn eine_anmeldung_gibt_zurueck_was_aufbewahrt_werden_muss() {
        let server = homeserver_der_zusagt().await;
        let ordner = tempfile::tempdir().unwrap();

        let sitzung = anmelden(
            Anmeldung {
                homeserver: &server.uri(),
                benutzer: "tiffy",
                passwort: "hunter2",
            },
            ordner.path(),
            "passphrase",
        )
        .await
        .expect("Anmeldung sollte durchgehen");

        // Die Kennung kommt vom Server und nicht aus dem Formular: Getippt
        // wurde `tiffy`, aufbewahrt wird die vollständige Kennung.
        assert_eq!(sitzung.mxid.as_str(), "@tiffy:beispiel.test");
        assert_eq!(sitzung.device_id.as_str(), "OPENANY1");
        assert_eq!(sitzung.access_token, "geheim-fuer-dieses-geraet");
    }

    /// Der Speicher muss die Anmeldung überdauern – sonst wären die
    /// Verschlüsselungsschlüssel des Geräts nach dem Rückgabewert verloren.
    #[tokio::test]
    async fn die_anmeldung_hinterlaesst_einen_speicher() {
        let server = homeserver_der_zusagt().await;
        let ordner = tempfile::tempdir().unwrap();

        anmelden(
            Anmeldung {
                homeserver: &server.uri(),
                benutzer: "tiffy",
                passwort: "hunter2",
            },
            ordner.path(),
            "passphrase",
        )
        .await
        .unwrap();

        assert!(
            ordner.path().read_dir().unwrap().next().is_some(),
            "der Speicher des Geräts sollte angelegt sein",
        );
    }

    /// DER TEST ZUR TEUERSTEN FALLE DIESER DATEI.
    ///
    /// `get_dm_room` schaut nur in die Räume, die der Client KENNT. Nach
    /// einem Neustart kennt er keine – ohne den Sync dazwischen legte jedes
    /// Senden einen neuen Raum an, und der Empfänger bekäme bei jeder
    /// Nachricht eine weitere Einladung.
    ///
    /// Geprüft wird deshalb die REIHENFOLGE: erst abgleichen, dann anlegen.
    /// Ob das Senden danach durchgeht, ist hier nicht die Frage – dafür
    /// müsste der Fake-Homeserver auch noch Schlüssel aushandeln.
    #[tokio::test]
    async fn ohne_bekannten_raum_wird_erst_abgeglichen_und_dann_angelegt() {
        let server = homeserver_der_zusagt().await;
        let ordner = tempfile::tempdir().unwrap();

        let sitzung_client = Client::builder()
            .homeserver_url(server.uri())
            .sqlite_store(ordner.path(), Some("passphrase"))
            .build()
            .await
            .unwrap();
        sitzung_client
            .matrix_auth()
            .login_username("tiffy", "hunter2")
            .await
            .unwrap();

        // Ein leerer Abgleich: Der Homeserver kennt keinen gemeinsamen Raum.
        Mock::given(method("GET"))
            .and(path("/_matrix/client/v3/sync"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "next_batch": "s1",
            })))
            .expect(1..)
            .mount(&server)
            .await;

        // DIE EIGENTLICHE ZUSAGE: Genau EIN neuer Raum, nicht mehr.
        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/createRoom"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "room_id": "!neu:beispiel.test",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let _ = senden(&sitzung_client, "@ferdinand:beispiel.test", "Hallo").await;

        // `verify` prüft die `expect`-Zusagen oben. Ohne den Sync in
        // `senden` bliebe die Sync-Erwartung unerfüllt und dieser Test rot.
        server.verify().await;
    }

    async fn angemeldet(server: &MockServer, ordner: &Path) -> Client {
        let client = Client::builder()
            .homeserver_url(server.uri())
            .sqlite_store(ordner, Some("passphrase"))
            .build()
            .await
            .unwrap();
        client
            .matrix_auth()
            .login_username("tiffy", "hunter2")
            .await
            .unwrap();
        client
    }

    /// Das Push-Ziel geht als `event_id_only` hinaus -- sonst schickte der
    /// Homeserver Absender und Text an ntfy, und genau das soll nicht sein.
    #[tokio::test]
    async fn das_push_ziel_verlangt_nur_die_ereignis_id() {
        use wiremock::matchers::body_partial_json;

        let server = homeserver_der_zusagt().await;
        let ordner = tempfile::tempdir().unwrap();
        let client = angemeldet(&server, ordner.path()).await;

        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/pushers/set"))
            .and(body_partial_json(serde_json::json!({
                "kind": "http",
                "app_id": "de.openany.app",
                "pushkey": "https://ntfy.openany.de/upGeheim?up=1",
                "data": {
                    "url": "https://ntfy.openany.de/_matrix/push/v1/notify",
                    "format": "event_id_only",
                },
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&server)
            .await;

        weckruf_eintragen(
            &client,
            "de.openany.app",
            "https://ntfy.openany.de/upGeheim?up=1",
            "https://ntfy.openany.de/_matrix/push/v1/notify",
            "Tablet",
        )
        .await
        .expect("Eintragen sollte durchgehen");
        server.verify().await;
    }

    /// Ein Durchgang meldet, was kam, und kehrt zurueck -- ohne haengen zu
    /// bleiben wie `abgleichen`.
    #[tokio::test]
    async fn einmal_abgleichen_meldet_und_kehrt_zurueck() {
        let server = homeserver_der_zusagt().await;
        let ordner = tempfile::tempdir().unwrap();
        let client = angemeldet(&server, ordner.path()).await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/v3/sync"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "next_batch": "s2",
                "rooms": { "join": { "!raum:beispiel.test": { "timeline": { "events": [{
                    "type": "m.room.message",
                    "event_id": "$e1",
                    "sender": "@ferdinand:beispiel.test",
                    "origin_server_ts": 1790000000000u64,
                    "content": { "msgtype": "m.text", "body": "Bist du da?" }
                }]}}}}
            })))
            .mount(&server)
            .await;

        let gesehen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let g = gesehen.clone();
        einmal_abgleichen(&client, move |e| {
            g.lock().unwrap().push((e.absender, e.text, e.raum));
            async {}
        })
        .await
        .expect("ein Durchgang sollte durchgehen");

        assert_eq!(
            *gesehen.lock().unwrap(),
            vec![(
                "@ferdinand:beispiel.test".to_string(),
                "Bist du da?".to_string(),
                "!raum:beispiel.test".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn ein_homeserver_der_ablehnt_wird_gemeldet_und_nicht_verschluckt() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "versions": ["v1.1"],
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/login"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "errcode": "M_FORBIDDEN",
                "error": "Invalid password",
            })))
            .mount(&server)
            .await;

        let ordner = tempfile::tempdir().unwrap();

        let ergebnis = anmelden(
            Anmeldung {
                homeserver: &server.uri(),
                benutzer: "tiffy",
                passwort: "falsch",
            },
            ordner.path(),
            "passphrase",
        )
        .await;

        assert!(
            ergebnis.is_err(),
            "ein falsches Passwort darf nicht durchgehen"
        );
    }
}
