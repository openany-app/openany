/*
 * Wann ein Bild oder Video entstand -- und die Galerie danach in Monaten
 * (Tiffy, 02.10.2026: neueste zuerst, auch in Alben, mit Monatsüberschriften).
 *
 * Das Aufnahmedatum kommt aus den Fotodaten (EXIF `DateTimeOriginal`, beim
 * Hochladen in `custom_properties.exif.date` abgelegt, Form
 * „2026:08:15 14:03:00"). Fehlt es -- Screenshot, bearbeitetes Bild, Video --,
 * zählt, wann es hochgeladen wurde. Der Server führt dasselbe als Spalte
 * `aufgenommen_at`, damit auch das Blättern stimmt; hier gilt es für das, was
 * geladen ist, und für das Programm.
 */

/** @returns {number} Millisekunden, 0 wenn gar nichts bekannt ist */
export function aufnahmezeit(bild) {
  const exif = bild?.custom_properties?.exif?.date;
  if (typeof exif === 'string') {
    const m = exif.match(/^(\d{4})[:-](\d{2})[:-](\d{2})[ T](\d{2}):(\d{2}):(\d{2})/);
    if (m) {
      const t = new Date(+m[1], +m[2] - 1, +m[3], +m[4], +m[5], +m[6]).getTime();
      if (!Number.isNaN(t) && +m[1] > 1900) return t;
    }
  }
  for (const feld of ['aufgenommen_at', 'created_at', 'updated_at']) {
    const t = bild?.[feld] ? new Date(bild[feld]).getTime() : NaN;
    if (!Number.isNaN(t)) return t;
  }
  return 0;
}

/** Neueste zuerst; bei gleicher Zeit bleibt die bisherige Reihenfolge. */
export function nachAufnahme(bilder) {
  return bilder
    .map((bild, i) => ({ bild, i, t: aufnahmezeit(bild) }))
    .sort((a, b) => b.t - a.t || a.i - b.i)
    .map((x) => x.bild);
}

/**
 * In Monate geteilt, neueste zuerst.
 * @returns {{ schluessel: string, titel: string, bilder: object[] }[]}
 */
export function nachMonaten(bilder, sprache = 'de') {
  const format = new Intl.DateTimeFormat(sprache, { month: 'long', year: 'numeric' });
  const gruppen = [];
  for (const bild of nachAufnahme(bilder)) {
    const t = aufnahmezeit(bild);
    const d = new Date(t);
    const schluessel = t ? `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}` : 'ohne';
    let g = gruppen[gruppen.length - 1];
    if (!g || g.schluessel !== schluessel) {
      g = { schluessel, titel: t ? format.format(d) : '', bilder: [] };
      gruppen.push(g);
    }
    g.bilder.push(bild);
  }
  return gruppen;
}
