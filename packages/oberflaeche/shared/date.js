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
// Vorher stand in fast jeder Ansicht eine eigene Fassung – vierzehn insgesamt.
// Das war nicht nur Doppelung: Sechs davon hatten 'de-DE' fest verdrahtet.
// Wer die Sprache auf Englisch stellte, bekam auf der Startseite, im Projekt,
// im Chat und in den Notizen weiter deutsche Datumsangaben, in Einstellungen,
// Kalender und Papierkorb dagegen englische.

const LEER = '–';

const STILE = {
  date: { day: '2-digit', month: '2-digit', year: 'numeric' },
  dateShort: { day: '2-digit', month: 'short', year: 'numeric' },
  dateLong: { day: '2-digit', month: 'long', year: 'numeric' },
  // Ohne Jahr – für den Kalenderkopf auf dem Telefon, siehe
  // formatDayRangeShort.
  dayShort: { day: '2-digit', month: 'short' },
  dateTime: { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  dateTimeShort: { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  // Terminvorschläge: Wochentag trägt die Aussage, das Jahr ist Beiwerk.
  weekdayTime: { weekday: 'short', day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' },
  weekday: { weekday: 'short' },
  weekdayLong: { weekday: 'long' },
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

/** 01. August 2026 */
export const formatDateLong = (wert, leer = LEER) => anzeigen('dateLong', wert, leer);

/** 01.08.2026, 14:30 */
export const formatDateTime = (wert, leer = LEER) => anzeigen('dateTime', wert, leer);

/** 01. Aug. 2026, 14:30 */
export const formatDateTimeShort = (wert, leer = LEER) => anzeigen('dateTimeShort', wert, leer);

/** Sa., 01.08., 14:30 */
export const formatWeekdayTime = (wert, leer = LEER) => anzeigen('weekdayTime', wert, leer);

/** Sa. */
export const formatWeekday = (wert, leer = LEER) => anzeigen('weekday', wert, leer);

/** Samstag */
export const formatWeekdayLong = (wert, leer = LEER) => anzeigen('weekdayLong', wert, leer);

/** 14:30 */
export const formatTime = (wert, leer = LEER) => anzeigen('time', wert, leer);

/** 13.–19. Juli 2026 – ein Zeitraum, den die Sprache selbst zusammenzieht */
export function formatDateRangeShort(von, bis, leer = LEER) {
  const a = toDate(von);
  const b = toDate(bis);

  return a === null || b === null ? leer : formatter('dateShort').formatRange(a, b);
}

/**
 * Derselbe Zeitraum ohne Jahr: 13.–19. Juli.
 *
 * Für den Kalenderkopf auf dem Telefon. Dort stehen vier Bedienelemente
 * nebeneinander, und die Jahreszahl ist das Erste, worauf man verzichten
 * kann: Wer eine einzelne Woche ansieht, hat sie gerade selbst angesteuert.
 * Ab `sm` steht sie wieder da.
 */
export function formatDayRangeShort(von, bis, leer = LEER) {
  const a = toDate(von);
  const b = toDate(bis);

  return a === null || b === null ? leer : formatter('dayShort').formatRange(a, b);
}
