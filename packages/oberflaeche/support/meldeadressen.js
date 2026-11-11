/*
 * Die Adressen der Meldeseite, eine je Sprache (docs/plan-registrierung-app.md,
 * A5). Alle liefern dieselbe Seite; die Sprache folgt aus dem Pfad. Die App
 * verlinkt die Adresse ihrer eingestellten Sprache.
 *
 * Ohne Sonderzeichen, damit man sie abtippen kann; Portugiesisch nimmt das
 * Substantiv, weil `denunciar` schon Spanisch ist.
 */
export const MELDEPFADE = {
  de: 'melden',
  en: 'report',
  fr: 'signaler',
  es: 'denunciar',
  pt: 'denuncia',
  pl: 'zglos',
  uk: 'skarga',
};

/** Die Sprache zu einem Pfad wie `/report`; Deutsch, wenn keiner passt. */
export function spracheAusPfad(pfad) {
  const teil = String(pfad ?? '').replace(/^\/+|\/+$/g, '').split('/')[0];
  return Object.keys(MELDEPFADE).find((s) => MELDEPFADE[s] === teil) ?? 'de';
}

/**
 * Die volle Adresse für eine Sprache (Kürzel wie `de` oder `pt-BR`). Mit
 * `art` ist im Formular diese Art schon gewählt, etwa `widerspruch`.
 */
export function meldeadresse(sprache, basis = 'https://openany.de', art = null) {
  const kurz = String(sprache ?? 'de').slice(0, 2);
  return `${basis}/${MELDEPFADE[kurz] ?? MELDEPFADE.de}${art ? `?art=${encodeURIComponent(art)}` : ''}`;
}
