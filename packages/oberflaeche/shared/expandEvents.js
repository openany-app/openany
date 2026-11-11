import { getLocalDateString } from '@oberflaeche/shared/date';

// Wiederkehrende Termine in einzelne Instanzen auffalten (bis 2 Jahre in
// die Zukunft bzw. rrule_until, mit EXDATE-Ausnahmen). Instanzen tragen
// real_id + original_event für Bearbeiten/Löschen. Gemeinsame Basis für
// das Monatsraster (useCalendarGrid) und das Kalender-Widget der
// Startseite.
//
// Die Regel: `rrule` ist die Frequenz (DAILY … YEARLY), `rrule_until` das
// Ende, `rrule_parts` der Rest (INTERVAL, COUNT, BYDAY, BYMONTHDAY, BYMONTH,
// BYSETPOS, WKST -- RFC 5545). Ein einzeln verschobenes Vorkommen ist ein
// eigener Termin mit `series` (die Serie) und `occurrence` (sein alter
// Beginn); die Serie lässt diesen Platz aus.

const TAGE = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'];
const GRENZE = 730;

/** `INTERVAL=2;BYDAY=TU,TH` → { INTERVAL: '2', BYDAY: 'TU,TH' }. */
export function regelTeile(text) {
  const teile = {};
  for (const t of (text || '').split(';')) {
    const [name, wert] = t.split('=');
    if (name && wert) teile[name.trim().toUpperCase()] = wert.trim().toUpperCase();
  }
  return teile;
}

/** `2TU` → { n: 2, tag: 2 }, `FR` → { n: 0, tag: 5 }. */
function wochentag(text) {
  const m = /^([+-]?\d{1,2})?([A-Z]{2})$/.exec(text);
  if (!m) return null;
  const tag = TAGE.indexOf(m[2]);
  return tag < 0 ? null : { n: m[1] ? parseInt(m[1], 10) : 0, tag };
}

function zahlen(text) {
  return (text || '').split(',').map((z) => parseInt(z, 10)).filter((z) => Number.isFinite(z) && z !== 0);
}

/** Die Wanduhrzeit als `YYYYMMDDTHHMMSS` -- die Schreibweise von `exdates`. */
export function kompakt(d) {
  const p = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}T${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

function tageImMonat(jahr, monat) {
  return new Date(jahr, monat + 1, 0).getDate();
}

// Die Tage eines Monats nach BYDAY (mit oder ohne Ordnungszahl, die dann im
// Monat zählt).
function tageNachWochentag(jahr, monat, byday) {
  const letzter = tageImMonat(jahr, monat);
  const aus = [];
  for (const w of byday) {
    const passend = [];
    for (let d = 1; d <= letzter; d++) {
      if (new Date(jahr, monat, d).getDay() === w.tag) passend.push(d);
    }
    if (w.n === 0) aus.push(...passend);
    else {
      const d = w.n > 0 ? passend[w.n - 1] : passend[passend.length + w.n];
      if (d) aus.push(d);
    }
  }
  return aus;
}

function monatstage(jahr, monat, bymonthday) {
  const letzter = tageImMonat(jahr, monat);
  return bymonthday
    .map((z) => (z > 0 ? z : letzter + z + 1))
    .filter((d) => d >= 1 && d <= letzter);
}

// Die Kandidaten eines Zeitraums (ein Tag, eine Woche, ein Monat, ein Jahr)
// als Datumsangaben ohne Uhrzeit, aufsteigend.
function kandidaten(freq, periode, start, r) {
  const { byday, bymonthday, bymonth } = r;
  let tage = [];
  if (freq === 'DAILY') {
    tage = [periode];
  } else if (freq === 'WEEKLY') {
    if (byday.length) {
      for (let i = 0; i < 7; i++) {
        const d = new Date(periode.getFullYear(), periode.getMonth(), periode.getDate() + i);
        if (byday.some((w) => w.tag === d.getDay())) tage.push(d);
      }
    } else {
      const d = new Date(periode);
      d.setDate(d.getDate() + ((start.getDay() - r.wkst + 7) % 7));
      tage = [d];
    }
  } else if (freq === 'MONTHLY') {
    tage = tageImJahrMonat(periode.getFullYear(), periode.getMonth(), start, r);
  } else if (freq === 'YEARLY') {
    const monate = bymonth.length ? bymonth.map((m) => m - 1) : [start.getMonth()];
    if (!bymonth.length && byday.length && !bymonthday.length) {
      // BYDAY ohne BYMONTH: die Ordnungszahl zählt im Jahr.
      const jahr = periode.getFullYear();
      for (const w of byday) {
        const passend = [];
        for (let d = new Date(jahr, 0, 1); d.getFullYear() === jahr; d.setDate(d.getDate() + 1)) {
          if (d.getDay() === w.tag) passend.push(new Date(d));
        }
        if (w.n === 0) tage.push(...passend);
        else {
          const d = w.n > 0 ? passend[w.n - 1] : passend[passend.length + w.n];
          if (d) tage.push(d);
        }
      }
    } else {
      for (const m of monate) tage.push(...tageImJahrMonat(periode.getFullYear(), m, start, r));
    }
  }
  // Begrenzende Teile.
  if (bymonth.length) tage = tage.filter((d) => bymonth.includes(d.getMonth() + 1));
  if (freq === 'DAILY' || freq === 'WEEKLY') {
    if (bymonthday.length) {
      tage = tage.filter((d) => monatstage(d.getFullYear(), d.getMonth(), bymonthday).includes(d.getDate()));
    }
    if (freq === 'DAILY' && byday.length) tage = tage.filter((d) => byday.some((w) => w.tag === d.getDay()));
  }
  tage = [...new Map(tage.map((d) => [d.getTime(), d])).values()].sort((a, b) => a - b);
  if (r.bysetpos.length) {
    tage = r.bysetpos
      .map((p) => (p > 0 ? tage[p - 1] : tage[tage.length + p]))
      .filter(Boolean)
      .sort((a, b) => a - b);
  }
  return tage;
}

function tageImJahrMonat(jahr, monat, start, r) {
  let tage;
  if (r.bymonthday.length) {
    tage = monatstage(jahr, monat, r.bymonthday);
    if (r.byday.length) {
      const erlaubt = tageNachWochentag(jahr, monat, r.byday);
      tage = tage.filter((d) => erlaubt.includes(d));
    }
  } else if (r.byday.length) {
    tage = tageNachWochentag(jahr, monat, r.byday);
  } else {
    // Ohne Angabe der Tag des Beginns; einen 31. gibt es nicht jeden Monat.
    tage = start.getDate() <= tageImMonat(jahr, monat) ? [start.getDate()] : [];
  }
  return tage.map((d) => new Date(jahr, monat, d));
}

// Der Beginn des Zeitraums, in dem `d` liegt.
function periodeVon(freq, d, wkst) {
  if (freq === 'WEEKLY') {
    const p = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    p.setDate(p.getDate() - ((d.getDay() - wkst + 7) % 7));
    return p;
  }
  if (freq === 'MONTHLY') return new Date(d.getFullYear(), d.getMonth(), 1);
  if (freq === 'YEARLY') return new Date(d.getFullYear(), 0, 1);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

function weiter(freq, p, schritt) {
  const n = new Date(p);
  if (freq === 'DAILY') n.setDate(n.getDate() + schritt);
  else if (freq === 'WEEKLY') n.setDate(n.getDate() + 7 * schritt);
  else if (freq === 'MONTHLY') n.setMonth(n.getMonth() + schritt);
  else n.setFullYear(n.getFullYear() + schritt);
  return n;
}

/**
 * Die Beginne einer Serie (Date), bis `bisTag` (YYYY-MM-DD, einschließlich).
 * COUNT zählt wie in RFC 5545 vor dem Auslassen der Ausnahmen.
 */
export function vorkommen(ev, bisTag) {
  const freq = ev.rrule;
  const teile = regelTeile(ev.rrule_parts);
  const r = {
    byday: (teile.BYDAY || '').split(',').map(wochentag).filter(Boolean),
    bymonthday: zahlen(teile.BYMONTHDAY),
    bymonth: zahlen(teile.BYMONTH),
    bysetpos: zahlen(teile.BYSETPOS),
    wkst: Math.max(0, TAGE.indexOf(teile.WKST || 'MO')),
  };
  const intervall = Math.max(1, parseInt(teile.INTERVAL, 10) || 1);
  const anzahl = parseInt(teile.COUNT, 10) || 0;
  const start = new Date(ev.start_date);
  const ende = ev.rrule_until ? getLocalDateString(new Date(ev.rrule_until)) : null;
  const aus = [start];
  let periode = periodeVon(freq, start, r.wkst);
  for (let runde = 0; runde < 5000 && aus.length < GRENZE; runde++) {
    if (getLocalDateString(periode) > bisTag) break;
    for (const tag of kandidaten(freq, periode, start, r)) {
      const d = new Date(tag.getFullYear(), tag.getMonth(), tag.getDate(), start.getHours(), start.getMinutes(), start.getSeconds());
      if (d <= start) continue;
      const tagText = getLocalDateString(d);
      if (tagText > bisTag || (ende && tagText > ende) || (anzahl && aus.length >= anzahl) || aus.length >= GRENZE) {
        return aus;
      }
      aus.push(d);
    }
    periode = weiter(freq, periode, intervall);
  }
  return aus;
}

// Der Tag einer Ausnahme (`YYYYMMDD`) in der Zone des Geräts. Aus ICS kommt
// sie als Wanduhrzeit, aus dem Kalender des Telefons auch in UTC (`…Z`).
// Eine Serie hat höchstens ein Vorkommen am Tag -- der Tag genügt.
function ausnahmeTag(ex) {
  const m = /^(\d{4})(\d{2})(\d{2})(?:T(\d{2})(\d{2})(\d{2})(Z)?)?$/.exec(ex.trim());
  if (!m) return null;
  if (!m[7]) return `${m[1]}${m[2]}${m[3]}`;
  const d = new Date(Date.UTC(+m[1], +m[2] - 1, +m[3], +m[4], +m[5], +m[6]));
  return kompakt(d).split('T')[0];
}

export function expandRecurringEvents(events) {
  const expanded = [];
  const limitDate = new Date();
  limitDate.setFullYear(limitDate.getFullYear() + 2);
  const limitStr = getLocalDateString(limitDate);

  // Verschobene Vorkommen je Serie: Ihr alter Platz bleibt frei.
  const verschoben = new Map();
  for (const ev of events) {
    if (ev.series && ev.occurrence) {
      if (!verschoben.has(ev.series)) verschoben.set(ev.series, new Set());
      verschoben.get(ev.series).add(ev.occurrence);
    }
  }

  for (const ev of events) {
    if (!ev.rrule || ev.rrule === 'NONE') {
      expanded.push(ev);
      continue;
    }

    const ausgelassen = new Set(
      [...(ev.exdates ? ev.exdates.split(',') : []), ...(verschoben.get(ev.id) || [])]
        .map(ausnahmeTag)
        .filter(Boolean),
    );
    const dauer = new Date(ev.end_date) - new Date(ev.start_date);

    vorkommen(ev, limitStr).forEach((beginn, i) => {
      if (ausgelassen.has(kompakt(beginn).split('T')[0])) return;
      expanded.push({
        ...ev,
        id: ev.id + '-' + (i + 1),
        real_id: ev.id,
        original_event: ev,
        start_date: beginn.toISOString(),
        end_date: new Date(beginn.getTime() + dauer).toISOString(),
      });
    });
  }
  return expanded;
}
