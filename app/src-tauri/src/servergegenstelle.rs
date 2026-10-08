//! openany.de als Gegenstelle -- **mit** Dateien und Bildern.
//!
//! Der [`OpenanyClient`] ist selbst schon eine
//! [`Gegenstelle`](openany_sync::Gegenstelle), aber eine ohne Inhalte:
//! `traegt_dateien()` ist dort `false`, und das war bis zum 16.09.2026
//! richtig. Was fehlte, ist nicht HTTP, sondern eine Uebersetzung.
//!
//! **Zwei Weltbilder.** Zwei Geraete benennen Inhalte nach ihrem Abdruck: Sie
//! legen sie unter `sha256` ab, derselbe Inhalt liegt genau einmal da, und
//! sie koennen einander fragen „welche dieser Abdruecke hast du?". Der Server
//! kennt diese Sicht ueberhaupt nicht -- bei ihm haengen Bytes an einer Datei
//! oder einem Bild, und `GET /api/sync/content` fragt nach `type` und `key`.
//! Einen Abdruck fuehrt er nirgends.
//!
//! Also uebersetzt diese Seite, und zwar mit dem, was nur sie weiss:
//!
//! * **Abdruck → Sache** kommt aus dem eigenen Speicher (`sache_zu_abdruck`).
//!   Die Datei, die diesen Inhalt traegt, hat drueben dieselbe uuid.
//! * **„hast du das?"** beantwortet die eigene Buchfuehrung
//!   (`inhalt_dort`): Was von dort kam oder erfolgreich dorthin ging.
//!
//! **Beides wird EINMAL vor der Uebertragung gebaut und dann nicht mehr
//! angefasst.** Waehrend Inhalte fliessen, darf der Speicher nicht gesperrt
//! sein -- ein Video kann Minuten dauern, und die Oberflaeche soll bedienbar
//! bleiben. Deshalb traegt diese Struktur zwei fertige Tabellen mit sich und
//! keine Verbindung zum Speicher.

use async_trait::async_trait;
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyClient, OpenanyError};
use openany_sync::Gegenstelle;
use std::collections::{BTreeMap, BTreeSet};

pub struct ServerGegenstelle {
    client: OpenanyClient,
    /// Abdruck → (`file_node`|`media`, uuid).
    wegweiser: BTreeMap<String, (String, String)>,
    /// Welche Abdruecke drueben liegen -- nach eigener Buchfuehrung.
    dort: BTreeSet<String>,
}

impl ServerGegenstelle {
    /// Fuer den Lauf selbst: Der Laeufer holt keine Dateiinhalte, er gleicht
    /// ab, WAS es gibt. Die beiden Tabellen bleiben leer.
    pub fn neu(client: OpenanyClient) -> Self {
        Self {
            client,
            wegweiser: BTreeMap::new(),
            dort: BTreeSet::new(),
        }
    }

    /// Fuer die Inhalte danach -- mit dem Wissen, das der Lauf erst geschaffen
    /// hat.
    pub fn mit_wegweiser(
        mut self,
        wegweiser: BTreeMap<String, (String, String)>,
        dort: BTreeSet<String>,
    ) -> Self {
        self.wegweiser = wegweiser;
        self.dort = dort;
        self
    }

    pub fn client(&self) -> &OpenanyClient {
        &self.client
    }

    /// Die Adresse, an der Marken, Urspruenge und die Buchfuehrung haengen.
    ///
    /// Eigener Name neben `Gegenstelle::basis`, damit ein Aufrufer sie nennen
    /// kann, ohne das Trait hereinzuholen -- und ohne dass zwei gleichnamige
    /// Methoden die Wahl zur Ratesache machen.
    pub fn basis_adresse(&self) -> &str {
        OpenanyClient::basis(&self.client)
    }

    /// Wie die Sache heisst, die diesen Inhalt traegt.
    pub fn sache(&self, abdruck: &str) -> Option<&(String, String)> {
        self.wegweiser.get(abdruck)
    }

    /// Liegt dieser Inhalt drueben? -- nach eigener Buchfuehrung.
    ///
    /// **Nur fuer die Richtung hinauf.** Sie haelt fest, was von dort kam oder
    /// erfolgreich dorthin ging, und beantwortet damit „was muss ich noch
    /// schicken?". Fuer die Gegenrichtung taugt sie nicht -- siehe
    /// [`Gegenstelle::inhalte_da`].
    pub fn liegt_dort(&self, abdruck: &str) -> bool {
        self.dort.contains(abdruck)
    }
}

#[async_trait]
impl Gegenstelle for ServerGegenstelle {
    fn basis(&self) -> &str {
        OpenanyClient::basis(&self.client)
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        self.client.delta(seit).await
    }

    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        self.client.notiztext(zk_id).await
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        self.client.anwenden(eintraege).await
    }

    async fn kontaktfoto(&self, uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        self.client.inhalt("contact_photo", uuid).await
    }

    /// **Seit dem 16.09.2026 ja.** Davor stand hier `false` mit der
    /// Begruendung, die Bytes muessten ueber `UploadSession` kommen -- das
    /// war die Beschreibung der Luecke, die diese Datei schliesst.
    fn traegt_dateien(&self) -> bool {
        true
    }

    async fn inhalt(&self, abdruck: &str, von: u64, laenge: u64) -> Result<Vec<u8>, OpenanyError> {
        let Some((art, key)) = self.sache(abdruck) else {
            // Kein Wegweiser heisst: Diese Seite weiss nicht, welche Datei
            // den Inhalt traegt -- also kann sie ihn drueben nicht benennen.
            return Err(OpenanyError::Unlesbar(format!(
                "No file is known here for {abdruck}."
            )));
        };

        self.client.inhalt_bereich(art, key, von, laenge).await
    }

    /// **Was ich benennen kann, frage ich auch.**
    ///
    /// Der Server kann diese Frage nicht beantworten -- er fuehrt keine
    /// Abdruecke. Die Buchfuehrung ([`ServerGegenstelle::liegt_dort`]) taugt
    /// hier auch nicht: Sie haelt fest, was HINAUF ging, und beim ersten Lauf
    /// ist sie leer. Wer sie hier befragte, holte nie etwas herunter.
    ///
    /// Die Antwort ist trotzdem fast immer richtig, und zwar aus einem
    /// Grund, der nichts mit dem Server zu tun hat: **Ein Inhalt, der hier
    /// fehlt, ist hier nie entstanden.** Was diese Seite selbst anlegt, legt
    /// sie mitsamt Bytes ab. Fehlt etwas, kam die Datei von drueben -- und
    /// dort liegen ihre Bytes.
    ///
    /// Bleibt der seltene Fall, dass auch drueben keine Bytes liegen (eine
    /// Datei, die ein drittes Geraet angelegt und noch nirgends hochgeladen
    /// hat). Dann antwortet der Server mit 404, und der Versuch steht als
    /// Fehler im Bericht statt als „nicht da". Das ist eine Zeile zu viel im
    /// Protokoll, kein falscher Inhalt.
    async fn inhalte_da(&self, abdruecke: &[String]) -> Result<Vec<String>, OpenanyError> {
        Ok(abdruecke
            .iter()
            .filter(|a| self.wegweiser.contains_key(*a))
            .cloned()
            .collect())
    }

    /// Wie beim blanken Client: Der Server rechnet ein Kontaktfoto neu, und
    /// dasselbe Bild kaeme sonst bei jedem Lauf als Aenderung zurueck.
    fn fotos_mitschicken(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ein Client, der nie etwas schickt -- hier wird nur uebersetzt.
    fn gegenstelle() -> ServerGegenstelle {
        ServerGegenstelle::neu(OpenanyClient::neu("https://beispiel.invalid", "x").unwrap())
    }

    fn mit(paare: &[(&str, &str, &str)], dort: &[&str]) -> ServerGegenstelle {
        gegenstelle().mit_wegweiser(
            paare
                .iter()
                .map(|(a, art, uuid)| (a.to_string(), (art.to_string(), uuid.to_string())))
                .collect(),
            dort.iter().map(|d| d.to_string()).collect(),
        )
    }

    #[test]
    fn ein_abdruck_wird_zur_sache() {
        let g = mit(&[("aa", "file_node", "u-1"), ("bb", "media", "u-2")], &[]);

        assert_eq!(
            g.sache("aa"),
            Some(&("file_node".to_string(), "u-1".to_string()))
        );
        assert_eq!(
            g.sache("bb"),
            Some(&("media".to_string(), "u-2".to_string()))
        );
        assert_eq!(g.sache("cc"), None);
    }

    /// Ohne Wegweiser ist der Inhalt drueben nicht zu benennen -- und das
    /// muss ein Fehler sein, keine leere Antwort: Eine leere Antwort naehme
    /// [`openany_sync::inhalt_holen`] als „Datei zu Ende".
    #[tokio::test]
    async fn ohne_wegweiser_kein_inhalt() {
        let fehler = gegenstelle().inhalt("aa", 0, 10).await.unwrap_err();

        assert!(matches!(fehler, OpenanyError::Unlesbar(_)));
    }

    /// Gefragt wird nach allem, was sich benennen laesst -- nicht nach dem,
    /// was die Buchfuehrung kennt. Sonst holte der erste Lauf nie etwas.
    #[tokio::test]
    async fn gefragt_wird_was_benennbar_ist() {
        let g = mit(&[("aa", "file_node", "u-1")], &[]);

        let da = g
            .inhalte_da(&["aa".into(), "unbekannt".into()])
            .await
            .unwrap();

        assert_eq!(da, vec!["aa".to_string()]);
    }

    /// Die Buchfuehrung beantwortet die andere Frage: was muss noch hinauf?
    #[test]
    fn die_buchfuehrung_gilt_fuer_die_gegenrichtung() {
        let g = mit(
            &[("aa", "file_node", "u-1"), ("bb", "media", "u-2")],
            &["aa"],
        );

        assert!(g.liegt_dort("aa"));
        assert!(!g.liegt_dort("bb"));
    }

    #[test]
    fn dieser_server_traegt_dateien() {
        assert!(gegenstelle().traegt_dateien());
        // Und Kontaktfotos schickt er weiterhin nicht bei jedem Lauf mit:
        // Er rechnet sie neu, sie kaemen sonst als Aenderung zurueck.
        assert!(!gegenstelle().fotos_mitschicken());
    }
}
