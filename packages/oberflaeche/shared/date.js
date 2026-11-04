import { i18n } from '@oberflaeche/i18n';

// Lokales Datum als YYYY-MM-DD (ohne Zeitzonen-Verschiebung, anders als
// toISOString, das auf UTC umrechnet).
export function getLocalDateString(d) {
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// ---------- Anzeige ----------
//
// EINE Fassung für alle Ansichten, nach der eingestellten Sprache -- nicht
// eine je Ansicht mit fest verdrahtetem 'de-DE'.

const LEER = '–';

const STILE = {
  date: { day: '2-digit', month: '2-digit', year: 'numeric' },
  dateShort: { day: '2-digit', month: 'short', year: 'numeric' },
  dateTime: { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  dateTimeShort: { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  // Terminvorschläge: Wochentag trägt die Aussage, das Jahr ist Beiwerk.
  weekday: { weekday: 'short' },
  time: { hour: '2-digit', minute: '2-digit' },
};

// Intl.DateTimeFormat ist teuer zu erzeugen, aber beliebig wiederverwendbar.
// toLocaleString() legt bei JEDEM Aufruf einen neuen an – in langen Listen
// (Papierkorb, Chat, Notizen) ist das der Unterschied zwischen einer Instanz
// je Zeile und einer je Sprache und Stil.
const cache = new Map();

function formatter(stil) {
  // Bewusst bei jedem Aufruf gelesen: Dadurch hängt die Vorlage reaktiv an der
  // Sprache und Datumsangaben wechseln beim Umschalten sofort mit.
  const sprache = i18n.global.locale.value;
  const schluessel = `${sprache}|${stil}`;

  if (!cache.has(schluessel)) {
    cache.set(schluessel, new Intl.DateTimeFormat(sprache, STILE[stil]));
  }

  return cache.get(schluessel);
}

// Reine Datumsangaben (YYYY-MM-DD) liest new Date() als UTC-Mitternacht. In
// Zeitzonen westlich von Greenwich zeigt das den VORTAG an. Die Uhrzeit
// anzuhängen erzwingt die lokale Auslegung.
const NUR_DATUM = /^\d{4}-\d{2}-\d{2}$/;

// Kalenderwoche nach ISO 8601 (Montag zuerst; Woche 1 enthält den
// ersten Donnerstag des Jahres) -- so, wie sie in Deutschland gezählt wird.
export function isoWoche(datum) {
  const d = new Date(Date.UTC(datum.getFullYear(), datum.getMonth(), datum.getDate()));
  const tag = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - tag);
  const jahresanfang = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  return Math.ceil(((d - jahresanfang) / 86400000 + 1) / 7);
}

export function toDate(wert) {
  if (wert instanceof Date) return wert;
  if (typeof wert !== 'string' || wert === '') return null;

  const d = new Date(NUR_DATUM.test(wert) ? `${wert}T00:00:00` : wert);

  return Number.isNaN(d.getTime()) ? null : d;
}

function anzeigen(stil, wert, leer) {
  const d = toDate(wert);

  return d === null ? leer : formatter(stil).format(d);
}

/** 01.08.2026 */
export const formatDate = (wert, leer = LEER) => anzeigen('date', wert, leer);

/** 01. Aug. 2026 */
export const formatDateShort = (wert, leer = LEER) => anzeigen('dateShort', wert, leer);

/** 01.08.2026, 14:30 */
export const formatDateTime = (wert, leer = LEER) => anzeigen('dateTime', wert, leer);

/** 01. Aug. 2026, 14:30 */
export const formatDateTimeShort = (wert, leer = LEER) => anzeigen('dateTimeShort', wert, leer);

/** Sa. */
export const formatWeekday = (wert, leer = LEER) => anzeigen('weekday', wert, leer);

/** 14:30 */
export const formatTime = (wert, leer = LEER) => anzeigen('time', wert, leer);

/** 13.–19. Juli 2026 – ein Zeitraum, den die Sprache selbst zusammenzieht */
export function formatDateRangeShort(von, bis, leer = LEER) {
  const a = toDate(von);
  const b = toDate(bis);

  return a === null || b === null ? leer : formatter('dateShort').formatRange(a, b);
}
