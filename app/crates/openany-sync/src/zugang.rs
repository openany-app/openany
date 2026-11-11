//! Wie der Laeufer an den Speicher kommt: je Schritt, nicht fuer den ganzen
//! Lauf.
//!
//! **Warum.** Ein Lauf wartet die meiste Zeit auf das Netz. Hielte er den
//! Speicher dabei fest, wartete alles andere mit -- und verklemmte sich, wenn
//! die Gegenstelle waehrend des Laufs selbst an den Speicher muss (etwa um
//! einen Projektschluessel zu pruefen): Sie wartete auf eine Sperre, die erst
//! nach ihr frei wird.
//!
//! **Was zusammenbleibt.** Ein Aufruf von [`Speicherzugang::mit`] ist ein
//! Stueck ohne Netz: Lesen, Entscheiden und Schreiben eines Eintrags laufen
//! darin am Stueck, sodass niemand zwischen Lesen und Schreiben etwas
//! aendert. Zwischen zwei Stuecken darf geschrieben werden; das eigene
//! Protokoll zaehlt fortlaufend, also geht dabei nichts verloren.

use openany_store::Speicher;
use std::future::Future;

pub trait Speicherzugang: Sync {
    /// `f` mit dem Speicher ausfuehren -- ohne dazwischen auf etwas zu warten.
    fn mit<R: Send>(&self, f: impl FnOnce(&Speicher) -> R + Send)
        -> impl Future<Output = R> + Send;
}

/// Ohne Sperre: fuer Tests und alles, was den Speicher allein hat.
impl Speicherzugang for Speicher {
    fn mit<R: Send>(
        &self,
        f: impl FnOnce(&Speicher) -> R + Send,
    ) -> impl Future<Output = R> + Send {
        std::future::ready(f(self))
    }
}

/// Der geteilte Speicher der App: je Aufruf kurz gesperrt.
impl Speicherzugang for tokio::sync::Mutex<Speicher> {
    async fn mit<R: Send>(&self, f: impl FnOnce(&Speicher) -> R + Send) -> R {
        let speicher = self.lock().await;
        f(&speicher)
    }
}
