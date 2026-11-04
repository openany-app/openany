//! Der direkte Abgleich von selbst: Solange das Programm läuft, gleicht es mit
//! jedem gepaarten Gerät ab, das in der Nähe auftaucht, ohne dass jemand tippt.
//!
//! Am Schreibtisch läuft das, solange das Programm läuft. Unter Android nur,
//! solange der Prozess lebt; dazu versucht es der Auffrischer etwa stündlich
//! ([`einmal_bei_allen`]). Das andere Gerät muss dafür erreichbar sein. Deshalb
//! trägt ein Rechner, der durchläuft, die Fotos zwischen den Handys.

use crate::Zustand;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Gleichen zwei Geräte zugleich miteinander ab, weist eines das andere als
/// beschäftigt ab. Die Streuung im Takt lässt sie danach auseinanderlaufen.
fn takt() -> Duration {
    let streuung = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_millis() as u64 % 30)
        .unwrap_or(0);
    Duration::from_secs(45 + streuung)
}
/// So lange ruht ein Gerät nach einem gelungenen Lauf.
const ABSTAND: Duration = Duration::from_secs(5 * 60);

pub(crate) fn starten(zustand: Arc<Zustand>) {
    tauri::async_runtime::spawn(async move {
        let mut zuletzt: HashMap<String, Instant> = HashMap::new();
        tokio::time::sleep(Duration::from_secs(15)).await;
        loop {
            runde(&zustand, &mut zuletzt).await;
            tokio::time::sleep(takt()).await;
        }
    });
}

async fn runde(zustand: &Arc<Zustand>, zuletzt: &mut HashMap<String, Instant>) {
    let gepaarte = zustand.gastgeber.gepaarte();
    if gepaarte.is_empty() {
        return;
    }
    if let Some(n) = zustand.nah.lock().await.as_ref() {
        n.suchen();
    }
    for (fp, _) in gepaarte {
        if zuletzt.get(&fp).is_some_and(|t| t.elapsed() < ABSTAND) {
            continue;
        }
        let da = zustand
            .nah
            .lock()
            .await
            .as_ref()
            .and_then(|n| n.letzte_adresse(&fp))
            .is_some();
        if !da {
            continue;
        }
        if let Ok(anzeige) = crate::nah_lauf(zustand, &fp).await {
            zuletzt.insert(fp, Instant::now());
            // Die Oberfläche liest den neuen Stand, wie nach einem Lauf vom
            // anderen Gerät aus.
            let geholt = anzeige.inhalte.as_ref().is_some_and(|i| i.geholt > 0);
            if anzeige.gezogen > 0 || anzeige.konflikte > 0 || geholt {
                zustand.gastgeber.angenommen.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Einmal alle gepaarten Geräte versuchen, für den Auffrischer. Gibt zurück,
/// mit wie vielen es geklappt hat.
pub(crate) async fn einmal_bei_allen(zustand: &Arc<Zustand>) -> usize {
    let gepaarte = zustand.gastgeber.gepaarte();
    if gepaarte.is_empty() {
        return 0;
    }
    if zustand.nah.lock().await.is_none() {
        crate::nah_starten(zustand.clone()).await;
    }
    let mut gelungen = 0;
    for (fp, _) in gepaarte {
        let lauf = tokio::time::timeout(Duration::from_secs(4 * 60), crate::nah_lauf(zustand, &fp));
        if matches!(lauf.await, Ok(Ok(_))) {
            gelungen += 1;
        }
    }
    gelungen
}
