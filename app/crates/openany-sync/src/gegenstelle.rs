//! Was der Laeufer von der anderen Seite braucht -- und sonst nichts.
//!
//! **Ein Trait und nicht der Client selbst.** Der Laeufer ist der Teil, in
//! dem die Reihenfolge steckt: erst ziehen, dann schieben; Marke seitenweise;
//! Ursprung nur, wenn wirklich angewendet wurde. Genau das will man pruefen
//! koennen, ohne einen Server -- und zwar auch die Faelle, die ein echter
//! Server nur schwer herstellt: eine abreissende Leitung mitten in der
//! zweiten Seite, ein widerrufener Schluessel, ein Eintrag mit einer Art aus
//! der Zukunft.
//!
//! Drei Methoden. Was der [`openany_client::OpenanyClient`] sonst noch kann
//! -- Bytes holen, koppeln -- braucht der Laeufer nicht, und was er nicht
//! braucht, steht auch nicht in seiner Erwartung.

use async_trait::async_trait;
use openany_client::{Delta, Eintrag, Ergebnis, OpenanyClient, OpenanyError};

#[async_trait]
pub trait Gegenstelle: Send + Sync {
    /// Der Name dieser Gegenstelle -- die Basis-Adresse. Marken und
    /// Urspruenge haengen daran.
    fn basis(&self) -> &str;

    /// Wie diese Gegenstelle im Titel einer Konfliktkopie heisst.
    ///
    /// Vorgabe ist die Adresse ohne Schema (`openany.de`). Ein Geraet in der
    /// Naehe heisst nach dem Namen, den es sich gegeben hat -- sein
    /// Fingerabdruck saegte sonst jeden Titel entzwei.
    fn name(&self) -> String {
        self.basis()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .to_string()
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError>;

    /// Der Text einer Notiz. Er reist nicht im Eintrag mit -- anders als die
    /// neun Felder eines Termins, die kuerzer sind als die Anfrage, die sie
    /// holen wuerde.
    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError>;

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError>;

    /// Die Bytes eines Kontaktfotos. Sie reisen nicht im Eintrag mit -- der
    /// traegt nur den Abdruck, und geholt wird nur, wenn der sich aenderte.
    ///
    /// Mit Vorgabe, weil nicht jede Gegenstelle Fotos kann: Ohne sie bleibt
    /// der Kontakt vollstaendig, nur ohne Bild.
    async fn kontaktfoto(&self, _uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        Err(OpenanyError::Unlesbar(
            "This counterpart does not deliver photos.".into(),
        ))
    }

    /// Gleicht diese Gegenstelle Dateien und Ordner ab?
    ///
    /// **Ein Server (noch) nicht:** Dort gehoeren zu einer Datei Bytes, die
    /// ueber `UploadSession` kommen muessen -- das ist Phase 3, Stufe D. Bis
    /// dahin gehen `file_node`-Eintraege weder hin noch her. **Ein Geraet ja.**
    fn traegt_dateien(&self) -> bool {
        false
    }

    /// Ein Stueck des Inhalts mit diesem Abdruck.
    async fn inhalt(
        &self,
        _abdruck: &str,
        _von: u64,
        _laenge: u64,
    ) -> Result<Vec<u8>, OpenanyError> {
        Err(OpenanyError::Unlesbar(
            "This counterpart does not deliver content.".into(),
        ))
    }

    /// Welche dieser Abdruecke hat die Gegenstelle als Inhalt vorliegen?
    async fn inhalte_da(&self, _abdruecke: &[String]) -> Result<Vec<String>, OpenanyError> {
        Ok(Vec::new())
    }

    /// Kontaktfotos bei jedem Schieben als Bytes mitschicken?
    ///
    /// **Ein Server nein:** Er rechnet ein Bild neu, und dasselbe Foto
    /// kaeme bei jedem Lauf als Aenderung zurueck. Er bekommt es nur, wenn es
    /// hier gesetzt wurde. **Ein Geraet ja:** Es hat sonst keinen Weg, ein
    /// Foto zu holen, das ueber ein drittes Geraet kam -- die Auskunft kann
    /// nicht zurueckfragen. Es vergleicht den Abdruck und schreibt nur, was
    /// neu ist.
    fn fotos_mitschicken(&self) -> bool {
        false
    }

    /// Nach einem ganzen Lauf: der Gegenstelle sagen, wie weit beide sind.
    ///
    /// * `eigene` -- bis hierhin ist das eigene Protokoll drueben angekommen.
    /// * `fremde` -- bis hierhin ist das Protokoll von drueben hier angekommen.
    ///
    /// **Ein Server braucht das nicht** (siehe Crate-Doku: er merkt sich
    /// nichts). Ein Geraet schon: Es gleicht selbst auch ab, und ohne diese
    /// Meldung holte sein naechster Lauf alles noch einmal, was eben erst
    /// hinueberging -- und ein aelterer Kontakt ueberschriebe dabei einen,
    /// der dort seither geaendert wurde.
    async fn marken_melden(&self, _eigene: i64, _fremde: i64) -> Result<(), OpenanyError> {
        Ok(())
    }
}

#[async_trait]
impl Gegenstelle for OpenanyClient {
    fn basis(&self) -> &str {
        OpenanyClient::basis(self)
    }

    async fn delta(&self, seit: i64) -> Result<Delta, OpenanyError> {
        OpenanyClient::delta(self, seit).await
    }

    async fn notiztext(&self, zk_id: &str) -> Result<String, OpenanyError> {
        OpenanyClient::notiztext(self, zk_id).await
    }

    async fn anwenden(&self, eintraege: &[Eintrag]) -> Result<Vec<Ergebnis>, OpenanyError> {
        OpenanyClient::anwenden(self, eintraege).await
    }

    async fn kontaktfoto(&self, uuid: &str) -> Result<Vec<u8>, OpenanyError> {
        OpenanyClient::inhalt(self, "contact_photo", uuid).await
    }
}
