/*
 * Eine Abstimmung aus ihren entschlüsselten Einträgen bauen
 * (docs/plan-abstimmungen-e2e.md, 5b).
 *
 * Der Server liest nichts mehr; er verteilt nur Einträge mit laufender
 * Nummer. Was sie bedeuten und welche Regeln gelten, steht HIER -- für alle
 * Geräte gleich, damit alle dasselbe zählen:
 *
 *   kopf      { art, typ, titel, beschreibung, einstellungen, bis, geschlossen, final }
 *             nur von der Person, die die Abstimmung angelegt hat
 *   option    { art, label, beginn, ende, kapazitaet, person, position }
 *             von der Erstellerin; bei Listen mit „Mitglieder dürfen
 *             ergänzen" auch von Mitgliedern
 *   stimme    { art, option, antwort, kommentar }   je Person und Option eine
 *   gast      { art, option, status }  Zu-/Absage für Gäste ohne Konto
 *             (nur die Erstellerin)
 *   erledigt  { art, option, erledigt }  Abhaken (Erstellerin, Eigentümer,
 *             wer sich eingetragen hat)
 *
 * Regeln:
 *   - Kapazität: Es zählen die ersten N Eintragungen nach der Nummer des
 *     Servers; wer zu spät kam, zählt nicht.
 *   - Eine Stimme (Umfrage ohne Mehrfachwahl): die zuerst abgegebene.
 *   - Einladung: Zu-/Absage zählt nur von der eingeladenen Person.
 *   - Abgelaufen oder geschlossen: Danach Geschriebenes zählt nicht.
 *
 * Heraus kommt die Form, die die Ansichten (polls/*.vue) seit jeher kennen
 * -- früher baute sie der Server (PollResource). Ids sind uuids; die eigene
 * Person trägt `meId`.
 */

const zeitVon = (wert) => (wert ? new Date(wert).getTime() : null);

function letzter(liste) {
  return liste.reduce((a, b) => (a === null || b.nr > a.nr ? b : a), null);
}

/**
 * @param {object} roh      { uuid, created_by, created_at, ich, mitglieder, eintraege }
 * @param {*}      meId     Kennung der eigenen Person in den Ansichten
 * @param {Date}   jetzt
 * @returns {object|null}   null, wenn der Kopf (noch) nicht lesbar ist
 */
export function abstimmungBauen(roh, meId = null, jetzt = new Date()) {
  const ich = roh.ich ?? null;
  const uid = (u) => (u !== null && u !== undefined && u === ich ? meId : u);
  const mitglieder = roh.mitglieder ?? [];
  const mitgliedIds = new Set(mitglieder.map((m) => m.user_id));
  const eigentuemer = new Set(mitglieder.filter((m) => m.rolle === 'owner').map((m) => m.user_id));
  const namen = Object.fromEntries(mitglieder.map((m) => [m.user_id, m.name]));
  const eintraege = [...(roh.eintraege ?? [])].sort((a, b) => a.nr - b.nr);
  const art = (a) => eintraege.filter((e) => e.inhalt?.art === a);

  const kopfEintrag = letzter(art('kopf').filter((e) => e.von === roh.created_by));
  if (!kopfEintrag) return null;
  const kopf = kopfEintrag.inhalt;
  const typ = kopf.typ;
  const einstellungen = kopf.einstellungen ?? {};

  // Ab wann nichts mehr zählt.
  const schluss = [
    kopf.geschlossen ? zeitVon(kopfEintrag.geaendert ?? kopfEintrag.zeit) : null,
    kopf.bis && zeitVon(kopf.bis) <= jetzt.getTime() ? zeitVon(kopf.bis) : null,
  ].filter((z) => z !== null);
  const geschlossenAb = schluss.length ? Math.min(...schluss) : null;
  const zaehlt = (e) => geschlossenAb === null || zeitVon(e.geaendert ?? e.zeit) <= geschlossenAb;

  // Optionen.
  const darfOption = (e) => e.von === roh.created_by
    || (typ === 'signup' && einstellungen.member_options && mitgliedIds.has(e.von));
  const optionen = art('option').filter(darfOption)
    .map((e) => ({ e, o: e.inhalt }))
    .sort((a, b) => (a.o.position ?? 0) - (b.o.position ?? 0) || a.e.nr - b.e.nr);
  const optionIds = new Set(optionen.map(({ e }) => e.uuid));

  // Stimmen: gültige Option, rechtzeitig, je Person und Option die letzte.
  const stimmenRoh = art('stimme').filter((e) => optionIds.has(e.inhalt.option) && zaehlt(e));
  const jePaar = new Map();
  for (const e of stimmenRoh) jePaar.set(`${e.inhalt.option}|${e.von}`, e);
  let stimmen = [...jePaar.values()].sort((a, b) => a.nr - b.nr);

  if (typ === 'survey' && !einstellungen.multiple) {
    const gesehen = new Set();
    stimmen = stimmen.filter((e) => (gesehen.has(e.von) ? false : (gesehen.add(e.von), true)));
  }

  const basis = {
    id: roh.uuid,
    type: typ,
    title: kopf.titel ?? '',
    description: kopf.beschreibung ?? null,
    status: geschlossenAb !== null ? 'closed' : 'open',
    created_by: uid(roh.created_by),
    final_option_id: kopf.final && optionIds.has(kopf.final) ? kopf.final : null,
    settings: einstellungen,
    expires_at: kopf.bis ?? null,
    created_at: roh.created_at ?? null,
    options_count: optionen.length,
  };

  const mitgliederAnzeige = (abgestimmt) => mitglieder.map((m) => ({
    user_id: uid(m.user_id),
    name: m.name,
    voted: abgestimmt.has(m.user_id),
  }));

  if (typ === 'invite') {
    const optionenAnzeige = optionen.map(({ e, o }) => {
      let status;
      if (o.person !== null && o.person !== undefined) {
        const antwort = letzter(stimmen.filter((s) => s.inhalt.option === e.uuid && s.von === o.person));
        status = antwort ? antwort.inhalt.antwort : 'invited';
      } else {
        const gast = letzter(art('gast').filter((g) => g.inhalt.option === e.uuid && g.von === roh.created_by && zaehlt(g)));
        status = gast ? gast.inhalt.status : 'pending';
      }
      return { id: e.uuid, eigen: o.person === ich && ich !== null, label: o.label ?? '', user_id: o.person === null || o.person === undefined ? null : uid(o.person), invite_status: status, position: o.position ?? 0 };
    });
    const zahl = (s) => optionenAnzeige.filter((o) => o.invite_status === s).length;
    return {
      ...basis,
      options: optionenAnzeige,
      counts: { total: optionenAnzeige.length, yes: zahl('yes'), no: zahl('no'), invited: zahl('invited') },
    };
  }

  if (typ === 'schedule') {
    let beste = null;
    let besteJa = 0;
    const optionenAnzeige = optionen.map(({ e, o }) => {
      const hier = stimmen.filter((s) => s.inhalt.option === e.uuid);
      const tally = {
        yes: hier.filter((s) => s.inhalt.antwort === 'yes').length,
        no: hier.filter((s) => s.inhalt.antwort === 'no').length,
        maybe: hier.filter((s) => s.inhalt.antwort === 'maybe').length,
      };
      if (tally.yes > besteJa) { besteJa = tally.yes; beste = e.uuid; }
      return { id: e.uuid, starts_at: o.beginn ?? null, ends_at: o.ende ?? null, position: o.position ?? 0, tally };
    });
    return {
      ...basis,
      best_option_id: beste,
      options: optionenAnzeige,
      members: mitgliederAnzeige(new Set(stimmen.map((s) => s.von))),
      votes: stimmen.map((s) => ({ option_id: s.inhalt.option, user_id: uid(s.von), answer: s.inhalt.antwort })),
    };
  }

  if (typ === 'survey') {
    const abgestimmt = new Set(stimmen.map((s) => s.von));
    const anzeige = {
      ...basis,
      options: optionen.map(({ e, o }) => ({
        id: e.uuid,
        label: o.label ?? '',
        position: o.position ?? 0,
        count: stimmen.filter((s) => s.inhalt.option === e.uuid).length,
      })),
      my_option_ids: stimmen.filter((s) => s.von === ich).map((s) => s.inhalt.option),
      voted_count: abgestimmt.size,
      members_count: mitglieder.length,
    };
    if (!einstellungen.anonymous) {
      anzeige.members = mitgliederAnzeige(abgestimmt);
      anzeige.votes = stimmen.map((s) => ({ option_id: s.inhalt.option, user_id: uid(s.von) }));
    }
    return anzeige;
  }

  // signup: Schichtplan / Mitbringliste
  return {
    ...basis,
    options: optionen.map(({ e, o }) => {
      const kapazitaet = o.kapazitaet ?? null;
      const alle = stimmen.filter((s) => s.inhalt.option === e.uuid);
      const gezaehlt = kapazitaet === null ? alle : alle.slice(0, kapazitaet);
      const eingetragen = new Set(gezaehlt.map((s) => s.von));
      const haken = letzter(art('erledigt').filter((h) => h.inhalt.option === e.uuid
        && (h.von === roh.created_by || eigentuemer.has(h.von) || eingetragen.has(h.von))));
      return {
        id: e.uuid,
        label: o.label ?? '',
        capacity: kapazitaet,
        done: Boolean(haken?.inhalt.erledigt),
        created_by: uid(e.von),
        position: o.position ?? 0,
        claims: gezaehlt.map((s) => ({
          user_id: uid(s.von),
          name: namen[s.von] ?? s.von_name ?? '–',
          comment: s.inhalt.kommentar ?? null,
        })),
      };
    }),
  };
}
