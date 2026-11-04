/*
 * Tondateien in der Mediathek: erkennen und ihre Dauer lesen.
 *
 * NACH TYP UND ENDUNG. Die Dateiauswahl mancher Geräte meldet bei .opus oder
 * .m4a gar keinen Typ, die Datei soll trotzdem als Ton durchgehen.
 */
const NACH_ENDUNG = {
  aac: 'audio/aac', flac: 'audio/flac', m4a: 'audio/mp4', mp3: 'audio/mpeg',
  oga: 'audio/ogg', ogg: 'audio/ogg', opus: 'audio/ogg', wav: 'audio/wav', weba: 'audio/webm',
};

/** Der Typ zu einem Dateinamen, '' wenn es keine bekannte Tondatei ist. */
export function tonMime(name) {
  return NACH_ENDUNG[String(name ?? '').toLowerCase().split('.').pop()] ?? '';
}

/** Ist das Ton? Für eine Datei wie für einen Eintrag der Mediathek. */
export function istTon(datei) {
  const typ = String(datei?.type ?? datei?.mime_type ?? datei?.mime ?? '');
  return typ ? typ.startsWith('audio/') : Boolean(tonMime(datei?.name));
}

/**
 * Die Dauer in Sekunden, oder null. Darf scheitern: Ohne Dauer zeigt die
 * Kachel eben keine. Aufgenommenes WebM meldet zunächst „unendlich", das
 * zählt als unbekannt.
 */
export function tonDauer(datei) {
  return new Promise((fertig) => {
    const adresse = URL.createObjectURL(datei);
    const ton = new Audio();
    const ende = (wert) => {
      clearTimeout(frist);
      URL.revokeObjectURL(adresse);
      fertig(wert);
    };
    const frist = setTimeout(() => ende(null), 5000);
    ton.preload = 'metadata';
    ton.onloadedmetadata = () => ende(Number.isFinite(ton.duration) && ton.duration > 0 ? ton.duration : null);
    ton.onerror = () => ende(null);
    ton.src = adresse;
  });
}

/** Bis zu dieser Größe wird eine Datei für die Wellenform dekodiert: Entpackter Ton braucht ein Vielfaches an Speicher. */
export const WELLE_HOECHSTENS = 32 * 1024 * 1024;

/**
 * `anzahl` Balken aus den Kanälen eines Tons: je Abschnitt die lauteste Stelle,
 * bezogen auf die lauteste der ganzen Datei (0–1).
 */
export function spitzen(kanaele, anzahl) {
  const laenge = kanaele[0]?.length ?? 0;
  const balken = new Array(anzahl).fill(0);
  if (!laenge) return balken;
  for (const kanal of kanaele) {
    for (let i = 0; i < laenge; i++) {
      const b = Math.min(anzahl - 1, Math.floor((i * anzahl) / laenge));
      const wert = Math.abs(kanal[i]);
      if (wert > balken[b]) balken[b] = wert;
    }
  }
  const hoechst = Math.max(...balken);
  return hoechst ? balken.map((w) => w / hoechst) : balken;
}

/**
 * Die Wellenform einer Tondatei, oder null, wenn sie sich hier nicht dekodieren
 * lässt. Der Player zeigt dann eine flache Leiste. 8 kHz reichen für die Form
 * und halten den Speicherbedarf klein.
 */
export async function wellenform(blob, anzahl) {
  if (!blob || blob.size > WELLE_HOECHSTENS || typeof OfflineAudioContext === 'undefined') return null;
  try {
    const ton = await new OfflineAudioContext(1, 1, 8000).decodeAudioData(await blob.arrayBuffer());
    const kanaele = [];
    for (let k = 0; k < ton.numberOfChannels; k++) kanaele.push(ton.getChannelData(k));
    return spitzen(kanaele, anzahl);
  } catch {
    return null;
  }
}
