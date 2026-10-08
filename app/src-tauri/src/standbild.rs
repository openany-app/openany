//! Das Standbild eines Videos, gezogen vom Betriebssystem.
//!
//! Auf Android über `MediaMetadataRetriever` (`Standbild.kt`, per JNI): liest
//! direkt aus der Datei, mit dem Hardware-Decoder, ohne Webansicht und ohne
//! Abspielen -- also auch im Hintergrund nach dem Abgleich. Entschieden am
//! 27.09.2026, nachdem Firefox auf Android nur schwarze Standbilder lieferte
//! und Videos in der Webansicht des Programms nicht zuverlässig spielten
//! (docs/plan-videos-galerie.md).
//!
//! Auf dem Schreibtisch gibt es (noch) keinen Weg: `None`. Dort kann das
//! Standbild später aus der Webansicht kommen, die Videos dort abspielt.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use std::path::Path;

/// `(JPEG, Dauer in Sekunden)`. Das JPEG fehlt, wenn kein Bild zu ziehen war
/// -- die Dauer steht dann trotzdem da.
pub struct Standbild {
    pub jpeg: Option<Vec<u8>>,
    pub dauer: Option<f64>,
}

/// Aus der Antwort von `Standbild.ziehen`: `[Dauer ms, 8 Bytes BE] ‖ JPEG`.
fn zerlegen(roh: &[u8]) -> Option<Standbild> {
    let (kopf, jpeg) = roh.split_at_checked(8)?;
    let ms = u64::from_be_bytes(kopf.try_into().ok()?);
    Some(Standbild {
        jpeg: (!jpeg.is_empty()).then(|| jpeg.to_vec()),
        dauer: (ms > 0).then(|| ms as f64 / 1000.0),
    })
}

/// Blockiert (Dekodieren dauert): nicht auf dem Laufzeitkern rufen.
pub fn ziehen(pfad: &Path) -> Option<Standbild> {
    #[cfg(target_os = "android")]
    {
        android::rufen(&pfad.to_string_lossy()).and_then(|roh| zerlegen(&roh))
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = pfad;
        None
    }
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JValue};
    use jni::{JNIEnv, JavaVM};
    use std::sync::OnceLock;

    static STANDBILD: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();

    #[no_mangle]
    pub extern "system" fn Java_de_openany_app_Standbild_anmelden(env: JNIEnv, klasse: JClass) {
        if STANDBILD.get().is_some() {
            return;
        }
        if let (Ok(vm), Ok(global)) = (env.get_java_vm(), env.new_global_ref(&klasse)) {
            let _ = STANDBILD.set((vm, global));
        }
    }

    pub(super) fn rufen(pfad: &str) -> Option<Vec<u8>> {
        let (vm, klasse) = STANDBILD.get()?;
        // Dauerhaft angehängt wie beim Tresor; die lokalen Referenzen räumt
        // der Rahmen ab.
        let mut env = vm.attach_current_thread_permanently().ok()?;
        let ergebnis = env.with_local_frame(8, |env| -> jni::errors::Result<Option<Vec<u8>>> {
            let text = env.new_string(pfad)?;
            let klasse: &JClass = klasse.as_obj().into();
            let antwort = env
                .call_static_method(
                    klasse,
                    "ziehen",
                    "(Ljava/lang/String;)[B",
                    &[JValue::Object(&JObject::from(text))],
                )?
                .l()?;
            if antwort.is_null() {
                return Ok(None);
            }
            Ok(Some(env.convert_byte_array(JByteArray::from(antwort))?))
        });
        match ergebnis {
            Ok(r) => r,
            Err(_) => {
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_clear();
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::zerlegen;

    #[test]
    fn antwort_zerlegen() {
        let mut roh = 26_200u64.to_be_bytes().to_vec();
        roh.extend_from_slice(b"\xFF\xD8jpeg");
        let s = zerlegen(&roh).unwrap();
        assert_eq!(s.dauer, Some(26.2));
        assert_eq!(s.jpeg.as_deref(), Some(&b"\xFF\xD8jpeg"[..]));

        let nur_dauer = zerlegen(&5_000u64.to_be_bytes()).unwrap();
        assert!(nur_dauer.jpeg.is_none());
        assert_eq!(nur_dauer.dauer, Some(5.0));

        assert!(zerlegen(b"kurz").is_none());
    }
}
