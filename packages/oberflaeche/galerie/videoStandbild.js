/*
 * Das Standbild eines Videos, gezogen auf dem Gerät.
 *
 * Der Server kann das nicht (dafür bräuchte er ffmpeg, siehe
 * docs/plan-videos-galerie.md), Browser und Programm können es: Das Video in
 * ein unsichtbares <video> laden, ein Stück hineinspringen, das Bild auf ein
 * Canvas zeichnen, als JPEG hergeben.
 *
 * DARF SCHEITERN, UND SAGT ES MIT `null`. Ein HEVC-Video in einem Browser,
 * der HEVC nicht kann, liefert kein Bild; ein kaputtes Video auch nicht. Das
 * Video wird trotzdem gespeichert -- die Kachel zeigt dann ihr Film-Symbol.
 * Deshalb wirft hier nichts, und nach spätestens fünfzehn Sekunden ist Schluss.
 *
 * NICHT NUR SPRINGEN, SONDERN KURZ ABSPIELEN (26.09.2026). Firefox auf
 * Android zeichnete ein Video, das nur angesprungen und nie abgespielt war,
 * als schwarze Fläche -- mit Hardware-Dekodierung liegt das Bild dann noch
 * nicht dort, wo ein Canvas es lesen kann. Das erste echte Handyvideo bekam
 * so ein Standbild mit Helligkeit 0. Deshalb: stumm abspielen, auf ein
 * tatsächlich gezeigtes Bild warten, anhalten, zeichnen -- und prüfen, ob es
 * schwarz ist. Wenn ja, ein Stück weiter; nach drei Versuchen lieber gar
 * kein Standbild als ein schwarzes.
 */

/** Wie weit hinein: eine Sekunde, damit es nicht das schwarze Anfangsbild ist. */
const SPRUNG = 1;

/** Lange Seite höchstens so groß -- der Server verkleinert ohnehin auf 600. */
const KANTE = 1280;

const FRIST_MS = 15_000;

/** So viele Stellen im Video, bevor aufgegeben wird. */
const VERSUCHE = 3;

/**
 * Warten, bis das <video> ein Bild tatsächlich gezeigt hat. Mit
 * `requestVideoFrameCallback`, wo es das gibt -- das meldet genau das.
 * Sonst zwei Animationsbilder, das reicht in der Praxis.
 */
function gezeigt(video) {
  return new Promise((weiter) => {
    if (typeof video.requestVideoFrameCallback === 'function') {
      video.requestVideoFrameCallback(() => weiter());
      // Kommt kein Bild (angehalten, kaputt), nicht ewig warten.
      setTimeout(weiter, 1500);
      return;
    }
    requestAnimationFrame(() => requestAnimationFrame(() => weiter()));
  });
}

/** Das aktuelle Bild des <video> als Canvas, lange Seite höchstens KANTE. */
function zeichnen(video) {
  const breite = video.videoWidth;
  const hoehe = video.videoHeight;
  if (!breite || !hoehe) return null;
  const faktor = Math.min(1, KANTE / Math.max(breite, hoehe));
  const canvas = document.createElement('canvas');
  canvas.width = Math.round(breite * faktor);
  canvas.height = Math.round(hoehe * faktor);
  canvas.getContext('2d').drawImage(video, 0, 0, canvas.width, canvas.height);
  return canvas;
}

/**
 * Ist das (fast) schwarz? Auf 16×16 verkleinert gemittelt -- billig, und ein
 * echtes Bild hat fast nie einen Mittelwert unter 3 von 255.
 */
export function istSchwarz(canvas) {
  try {
    const klein = document.createElement('canvas');
    klein.width = 16;
    klein.height = 16;
    const ctx = klein.getContext('2d');
    ctx.drawImage(canvas, 0, 0, 16, 16);
    const d = ctx.getImageData(0, 0, 16, 16).data;
    let summe = 0;
    for (let i = 0; i < d.length; i += 4) summe += d[i] + d[i + 1] + d[i + 2];
    return summe / (16 * 16 * 3) < 3;
  } catch {
    // Nicht lesbar (fremde Herkunft): dann wissen wir es nicht und nehmen es.
    return false;
  }
}

function alsJpeg(canvas) {
  return new Promise((fertig) => canvas.toBlob((bild) => fertig(bild), 'image/jpeg', 0.8));
}

/**
 * Aus einer gewählten Datei (beim Hochladen).
 *
 * @param {File|Blob} datei
 * @returns {Promise<{ bild: Blob|null, dauer: number } | null>}
 *   `bild` fehlt, wenn keines zu ziehen war -- die DAUER steht trotzdem in
 *   den Metadaten und kommt mit (Firefox auf Android liefert nur schwarze
 *   Bilder, aber die Dauer kennt er).
 */
export function videoStandbild(datei) {
  if (typeof document === 'undefined') return Promise.resolve(null);
  const url = URL.createObjectURL(datei);
  return ausQuelle(url, { freigeben: () => URL.revokeObjectURL(url) });
}

/**
 * Aus einer Adresse -- im Programm die lokale Datei über das Asset-Protokoll.
 * Das hat eine andere Herkunft als die Seite; mit `crossOrigin` und der
 * CORS-Kopfzeile, die Tauri schickt, bleibt das Canvas trotzdem lesbar.
 *
 * @param {string} adresse
 */
export function videoStandbildAusAdresse(adresse) {
  if (typeof document === 'undefined' || !adresse) return Promise.resolve(null);
  return ausQuelle(adresse, { crossOrigin: 'anonymous' });
}

function ausQuelle(url, { freigeben = () => {}, crossOrigin = null } = {}) {
  return new Promise((fertig) => {
    const video = document.createElement('video');
    if (crossOrigin) video.crossOrigin = crossOrigin;
    let erledigt = false;
    let laeuft = false;
    let versuch = 0;

    const ende = (ergebnis) => {
      if (erledigt) return;
      erledigt = true;
      clearTimeout(uhr);
      video.pause();
      video.removeAttribute('src');
      video.load();
      video.remove();
      freigeben();
      fertig(ergebnis);
    };
    const dauer = () => (Number.isFinite(video.duration) ? video.duration : 0);
    // Ohne Bild, aber mit Dauer, wenn sie bekannt ist.
    const ohneBild = () => (dauer() > 0 ? { bild: null, dauer: dauer() } : null);
    const uhr = setTimeout(() => ende(ohneBild()), FRIST_MS);

    // IM DOKUMENT, aber unsichtbar: Manche Browser dekodieren ein <video>,
    // das nirgends hängt, gar nicht erst.
    Object.assign(video.style, {
      position: 'fixed', left: '-10000px', top: '0', width: '2px', height: '2px', opacity: '0', pointerEvents: 'none',
    });
    video.muted = true;
    video.playsInline = true;
    video.preload = 'auto';
    video.addEventListener('error', () => ende(null));
    video.addEventListener('loadedmetadata', () => {
      // Kurze Clips: in die Mitte statt über das Ende hinaus.
      video.currentTime = Math.min(SPRUNG, dauer() > 0 ? dauer() / 2 : 0);
    });
    video.addEventListener('seeked', async () => {
      if (laeuft || erledigt) return;
      laeuft = true;
      try {
        // Stumm abspielen ist überall ohne Tippen erlaubt; wo nicht, eben
        // nur gesprungen.
        await video.play().catch(() => {});
        await gezeigt(video);
        video.pause();
        const canvas = zeichnen(video);
        if (!canvas) {
          ende(ohneBild());
          return;
        }
        if (istSchwarz(canvas)) {
          versuch += 1;
          if (versuch >= VERSUCHE || dauer() <= 0) {
            ende(ohneBild());
            return;
          }
          laeuft = false;
          video.currentTime = Math.min(dauer() - 0.1, video.currentTime + Math.max(1, dauer() / 5));
          return;
        }
        const bild = await alsJpeg(canvas);
        ende(bild ? { bild, dauer: dauer() } : ohneBild());
      } catch {
        ende(ohneBild());
      }
    });

    document.body.appendChild(video);
    video.src = url;
  });
}

/**
 * Ein Standbild aus einem <video>, das gerade LÄUFT -- im Betrachter, für
 * Videos, die ohne Standbild ankamen (älterer Browser, die Firefox-Macke
 * oben). Das Bild ist dort sicher dekodiert, weil man es sieht.
 *
 * Geht nur, wenn das Video von derselben Herkunft kommt (openany.de oder die
 * lokale Datei im Programm); sonst ist das Canvas gesperrt, und es gibt `null`.
 *
 * @param {HTMLVideoElement} video
 * @returns {Promise<{ bild: Blob, dauer: number } | null>}
 */
export async function standbildAusLaufendem(video) {
  try {
    await gezeigt(video);
    const canvas = zeichnen(video);
    if (!canvas || istSchwarz(canvas)) return null;
    const bild = await alsJpeg(canvas);
    return bild ? { bild, dauer: Number.isFinite(video.duration) ? video.duration : 0 } : null;
  } catch {
    return null;
  }
}

/** Ist das ein Video? Nach dem Typ, den Browser oder System melden. */
export function istVideo(datei) {
  return String(datei?.type ?? datei?.mime_type ?? datei?.mime ?? '').startsWith('video/')
    || ['application/mp4'].includes(String(datei?.mime_type ?? datei?.mime ?? ''));
}

/** 75 → „1:15", 3725 → „1:02:05". */
export function dauerText(sekunden) {
  const s = Math.max(0, Math.round(Number(sekunden) || 0));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = String(s % 60).padStart(2, '0');
  return h ? `${h}:${String(m).padStart(2, '0')}:${r}` : `${m}:${r}`;
}
