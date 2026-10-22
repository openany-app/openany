/*
 * Das `api` der Projekt-Ansichten in der App (seit 01.10.2026).
 *
 * Die Ansichten kommen aus der Webapp (packages/oberflaeche/projekte) und
 * rufen dort Methoden wie `api.getPoll(projekt, umfrage)` auf. Hier stehen
 * dieselben Methoden -- beantwortet aus dem LOKALEN Speicher, in genau der
 * Form, die der Server liefern würde (`{ data }` wie bei axios). Geschrieben
 * wird über die allgemeinen Projektsachen-Befehle; sie merken jede Änderung,
 * und der Abgleich bringt sie zu openany.de (Server-Projekte) bzw. zu den
 * Mitgliedern vor Ort (lokale Projekte).
 *
 * Kennungen sind hier uuids statt Server-Nummern -- die Ansichten vergleichen
 * sie nur miteinander, das reicht.
 *
 * Was nur mit Server geht (KI, Paket, Kontosuche), steht hier nicht; eine
 * Ansicht, die danach fragt, bekommt einen Fehler.
 *
 * Kanban, Roadmaps, Orte und Schulplanung sind seit dem 16.10.2026
 * zurückgebaut (docs/plan-ende-zu-ende.md, Schritt 0).
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';
import { i18n } from '@oberflaeche/i18n';
import { abstimmungBauen } from '@oberflaeche/projekte/abstimmungBauen';
import * as whiteboardFreigaben from './whiteboardFreigaben';


const antwort = (data) => ({ data });
// Fehler in der Form, die die Ansichten aus der Webapp kennen
// (`err.response.data.message`): Der Satz des Servers kommt so wörtlich an.
const amServer = (aktion, projekt = null, nutzer = null, daten = null) => invoke('projekt_mitglieder_server', { aktion, projekt, nutzer, daten })
  .catch((e) => { const message = String(e); throw Object.assign(new Error(message), { response: { data: { message } } }); });

/*
 * MITGLIEDER EINES SERVER-PROJEKTS (docs/plan-ende-zu-ende.md, Schritt 1).
 *
 * Die eigene Kennung ist hier `null` -- wie bei den Abstimmungen, wo die
 * eigene Stimme `user_id: null` trägt und die Ansichten `meId = null` als
 * „ich" bekommen. Die echte Konto-Id (vom Server als `me_id`) merkt sich
 * diese Datei je Projekt und setzt sie wieder ein, wenn ein Befehl sie
 * braucht (Verlassen).
 */
const ichAmServer = new Map();
function serverProjekt(projekt, d) {
  if (d.me_id !== undefined) ichAmServer.set(projekt, d.me_id);
  const ich = (id) => (id === d.me_id ? null : id);
  return { ...d, id: projekt, lokal: false, members: (d.members ?? []).map((m) => ({ ...m, user_id: ich(m.user_id) })) };
}
const echteId = (projekt, id) => (id === null ? ichAmServer.get(projekt) : id);
const { t } = i18n.global;

// Eine Chat-Nachricht mit der eigenen Kennung `null` (wie bei Mitgliedern).
const alsMeine = (projekt, m) => (m?.user && m.user.id === ichAmServer.get(projekt) ? { ...m, user: { ...m.user, id: null } } : m);

/*
 * [[VERWEISE]] AUF DEM GERÄT (docs/plan-projektchat-e2e.md): Der Server liest
 * den Chat nicht mehr. Die verlinkbaren Ziele kennt er noch (Freigaben und
 * Notizen sind bis Schritt 6/7 im Klartext) -- daraus entsteht hier dieselbe
 * Karte „Linkziel (klein) => Ziel", die früher der Server mitschickte:
 * getippte Ziele (`art:id`) und Notiztitel. Eine Minute zwischengespeichert.
 */
const zieleCache = new Map();
async function verweisziele(projekt) {
  const da = zieleCache.get(projekt);
  if (da && Date.now() - da.zeit < 60_000) return da.ziele;
  const ziele = await invoke('projekt_server_verweisziele', { projekt }).catch(() => ({ items: [] }));
  zieleCache.set(projekt, { zeit: Date.now(), ziele });
  return ziele;
}
async function verweisKarte(projekt) {
  const karte = {};
  for (const z of (await verweisziele(projekt)).items ?? []) {
    const ziel = { ...z, label: z.title ?? z.name ?? '' };
    karte[`${z.kind}:${z.id}`.toLowerCase()] = ziel;
    if (z.kind === 'note' && z.title) karte[z.title.toLowerCase()] ??= ziel;
  }
  return karte;
}

export const projektApi = {
  /* ── Projekte (ProjectController) ── */

  async getProjects() {
    return antwort((await invoke('projekte_liste')).map((p) => ({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal })));
  },

  // Wie ProjectResource -- Mitglieder kennt die App nur bei lokalen
  // Projekten (unterschriebene Liste); die Kennung ist dort die Personen-Id.
  async getProject(projekt) {
    const p = (await invoke('projekte_liste')).find((x) => x.id === projekt);
    if (!p) throw new Error(t('app.fehler.gibtEsNicht.projekt'));
    // Ein Server-Projekt: Kopf und Mitglieder vom Server (die Zähler aus dem
    // Speicher, wie bisher).
    if (!p.lokal) {
      const d = await amServer('projekt', projekt).catch(() => null);
      if (d) {
        const unread = 0;
        // Den Namen kennt der Server nicht mehr (3c) -- er steht entschlüsselt im Speicher.
        return antwort({ ...serverProjekt(projekt, d), name: p.name, counts: { unread_messages: unread, planning: d.counts?.planning ?? 0, contents: d.counts?.contents ?? 0 } });
      }
    }
    let members = [];
    let contents = 0;
    if (p.lokal) {
      const m = await invoke('projekt_mitglieder', { projekt });
      members = (m.mitglieder ?? []).map((x) => ({ user_id: x.id, name: x.name, role: x.rolle, ich: x.ich }));
      const f = await invoke('projekt_lokale_freigaben', { projekt }).catch(() => null);
      contents = (f?.freigaben ?? []).reduce((n, x) => n + x.eintraege.length, 0);
    }
    // Ungelesen im Chat vor Ort (bis wohin hier gelesen ist).
    const unread = p.lokal ? await invoke('projekt_chat_ungelesen', { projekt }).catch(() => 0) : 0;
    const planning = p.abstimmungen;
    return antwort({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal, members, counts: { unread_messages: unread, planning, contents } });
  },

  // Nur Server-Projekte: „Mitglieder dürfen einladen" (und was der Server
  // sonst an PUT /projects/{id} zulässt).
  async updateProject(projekt, daten) {
    // Umbenennen: verschlüsselt (3c, projektchat.rs) -- nie im Klartext.
    if (daten.name !== undefined) {
      await invoke('projekt_server_umbenennen', { projekt, name: daten.name });
      const { name, ...rest } = daten;
      if (!Object.keys(rest).length) return projektApi.getProject(projekt);
      daten = rest;
    }
    const d = serverProjekt(projekt, await amServer('aendern', projekt, null, daten));
    const p = (await invoke('projekte_liste')).find((x) => x.id === projekt);
    return antwort({ ...d, name: p?.name ?? d.name });
  },

  async searchProjectUsers(q) {
    return antwort(await amServer('suche', null, null, { q }));
  },
  async addProjectMember(projekt, nutzer) {
    return antwort(await amServer('hinzufuegen', projekt, nutzer));
  },
  async createProjectMemberAccount(projekt, { name }) {
    return antwort(await amServer('konto', projekt, null, { name }));
  },
  async removeProjectMember(projekt, nutzer) {
    const d = await amServer('entfernen', projekt, echteId(projekt, nutzer));
    return antwort(d && d.members ? serverProjekt(projekt, d) : d);
  },
  async setProjectMemberRole(projekt, nutzer, role) {
    return antwort(serverProjekt(projekt, await amServer('rolle', projekt, echteId(projekt, nutzer), { role })));
  },

  /* ── Chat eines Server-Projekts (Ende-zu-Ende, src-tauri/src/projektchat.rs) ──
   *
   * Dieselbe Form wie ProjectMessageController; `body` entschlüsselt die
   * App. Die eigene Kennung ist `null` (siehe oben). Die [[Verweise]] löst
   * die App selbst auf -- aus den verlinkbaren Zielen, die auch die
   * Vorschlagsliste füllen.
   */
  async getProjectMessages(projekt, params = {}) {
    const d = await invoke('projekt_server_chat', { projekt, vor: params.before_id ?? null, nach: params.after_id ?? null });
    return antwort({ ...d, messages: (d.messages ?? []).map((m) => alsMeine(projekt, m)), pinned: (d.pinned ?? []).map((m) => alsMeine(projekt, m)), link_targets: await verweisKarte(projekt) });
  },
  async sendProjectMessage(projekt, body, notifyMembers = false) {
    const m = await invoke('projekt_server_chat_senden', { projekt, inhalt: body, benachrichtigen: Boolean(notifyMembers) });
    return antwort({ ...alsMeine(projekt, m), link_targets: await verweisKarte(projekt) });
  },
  async pinProjectMessage(projekt, nachricht, an) {
    const m = await invoke('projekt_server_chat_anheften', { projekt, nachricht, an });
    return antwort({ ...alsMeine(projekt, m), link_targets: await verweisKarte(projekt) });
  },
  async deleteProjectMessage(projekt, nachricht) {
    await invoke('projekt_server_chat_loeschen', { projekt, nachricht });
    return antwort(null);
  },
  async getProjectLinkTargets(projekt) {
    return antwort(await verweisziele(projekt));
  },

  /*
   * Geteilte Kalender (docs/plan-kalender-e2e.md, 4c): Die Termine liest und
   * schreibt die App mit dem Projektschlüssel. Nur für Projekte auf
   * openany.de -- bei lokalen antwortet der Server nicht, und der Abschnitt
   * bleibt weg.
   */
  async getProjectCalendars(projekt) {
    return antwort(await invoke('projekt_kalender', { projekt }));
  },
  async shareCalendar(projekt, kalender) {
    return antwort(await invoke('projekt_kalender_teilen', { projekt, kalender }));
  },
  async unshareCalendar(projekt, freigabe) {
    await invoke('projekt_kalender_beenden', { projekt, freigabe });
    return antwort(null);
  },
  async getOwnCalendars() {
    return antwort((await invoke('kalenderbuch_kalender')).filter((k) => !k.nur_lesen));
  },

  async deleteProject(projekt) {
    await invoke('projekt_lokal_loeschen', { projekt });
    return antwort({});
  },

  /* ── Abstimmungen Ende-zu-Ende (docs/plan-abstimmungen-e2e.md) ──
   *
   * Beim Server liegt eine Abstimmung als Liste verschlüsselter Einträge
   * (Kopf, Optionen, Stimmen, Erledigt). Die App öffnet sie
   * (projektabstimmung.rs) und baut daraus die Form der Ansichten
   * (abstimmungBauen.js) -- dort stehen auch die Regeln. Die eigene Person
   * ist `null`, wie bei Mitgliedern und Chat.
   *
   * Je Person und Option gibt es eine Stimme: Sie hat einen festen
   * `schluessel` („stimme|<option>"), daraus macht die App die uuid.
   */

  async getPolls(projekt) {
    // Ein lokales Projekt kennt der Server nicht -- dort gibt es keine.
    const d = await invoke('projekt_abstimmungen', { projekt }).catch(() => null);
    if (!d) return antwort([]);
    return antwort((d.polls ?? []).map((p) => abstimmungBauen({ ...p, ich: d.ich, mitglieder: d.mitglieder }, null)).filter(Boolean));
  },

  async getPoll(projekt, umfrage) {
    return antwort(await abstimmung(projekt, umfrage));
  },

  async createPoll(projekt, payload) {
    const optionen = payload.options ?? [];
    const eintraege = [
      { schluessel: 'kopf', inhalt: kopfInhalt({ typ: payload.type, titel: payload.title, beschreibung: payload.description ?? null, einstellungen: payload.settings ?? {}, bis: alsZeitpunkt(payload.expires_at) }) },
      ...optionen.map((o, i) => ({ inhalt: optionInhalt(o, i) })),
    ];
    const einladen = optionen.map((o) => echteId(projekt, o.user_id ?? undefined)).filter((id) => Number.isInteger(id));
    const umfrage = await invoke('projekt_abstimmung_anlegen', { projekt, eintraege, benachrichtigen: Boolean(payload.notify_members), einladen });
    return projektApi.getPoll(projekt, umfrage);
  },

  async respondPoll(projekt, umfrage, payload) {
    const p = await abstimmung(projekt, umfrage);
    const stimme = (option, antwortWert, kommentar = null) => schreiben(projekt, umfrage, { art: 'stimme', option, antwort: antwortWert, kommentar }, `stimme|${option}`);
    const zurueck = (option) => weg(projekt, umfrage, { schluessel: `stimme|${option}` });
    if (payload.option_ids) {
      // Umfrage: die gewählten an, alle anderen aus.
      const meine = p.my_option_ids ?? [];
      for (const o of p.options) {
        const soll = payload.option_ids.includes(o.id);
        if (soll && !meine.includes(o.id)) await stimme(o.id, 'yes');
        if (!soll && meine.includes(o.id)) await zurueck(o.id);
      }
    } else if (payload.answer) {
      await stimme(payload.option_id, payload.answer);
    } else if (payload.status) {
      // Einladung: für sich selbst sagt man zu oder ab; für Gäste ohne
      // Konto trägt die Organisation es ein.
      const o = p.options.find((x) => x.id === payload.option_id);
      if (o?.eigen) await stimme(payload.option_id, payload.status);
      else await schreiben(projekt, umfrage, { art: 'gast', option: payload.option_id, status: payload.status }, `gast|${payload.option_id}`);
    } else if (payload.action === 'unclaim') {
      await zurueck(payload.option_id);
    } else {
      await stimme(payload.option_id, 'yes', payload.comment ?? null);
    }
    return projektApi.getPoll(projekt, umfrage);
  },

  async closePoll(projekt, umfrage, payload = {}) {
    const p = await abstimmung(projekt, umfrage);
    // Den Kopf schreibt nur, wer die Abstimmung angelegt hat.
    if (p.created_by !== null) throw new Error(t('app.fehler.nurErsteller'));
    await schreiben(projekt, umfrage, kopfInhalt({ typ: p.type, titel: p.title, beschreibung: p.description, einstellungen: p.settings, bis: p.expires_at, geschlossen: true, final: payload.final_option_id ?? null }), 'kopf');
    if (payload.add_to_calendar && p.type === 'schedule') {
      const o = p.options.find((x) => x.id === payload.final_option_id);
      if (o?.starts_at) await inDenKalender(p.title, p.description, o);
    }
    return projektApi.getPoll(projekt, umfrage);
  },

  async duplicatePoll(projekt, umfrage) {
    const roh = await invoke('projekt_abstimmung', { projekt, umfrage });
    const p = abstimmungBauen(roh, null);
    if (!p) throw new Error(t('app.fehler.gibtEsNicht.abstimmung'));
    const optionen = (roh.eintraege ?? []).filter((e) => e.inhalt?.art === 'option' && p.options.some((o) => o.id === e.uuid));
    const eintraege = [
      { schluessel: 'kopf', inhalt: kopfInhalt({ typ: p.type, titel: `${p.title} (${t('app.abstimmung.kopie')})`, beschreibung: p.description, einstellungen: p.settings }) },
      ...optionen.map((e) => ({ inhalt: { ...e.inhalt } })),
    ];
    const neu = await invoke('projekt_abstimmung_anlegen', { projekt, eintraege, benachrichtigen: false, einladen: [] });
    return projektApi.getPoll(projekt, neu);
  },

  async deletePoll(projekt, umfrage) {
    await invoke('projekt_abstimmung_loeschen', { projekt, umfrage });
    return antwort(null);
  },

  async addPollOption(projekt, umfrage, daten) {
    const p = await abstimmung(projekt, umfrage);
    await schreiben(projekt, umfrage, optionInhalt({ label: daten.label, capacity: daten.capacity, user_id: echteId(projekt, daten.user_id ?? undefined) }, p.options.length));
    return projektApi.getPoll(projekt, umfrage);
  },

  async removePollOption(projekt, umfrage, option) {
    await weg(projekt, umfrage, { uuid: option });
    return projektApi.getPoll(projekt, umfrage);
  },

  async togglePollOptionDone(projekt, umfrage, option, erledigt) {
    await schreiben(projekt, umfrage, { art: 'erledigt', option, erledigt: Boolean(erledigt) }, `erledigt|${option}`);
    return projektApi.getPoll(projekt, umfrage);
  },

  /* ── Whiteboards (docs/plan-whiteboard.md) ──
   *
   * Kopf und Zeichnung liegen beim Server verschlüsselt; projektwhiteboard.rs
   * öffnet sie. Speichern darf nur, wer es angelegt hat (`eigen`), und nur
   * auf dem gelesenen `stand` -- sonst kommt `whiteboard-veraltet`.
   */

  async getWhiteboards(projekt) {
    // Ein lokales Projekt kennt der Server nicht -- dort gibt es keine.
    return antwort(await invoke('projekt_whiteboards', { projekt }).catch(() => []));
  },

  async getWhiteboard(projekt, whiteboard) {
    return antwort(await invoke('projekt_whiteboard', { projekt, whiteboard }));
  },

  async createWhiteboard(projekt, { titel }) {
    const whiteboard = await invoke('projekt_whiteboard_anlegen', { projekt, titel });
    return projektApi.getWhiteboard(projekt, whiteboard);
  },

  async saveWhiteboard(projekt, whiteboard, { titel, inhalt, stand }) {
    return antwort(await invoke('projekt_whiteboard_speichern', { projekt, whiteboard, titel, inhalt, stand }));
  },

  async deleteWhiteboard(projekt, whiteboard) {
    await invoke('projekt_whiteboard_loeschen', { projekt, whiteboard });
    return antwort(null);
  },

  // Einbinden (W2): geteilte Inhalte des Projekts (whiteboardFreigaben.js).
  async getWhiteboardFreigaben(projekt) {
    return antwort(await whiteboardFreigaben.freigaben(projekt));
  },
  async getWhiteboardFreigabe(projekt, freigabe, art) {
    return antwort(await whiteboardFreigaben.inhalt(projekt, freigabe, art));
  },
  async getWhiteboardBild(projekt, freigabe, uuid) {
    return antwort(await whiteboardFreigaben.bild(projekt, freigabe, uuid));
  },
};

/** Eine Abstimmung, geöffnet und gebaut. */
async function abstimmung(projekt, umfrage) {
  const p = abstimmungBauen(await invoke('projekt_abstimmung', { projekt, umfrage }), null);
  if (!p) throw new Error(t('app.fehler.gibtEsNicht.abstimmung'));
  return p;
}

const schreiben = (projekt, umfrage, inhalt, schluessel = null) => invoke('projekt_abstimmung_schreiben', { projekt, umfrage, uuid: null, schluessel, inhalt });
const weg = (projekt, umfrage, { uuid = null, schluessel = null }) => invoke('projekt_abstimmung_eintrag_loeschen', { projekt, umfrage, uuid, schluessel });

function kopfInhalt({ typ, titel, beschreibung = null, einstellungen = {}, bis = null, geschlossen = false, final = null }) {
  return { art: 'kopf', typ, titel: (titel ?? '').trim(), beschreibung, einstellungen: einstellungen ?? {}, bis, geschlossen, final };
}

function optionInhalt(o, position) {
  return {
    art: 'option',
    label: o.label ?? null,
    beginn: o.starts_at ?? null,
    ende: o.ends_at ?? null,
    kapazitaet: o.capacity === '' || o.capacity === undefined || o.capacity === null ? null : Number(o.capacity),
    person: Number.isInteger(o.user_id) ? o.user_id : null,
    position,
  };
}

// `datetime-local` meint Ortszeit -- als Zeitpunkt, damit jedes Gerät
// dasselbe „abgelaufen" rechnet.
const alsZeitpunkt = (wert) => (wert ? new Date(wert).toISOString() : null);

/** Terminfindung → eigener Kalender: der erste eigene, sonst ein neuer. */
async function inDenKalender(titel, beschreibung, o) {
  const eigene = (await invoke('kalenderbuch_kalender')).filter((k) => !k.sync_url && !k.nur_lesen);
  const kalender = eigene[0]?.id ?? (await invoke('kalenderbuch_kalender_anlegen', { name: t('app.abstimmung.meinKalender'), farbe: null, aboUrl: null })).id;
  await invoke('kalenderbuch_termin_speichern', { id: null, kalender, titel, beschreibung: beschreibung ?? null, ort: null, beginn: o.starts_at, ende: o.ends_at ?? o.starts_at, ganztags: false, rrule: 'NONE' });
}
