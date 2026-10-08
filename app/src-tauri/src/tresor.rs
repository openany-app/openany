//! Die Ausweise im Tresor -- unter Android verschlossen mit einem Schluessel
//! aus dem Keystore.
//!
//! Drei Dinge machen aus diesem Geraet einen Zugang: das anyid-Geraetetoken,
//! der openany-Geraeteschluessel und die Matrix-Sitzung (samt der Passphrase,
//! mit der das SDK seinen eigenen Speicher verschliesst). Bis zum 22.09.2026
//! lagen alle drei als Klartext in `ausweise/` -- ueber
//! `anyid_client::Dateispeicher`, der sich selbst "fuer die Entwicklung"
//! nennt.
//!
//! **Unter Android** verschliesst jetzt `Tresor.kt` jeden Ausweis mit
//! AES-256-GCM; der Schluessel liegt im Keystore und verlaesst ihn nie. In der
//! Datei steht die Kennung [`KENNUNG`] und dahinter `IV ‖ Geheimtext ‖ Tag`.
//! Eine Sicherung des Geraets nimmt die Datei mit, aber nicht den Schluessel:
//! Auf einem anderen Geraet ist sie Datenmuell, und das Programm verlangt eine
//! neue Kopplung.
//!
//! **Auf dem Schreibtisch** bleibt es vorerst bei der Datei. Der Schluesselbund
//! von macOS, Windows und Linux ist eine eigene Stufe; der Anlass hier war das
//! Telefon, das in fremde Haende geraten kann.
//!
//! **Alte Ausweise gehen nicht verloren.** Findet [`Tresorablage::lesen`] eine
//! Datei ohne Kennung, ist das ein Ausweis aus der Zeit davor: Er wird gelesen
//! und sofort verschlossen zurueckgeschrieben. Eine Aktualisierung des
//! Programms kostet so keine Kopplung.

// Auf dem Schreibtisch nimmt `ablage` die Datei aus anyid_client; die
// Tresorablage hat dort nur ihre Tests. Unter Android ist alles in Gebrauch.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use anyid_client::Tokenspeicher;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Woran eine verschlossene Datei zu erkennen ist. Ein Token beginnt nie so --
/// sie sind druckbar und einzeilig.
pub const KENNUNG: &[u8] = b"OATRESOR1\n";

/// Was verschliesst und oeffnet. Unter Android der Keystore, in den Tests
/// eine Attrappe.
pub trait Schloss: Send + Sync {
    fn zu(&self, klar: &[u8]) -> io::Result<Vec<u8>>;

    /// `None`: Das laesst sich nicht mehr oeffnen (Schluessel fort, fremdes
    /// Geraet). Das ist kein Fehler, sondern "not paired".
    fn auf(&self, zu: &[u8]) -> io::Result<Option<Vec<u8>>>;
}

/// Ein Ausweis in einer Datei, verschlossen mit einem [`Schloss`].
pub struct Tresorablage<S: Schloss> {
    pfad: PathBuf,
    schloss: S,
}

impl<S: Schloss> Tresorablage<S> {
    pub fn neu(pfad: impl Into<PathBuf>, schloss: S) -> Self {
        Self {
            pfad: pfad.into(),
            schloss,
        }
    }
}

fn nicht_leer(text: String) -> Option<String> {
    let t = text.trim();

    (!t.is_empty()).then(|| t.to_string())
}

impl<S: Schloss> Tokenspeicher for Tresorablage<S> {
    fn lesen(&self) -> io::Result<Option<String>> {
        let roh = match fs::read(&self.pfad) {
            Ok(roh) => roh,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        if let Some(zu) = roh.strip_prefix(KENNUNG) {
            return match self.schloss.auf(zu)? {
                Some(klar) => Ok(nicht_leer(
                    String::from_utf8(klar).map_err(io::Error::other)?,
                )),
                None => Ok(None),
            };
        }

        // Klartext aus der Zeit vor dem Tresor: einmal lesen, gleich
        // verschlossen zurueck. Scheitert das Verschliessen, bleibt der
        // Ausweis trotzdem nutzbar -- lieber einmal Klartext weiter als ein
        // Mensch, der ohne Not neu koppeln muss.
        let Some(token) = nicht_leer(String::from_utf8(roh).map_err(io::Error::other)?) else {
            return Ok(None);
        };

        if let Err(e) = self.schreiben(&token) {
            eprintln!("Vault: old credentials stay unlocked ({e})");
        }

        Ok(Some(token))
    }

    fn schreiben(&self, token: &str) -> io::Result<()> {
        let zu = self.schloss.zu(token.as_bytes())?;

        if let Some(ordner) = self.pfad.parent() {
            fs::create_dir_all(ordner)?;
        }

        // Erst daneben, dann an die Stelle: Ein Absturz mitten im Schreiben
        // hinterlaesst keinen halben Ausweis, der sich nie mehr oeffnet.
        let neu = self.pfad.with_extension("neu");
        fs::write(&neu, [KENNUNG, &zu].concat())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&neu, fs::Permissions::from_mode(0o600))?;
        }

        fs::rename(&neu, &self.pfad)
    }

    fn vergessen(&self) -> io::Result<()> {
        match fs::remove_file(&self.pfad) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            anders => anders,
        }
    }
}

/// Die Ablage fuer einen Ausweis -- je nach Plattform.
pub fn ablage(pfad: PathBuf) -> Box<dyn Tokenspeicher + Send + Sync> {
    #[cfg(target_os = "android")]
    {
        Box::new(Tresorablage::neu(pfad, android::Keystore))
    }

    #[cfg(not(target_os = "android"))]
    {
        Box::new(anyid_client::Dateispeicher::neu(pfad))
    }
}

/// Die Bruecke zu `Tresor.kt`.
///
/// **Warum sich die Klasse selbst anmeldet.** JNI findet die Klassen dieser
/// App nur auf einem Java-Faden (`FindClass` fragt sonst den Klassenlader des
/// Systems). Die Befehle laufen aber auf Faeden von tokio. Deshalb ruft
/// `Tresor.kt` beim Laden `anmelden()` -- auf einem Java-Faden --, und hier
/// wird die Klasse als globale Referenz gehalten. Von ihr aus geht ein Aufruf
/// von jedem Faden.
#[cfg(target_os = "android")]
mod android {
    use super::Schloss;
    use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JValue};
    use jni::{JNIEnv, JavaVM};
    use std::io;
    use std::sync::OnceLock;

    static TRESOR: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Tresor_anmelden(env: JNIEnv, klasse: JClass) {
        if TRESOR.get().is_some() {
            return;
        }

        if let (Ok(vm), Ok(global)) = (env.get_java_vm(), env.new_global_ref(&klasse)) {
            let _ = TRESOR.set((vm, global));
        }
    }

    pub struct Keystore;

    impl Schloss for Keystore {
        fn zu(&self, klar: &[u8]) -> io::Result<Vec<u8>> {
            rufen("verschliessen", klar)?
                .ok_or_else(|| io::Error::other("The vault returned nothing."))
        }

        fn auf(&self, zu: &[u8]) -> io::Result<Option<Vec<u8>>> {
            rufen("oeffnen", zu)
        }
    }

    fn fehler(e: impl std::fmt::Display) -> io::Error {
        io::Error::other(format!("Vault: {e}"))
    }

    fn rufen(methode: &str, daten: &[u8]) -> io::Result<Option<Vec<u8>>> {
        let (vm, klasse) = TRESOR
            .get()
            .ok_or_else(|| fehler("not registered (Tresor.bereit() was never called)"))?;

        // Dauerhaft angehaengt: Ein tokio-Faden kommt wieder, und jedes Mal
        // an- und abzuhaengen kostet mehr als der Aufruf selbst. Die lokalen
        // Referenzen raeumt der Rahmen darunter ab -- ein angehaengter
        // Faden ohne Java-Rahmen gaebe sie sonst nie frei.
        let mut env = vm.attach_current_thread_permanently().map_err(fehler)?;

        let ergebnis = env.with_local_frame(8, |env| -> jni::errors::Result<Option<Vec<u8>>> {
            let eingabe = env.byte_array_from_slice(daten)?;
            let klasse: &JClass = klasse.as_obj().into();
            let antwort = env
                .call_static_method(
                    klasse,
                    methode,
                    "([B)[B",
                    &[JValue::Object(&JObject::from(eingabe))],
                )?
                .l()?;

            if antwort.is_null() {
                return Ok(None);
            }

            Ok(Some(env.convert_byte_array(JByteArray::from(antwort))?))
        });

        ergebnis.map_err(|e| {
            // Eine Java-Ausnahme bleibt sonst haengen und bricht den
            // naechsten JNI-Aufruf auf diesem Faden.
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }

            fehler(e)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Ein Schloss aus Pappe: kehrt die Bytes um und setzt ein Zeichen davor.
    /// Kryptographisch wertlos, aber es zeigt, ob wirklich verschlossen wurde.
    #[derive(Default)]
    struct Pappschloss {
        verloren: AtomicBool,
        kaputt: AtomicBool,
    }

    impl Schloss for Pappschloss {
        fn zu(&self, klar: &[u8]) -> io::Result<Vec<u8>> {
            if self.kaputt.load(Ordering::SeqCst) {
                return Err(io::Error::other("Keystore gone"));
            }

            Ok([
                b"#".as_slice(),
                &klar.iter().rev().copied().collect::<Vec<_>>(),
            ]
            .concat())
        }

        fn auf(&self, zu: &[u8]) -> io::Result<Option<Vec<u8>>> {
            if self.verloren.load(Ordering::SeqCst) {
                return Ok(None);
            }

            Ok(zu
                .strip_prefix(b"#")
                .map(|r| r.iter().rev().copied().collect()))
        }
    }

    fn ablage(ordner: &tempfile::TempDir) -> Tresorablage<Pappschloss> {
        Tresorablage::neu(
            ordner.path().join("ausweise").join("anyid-ausweis"),
            Pappschloss::default(),
        )
    }

    #[test]
    fn hin_und_zurueck() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);

        a.schreiben("geheim-123").unwrap();

        assert_eq!(a.lesen().unwrap().as_deref(), Some("geheim-123"));
    }

    /// DER KERN: In der Datei steht der Ausweis nicht mehr lesbar.
    #[test]
    fn in_der_datei_steht_kein_klartext() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);

        a.schreiben("geheim-123").unwrap();

        let roh = fs::read(&a.pfad).unwrap();
        assert!(roh.starts_with(KENNUNG));
        assert!(!String::from_utf8_lossy(&roh).contains("geheim-123"));
    }

    /// Ein Ausweis von vor dem Tresor wird gelesen UND gleich verschlossen.
    #[test]
    fn ein_alter_ausweis_wird_verschlossen() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);
        fs::create_dir_all(a.pfad.parent().unwrap()).unwrap();
        fs::write(&a.pfad, "alt-456\n").unwrap();

        assert_eq!(a.lesen().unwrap().as_deref(), Some("alt-456"));
        assert!(
            fs::read(&a.pfad).unwrap().starts_with(KENNUNG),
            "danach verschlossen"
        );
        assert_eq!(
            a.lesen().unwrap().as_deref(),
            Some("alt-456"),
            "und weiter lesbar"
        );
    }

    /// Kann der Tresor nicht verschliessen, bleibt der alte Ausweis nutzbar.
    #[test]
    fn ohne_tresor_bleibt_der_alte_ausweis_nutzbar() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);
        fs::create_dir_all(a.pfad.parent().unwrap()).unwrap();
        fs::write(&a.pfad, "alt-456").unwrap();
        a.schloss.kaputt.store(true, Ordering::SeqCst);

        assert_eq!(a.lesen().unwrap().as_deref(), Some("alt-456"));
    }

    /// Schluessel fort (Sicherung auf einem anderen Geraet): nicht gekoppelt,
    /// kein Fehler.
    #[test]
    fn ohne_schluessel_ist_das_geraet_nicht_gekoppelt() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);
        a.schreiben("geheim-123").unwrap();
        a.schloss.verloren.store(true, Ordering::SeqCst);

        assert_eq!(a.lesen().unwrap(), None);
    }

    #[test]
    fn nichts_da_ist_nichts_da() {
        let o = tempfile::tempdir().unwrap();

        assert_eq!(ablage(&o).lesen().unwrap(), None);
    }

    #[test]
    fn vergessen_raeumt_weg_auch_zweimal() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);
        a.schreiben("geheim-123").unwrap();

        a.vergessen().unwrap();
        a.vergessen().unwrap();

        assert_eq!(a.lesen().unwrap(), None);
    }

    /// Kein halber Ausweis: Die Zwischendatei ist nach dem Schreiben fort.
    #[test]
    fn es_bleibt_keine_zwischendatei() {
        let o = tempfile::tempdir().unwrap();
        let a = ablage(&o);

        a.schreiben("geheim-123").unwrap();

        assert!(!a.pfad.with_extension("neu").exists());
    }
}
