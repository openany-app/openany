/*
 * Das `api` der Projekt-Ansichten in der App (seit 01.10.2026).
 *
 * Die Ansichten kommen aus der Webapp (packages/oberflaeche/projekte) und
 * rufen dort Methoden wie `api.getBoard(projekt, board)` auf. Hier stehen
 * dieselben Methoden -- beantwortet aus dem LOKALEN Speicher, in genau der
 * Form, die der Server liefern würde (`{ data }` wie bei axios). Geschrieben
 * wird über die allgemeinen Projektsachen-Befehle; sie merken jede Änderung,
 * und der Abgleich bringt sie zu openany.de (Server-Projekte) bzw. zu den
 * Mitgliedern vor Ort (lokale Projekte).
 *
 * Kennungen sind hier uuids statt Server-Nummern -- die Ansichten vergleichen
 * sie nur miteinander, das reicht.
 *
 * Was nur mit Server geht (KI, Paket, Plan-Abo, Kontosuche), steht hier
 * nicht; eine Ansicht, die danach fragt, bekommt einen Fehler.
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';
import { i18n } from '@oberflaeche/i18n';

const sachen = (projekt, art, eltern = null) => invoke('projekt_sachen', { projekt, art, eltern });
const sache = (projekt, uuid) => invoke('projekt_sache', { projekt, uuid });
const anlegen = (projekt, art, eltern, felder) => invoke('projekt_sache_anlegen', { projekt, art, eltern, felder });
const aendern = (projekt, uuid, felder) => invoke('projekt_sache_aendern', { projekt, uuid, felder });
const umhaengen = (projekt, uuid, felder, eltern) => invoke('projekt_sache_aendern', { projekt, uuid, felder, eltern, elternSetzen: true });
const entfernen = (projekt, uuid) => invoke('projekt_sache_entfernen', { projekt, uuid });

const antwort = (data) => ({ data });
const { t } = i18n.global;
const nurWebapp = (was) => { throw new Error(t('app.fehler.nurWebapp', { was: t(`app.fehler.was.${was}`) })); };
const nachPosition = (a, b) => (a.felder.position ?? 0) - (b.felder.position ?? 0);

/** Ein Ort, wie ihn eine Karte zeigt (PlacePinResource). */
async function ortVon(projekt, uuid) {
  if (!uuid) return null;
  const o = await sache(projekt, uuid);
  return o ? { id: o.uuid, name: o.felder.name ?? '', lat: o.felder.lat ?? null, lng: o.felder.lng ?? null } : null;
}

/** Eine Karte wie KanbanCardResource. */
async function karte(projekt, k) {
  const ort = k.felder.place ?? null;
  return {
    id: k.uuid,
    column_id: k.eltern,
    title: k.felder.title ?? '',
    description: k.felder.description ?? '',
    position: k.felder.position ?? 0,
    created_by: null,
    // Die Zuweisung zeigt auf ein Konto -- sie reist nicht.
    assigned_to: null,
    assignee_name: null,
    due_date: k.felder.due_date || null,
    place_id: ort,
    place: await ortVon(projekt, ort),
    subject_id: k.felder.subject ?? null,
    subject: await fachVon(projekt, k.felder.subject),
  };
}

/** Die Karten einer Spalte neu durchzählen -- `karteUuid` an `index`. */
async function einreihen(projekt, spalte, karteUuid, index) {
  const andere = (await sachen(projekt, 'card', spalte))
    .filter((k) => k.uuid !== karteUuid)
    .sort(nachPosition);
  const reihe = [...andere];
  reihe.splice(Math.max(0, Math.min(index, reihe.length)), 0, { uuid: karteUuid });
  for (const [i, k] of reihe.entries()) {
    if (k.uuid === karteUuid) await umhaengen(projekt, k.uuid, { position: i }, spalte);
    else if (k.felder.position !== i) await aendern(projekt, k.uuid, { position: i });
  }
}

/** Ein Meilenstein wie MilestoneResource. */
async function meilenstein(projekt, m) {
  const o = m.felder.place ?? null;
  return {
    id: m.uuid,
    title: m.felder.title ?? '',
    description: m.felder.description ?? '',
    due_date: m.felder.due_date || null,
    status: m.felder.status || 'open',
    place_id: o,
    place: await ortVon(projekt, o),
    subject_id: m.felder.subject ?? null,
    subject: await fachVon(projekt, m.felder.subject),
    created_by: null,
    created_at: m.geaendert_at,
  };
}

/** Ein Ort wie PlaceResource. */
const ort = (o) => ({
  id: o.uuid,
  name: o.felder.name ?? '',
  note: o.felder.note ?? '',
  lat: Number(o.felder.lat ?? 0),
  lng: Number(o.felder.lng ?? 0),
  created_by: null,
  created_at: o.geaendert_at,
});

const STANDARD_SPALTEN = ['Offen', 'In Arbeit', 'Fertig'];

export const projektApi = {
  /* ── Kanban (KanbanBoardController, KanbanColumnController, KanbanCardController) ── */

  async getBoards(projekt) {
    const boards = await sachen(projekt, 'board');
    const liste = [];
    for (const b of boards) {
      if (b.felder.papierkorb_at) continue;
      let karten = 0;
      for (const s of await sachen(projekt, 'column', b.uuid)) karten += (await sachen(projekt, 'card', s.uuid)).length;
      liste.push({ id: b.uuid, name: b.felder.name ?? '', created_by: null, cards_count: karten, created_at: b.geaendert_at });
    }
    return antwort(liste);
  },

  async createBoard(projekt, name) {
    const board = await anlegen(projekt, 'board', null, { name });
    for (const [i, n] of STANDARD_SPALTEN.entries()) await anlegen(projekt, 'column', board, { name: n, position: i });
    return antwort({ id: board, name, created_by: null, columns: [] });
  },

  async getBoard(projekt, board) {
    const b = await sache(projekt, board);
    if (!b) throw new Error(t('app.fehler.gibtEsNicht.board'));
    const spalten = (await sachen(projekt, 'column', board)).sort(nachPosition);
    const columns = [];
    for (const s of spalten) {
      const cards = [];
      for (const k of (await sachen(projekt, 'card', s.uuid)).sort(nachPosition)) cards.push(await karte(projekt, k));
      columns.push({ id: s.uuid, board_id: board, name: s.felder.name ?? '', position: s.felder.position ?? 0, cards });
    }
    return antwort({ id: board, name: b.felder.name ?? '', created_by: null, columns });
  },

  async deleteBoard(projekt, board) {
    await entfernen(projekt, board);
    return antwort({});
  },

  async createColumn(projekt, board, name) {
    const spalten = await sachen(projekt, 'column', board);
    const position = spalten.reduce((m, s) => Math.max(m, (s.felder.position ?? 0) + 1), 0);
    const id = await anlegen(projekt, 'column', board, { name, position });
    return antwort({ id, board_id: board, name, position, cards: [] });
  },

  async renameColumn(projekt, board, spalte, name) {
    await aendern(projekt, spalte, { name });
    return antwort({ id: spalte, name });
  },

  // Wie drüben: Die Karten wandern in die Zielspalte (ans Ende), sonst gehen
  // sie mit der Spalte.
  async deleteColumn(projekt, board, spalte, ziel = null) {
    if (ziel) {
      const dort = (await sachen(projekt, 'card', ziel)).length;
      const karten = (await sachen(projekt, 'card', spalte)).sort(nachPosition);
      for (const [i, k] of karten.entries()) await umhaengen(projekt, k.uuid, { position: dort + i }, ziel);
    }
    await entfernen(projekt, spalte);
    return antwort({});
  },

  async reorderColumns(projekt, board, ids) {
    for (const [i, id] of ids.entries()) await aendern(projekt, id, { position: i });
    return antwort({});
  },

  async createCard(projekt, board, spalte, title, extra = {}) {
    const karten = await sachen(projekt, 'card', spalte);
    const position = karten.reduce((m, k) => Math.max(m, (k.felder.position ?? 0) + 1), 0);
    const felder = { title, description: extra.description ?? '', position, due_date: extra.due_date ?? null };
    if (extra.place_id) felder.place = extra.place_id;
    if (extra.subject_id) felder.subject = extra.subject_id;
    const id = await anlegen(projekt, 'card', spalte, felder);
    return antwort(await karte(projekt, { uuid: id, eltern: spalte, felder }));
  },

  async updateCard(projekt, board, kartenId, data) {
    const felder = {};
    for (const k of ['title', 'description', 'due_date']) if (k in data) felder[k] = data[k] ?? (k === 'due_date' ? null : '');
    if ('place_id' in data) felder.place = data.place_id ?? null;
    if ('subject_id' in data) felder.subject = data.subject_id ?? null;
    await aendern(projekt, kartenId, felder);
    const k = await sache(projekt, kartenId);
    return antwort(await karte(projekt, k));
  },

  async deleteCard(projekt, board, kartenId) {
    await entfernen(projekt, kartenId);
    return antwort({});
  },

  async moveCard(projekt, board, kartenId, spalte, position) {
    await einreihen(projekt, spalte, kartenId, position);
    return antwort({});
  },

  /* ── Roadmaps (RoadmapController, MilestoneController) ── */

  async getRoadmaps(projekt) {
    const liste = [];
    for (const r of await sachen(projekt, 'roadmap')) {
      if (r.felder.papierkorb_at) continue;
      liste.push({ id: r.uuid, name: r.felder.name ?? '', created_by: null, milestones_count: (await sachen(projekt, 'milestone', r.uuid)).length, created_at: r.geaendert_at });
    }
    return antwort(liste);
  },

  async createRoadmap(projekt, name) {
    const id = await anlegen(projekt, 'roadmap', null, { name });
    return antwort({ id, name, created_by: null, milestones: [] });
  },

  async deleteRoadmap(projekt, roadmap) {
    await entfernen(projekt, roadmap);
    return antwort({});
  },

  async getRoadmap(projekt, roadmap) {
    const r = await sache(projekt, roadmap);
    if (!r) throw new Error(t('app.fehler.gibtEsNicht.roadmap'));
    const milestones = [];
    for (const m of await sachen(projekt, 'milestone', roadmap)) milestones.push(await meilenstein(projekt, m));
    // Wie drüben: nach Datum.
    milestones.sort((a, b) => String(a.due_date ?? '').localeCompare(String(b.due_date ?? '')));
    return antwort({ id: roadmap, name: r.felder.name ?? '', created_by: null, milestones });
  },

  async createMilestone(projekt, roadmap, payload) {
    const felder = { title: payload.title ?? '', description: payload.description ?? '', due_date: payload.due_date ?? null, status: payload.status ?? 'open' };
    if (payload.place_id) felder.place = payload.place_id;
    if (payload.subject_id) felder.subject = payload.subject_id;
    const id = await anlegen(projekt, 'milestone', roadmap, felder);
    return antwort(await meilenstein(projekt, { uuid: id, eltern: roadmap, felder, geaendert_at: '' }));
  },

  async updateMilestone(projekt, roadmap, id, payload) {
    const felder = {};
    for (const k of ['title', 'description', 'due_date', 'status']) if (k in payload) felder[k] = payload[k];
    if ('place_id' in payload) felder.place = payload.place_id ?? null;
    if ('subject_id' in payload) felder.subject = payload.subject_id ?? null;
    await aendern(projekt, id, felder);
    return antwort(await meilenstein(projekt, await sache(projekt, id)));
  },

  async deleteMilestone(projekt, roadmap, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  /* ── Orte (PlaceGroupController, PlaceController) ── */

  async getPlaceGroups(projekt) {
    const liste = [];
    for (const g of await sachen(projekt, 'place_group')) {
      if (g.felder.papierkorb_at) continue;
      liste.push({ id: g.uuid, name: g.felder.name ?? '', created_by: null, places_count: (await sachen(projekt, 'place', g.uuid)).length, created_at: g.geaendert_at });
    }
    return antwort(liste);
  },

  async createPlaceGroup(projekt, name) {
    const id = await anlegen(projekt, 'place_group', null, { name });
    return antwort({ id, name, created_by: null, places: [] });
  },

  async deletePlaceGroup(projekt, gruppe) {
    await entfernen(projekt, gruppe);
    return antwort({});
  },

  async getPlaceGroup(projekt, gruppe) {
    const g = await sache(projekt, gruppe);
    if (!g) throw new Error(t('app.fehler.gibtEsNicht.sammlung'));
    const places = (await sachen(projekt, 'place', gruppe)).map(ort);
    return antwort({ id: gruppe, name: g.felder.name ?? '', created_by: null, places });
  },

  async createPlace(projekt, gruppe, payload) {
    const felder = { name: payload.name ?? '', note: payload.note ?? '', lat: payload.lat, lng: payload.lng };
    const id = await anlegen(projekt, 'place', gruppe, felder);
    return antwort(ort({ uuid: id, felder, geaendert_at: '' }));
  },

  async updatePlace(projekt, gruppe, id, payload) {
    const felder = {};
    for (const k of ['name', 'note', 'lat', 'lng']) if (k in payload) felder[k] = payload[k];
    await aendern(projekt, id, felder);
    return antwort(ort(await sache(projekt, id)));
  },

  async deletePlace(projekt, gruppe, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  /* ── Was die Karten-Auswahl braucht ── */

  // Alle Orte des Projekts, aus Sammlungen, die nicht im Papierkorb liegen.
  async getProjectPlaces(projekt) {
    const orte = [];
    for (const g of await sachen(projekt, 'place_group')) {
      if (g.felder.papierkorb_at) continue;
      for (const o of await sachen(projekt, 'place', g.uuid)) {
        orte.push({ id: o.uuid, place_group_id: g.uuid, name: o.felder.name ?? '', note: o.felder.note ?? '', lat: o.felder.lat ?? null, lng: o.felder.lng ?? null });
      }
    }
    return antwort(orte);
  },

  /* ── Fächer (SubjectController) ── */

  async getSubjects(projekt) {
    const faecher = (await sachen(projekt, 'subject')).sort((a, b) => nachPosition(a, b) || String(a.felder.name).localeCompare(String(b.felder.name), 'de'));
    const liste = [];
    for (const f of faecher) liste.push(await fach(projekt, f));
    return antwort(liste);
  },

  async createSubject(projekt, payload) {
    const position = (await sachen(projekt, 'subject')).reduce((m, f) => Math.max(m, (f.felder.position ?? 0) + 1), 0);
    const felder = { ...fachFelder(payload), position };
    const id = await anlegen(projekt, 'subject', null, felder);
    return antwort(await fach(projekt, { uuid: id, felder }));
  },

  async updateSubject(projekt, id, payload) {
    await aendern(projekt, id, fachFelder(payload));
    return antwort(await fach(projekt, await sache(projekt, id)));
  },

  async deleteSubject(projekt, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  // Ein Heft gehört einem Menschen, nicht dem Projekt -- das geht über ein Konto.
  createSubjectNoteFolder: () => nurWebapp('heftAnlegen'),

  /* ── Schuljahre (SchoolYearController, SchoolYearBreakController) ── */

  async getSchoolYears(projekt) {
    const liste = [];
    for (const j of await sachen(projekt, 'school_year')) {
      if (j.felder.papierkorb_at) continue;
      liste.push({ ...schuljahrKopf(j), breaks_count: (await sachen(projekt, 'school_year_break', j.uuid)).length, created_at: j.geaendert_at });
    }
    return antwort(liste);
  },

  async createSchoolYear(projekt, name, notify, extra = {}) {
    const felder = { name, starts_on: extra.starts_on ?? null, ends_on: extra.ends_on ?? null };
    const id = await anlegen(projekt, 'school_year', null, felder);
    return antwort({ ...schuljahrKopf({ uuid: id, felder }), breaks: [] });
  },

  async getSchoolYear(projekt, jahr) {
    const j = await sache(projekt, jahr);
    if (!j) throw new Error(t('app.fehler.gibtEsNicht.schuljahr'));
    const breaks = (await sachen(projekt, 'school_year_break', jahr))
      .map((b) => ferien(b, jahr))
      .sort((a, b) => String(a.starts_on).localeCompare(String(b.starts_on)));
    return antwort({ ...schuljahrKopf(j), breaks });
  },

  async updateSchoolYear(projekt, jahr, payload) {
    await aendern(projekt, jahr, nur(payload, ['name', 'starts_on', 'ends_on']));
    return projektApi.getSchoolYear(projekt, jahr);
  },

  async deleteSchoolYear(projekt, jahr) {
    await entfernen(projekt, jahr);
    return antwort({});
  },

  async createSchoolYearBreak(projekt, jahr, payload) {
    const felder = { kind: 'holiday', ...nur(payload, ['name', 'starts_on', 'ends_on', 'kind']) };
    const id = await anlegen(projekt, 'school_year_break', jahr, felder);
    return antwort(ferien({ uuid: id, felder }, jahr));
  },

  async updateSchoolYearBreak(projekt, jahr, id, payload) {
    await aendern(projekt, id, nur(payload, ['name', 'starts_on', 'ends_on', 'kind']));
    return antwort(ferien(await sache(projekt, id), jahr));
  },

  async deleteSchoolYearBreak(projekt, jahr, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  // Die Ferientermine der Länder liegen auf dem Server.
  async getSchoolHolidayStates() {
    return antwort([]);
  },
  importSchoolYearBreaks: () => nurWebapp('ferienImport'),
  importPresetBreaks: () => nurWebapp('ferienImport'),
  setUpSchool: () => nurWebapp('schulPaket'),

  /* ── Wochenpläne (WeekPlanController, …SlotController, …PeriodController) ── */

  async getWeekPlans(projekt) {
    const liste = [];
    for (const w of await sachen(projekt, 'week_plan')) {
      if (w.felder.papierkorb_at) continue;
      liste.push({ ...wochenplanKopf(w), slots_count: (await sachen(projekt, 'week_plan_slot', w.uuid)).length, created_at: w.geaendert_at });
    }
    return antwort(liste);
  },

  async createWeekPlan(projekt, name, notify, extra = {}) {
    const felder = {
      name,
      plan_type: extra.type ?? 'timetable',
      applies_to: extra.applies_to ?? 'all',
      school_year: extra.school_year_id ?? null,
      // Wie drüben: die Zone dessen, der ihn anlegt.
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'Europe/Berlin',
      settings: extra.settings ?? null,
      valid_from: null,
      valid_to: null,
      cycle_weeks: 1,
      cycle_mode: 'anchor',
      cycle_anchor: null,
    };
    const id = await anlegen(projekt, 'week_plan', null, felder);
    return projektApi.getWeekPlan(projekt, id);
  },

  async getWeekPlan(projekt, plan) {
    const w = await sache(projekt, plan);
    if (!w) throw new Error(t('app.fehler.gibtEsNicht.wochenplan'));
    const slots = [];
    for (const s of (await sachen(projekt, 'week_plan_slot', plan)).sort(nachPosition)) slots.push(await stunde(projekt, s));
    const periods = [];
    for (const p of await sachen(projekt, 'week_plan_period', plan)) periods.push(await zeitraum(projekt, p));
    periods.sort((a, b) => String(a.starts_on).localeCompare(String(b.starts_on)));
    return antwort({ ...wochenplanKopf(w), slots, periods });
  },

  async updateWeekPlan(projekt, plan, payload) {
    const felder = nur(payload, ['name', 'applies_to', 'timezone', 'valid_from', 'valid_to', 'cycle_weeks', 'cycle_mode', 'cycle_anchor', 'settings']);
    if ('type' in payload) felder.plan_type = payload.type;
    if ('school_year_id' in payload) felder.school_year = payload.school_year_id ?? null;
    await aendern(projekt, plan, felder);
    return projektApi.getWeekPlan(projekt, plan);
  },

  async deleteWeekPlan(projekt, plan) {
    await entfernen(projekt, plan);
    return antwort({});
  },

  async createWeekPlanSlot(projekt, plan, payload) {
    const felder = { week_index: 0, is_all_day: false, position: 0, ...stundenFelder(payload) };
    const id = await anlegen(projekt, 'week_plan_slot', plan, felder);
    return antwort(await stunde(projekt, { uuid: id, felder }));
  },

  async updateWeekPlanSlot(projekt, plan, id, payload) {
    await aendern(projekt, id, stundenFelder(payload));
    return antwort(await stunde(projekt, await sache(projekt, id)));
  },

  async deleteWeekPlanSlot(projekt, plan, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  async createWeekPlanPeriod(projekt, plan, payload) {
    const felder = zeitraumFelder(payload);
    const id = await anlegen(projekt, 'week_plan_period', plan, felder);
    return antwort(await zeitraum(projekt, { uuid: id, felder }));
  },

  async updateWeekPlanPeriod(projekt, plan, id, payload) {
    await aendern(projekt, id, zeitraumFelder(payload));
    return antwort(await zeitraum(projekt, await sache(projekt, id)));
  },

  async deleteWeekPlanPeriod(projekt, plan, id) {
    await entfernen(projekt, id);
    return antwort({});
  },

  /* ── Projekte (ProjectController) ── */

  async getProjects() {
    return antwort((await invoke('projekte_liste')).map((p) => ({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal })));
  },

  // Wie ProjectResource -- Mitglieder kennt die App nur bei lokalen
  // Projekten (unterschriebene Liste); die Kennung ist dort die Personen-Id.
  async getProject(projekt) {
    const p = (await invoke('projekte_liste')).find((x) => x.id === projekt);
    if (!p) throw new Error(t('app.fehler.gibtEsNicht.projekt'));
    let members = [];
    let contents = 0;
    if (p.lokal) {
      const m = await invoke('projekt_mitglieder', { projekt });
      members = (m.mitglieder ?? []).map((x) => ({ user_id: x.id, name: x.name, role: x.rolle, ich: x.ich }));
      const f = await invoke('projekt_lokale_freigaben', { projekt }).catch(() => null);
      contents = (f?.freigaben ?? []).reduce((n, x) => n + x.eintraege.length, 0);
    }
    const zaehlen = async (art) => (await sachen(projekt, art)).filter((x) => !x.felder.papierkorb_at).length;
    // Ungelesen im Chat vor Ort (bis wohin hier gelesen ist).
    const unread = p.lokal ? await invoke('projekt_chat_ungelesen', { projekt }).catch(() => 0) : 0;
    const planning = p.boards + p.roadmaps + p.ortsgruppen + p.abstimmungen + await zaehlen('school_year') + await zaehlen('week_plan');
    return antwort({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal, members, counts: { unread_messages: unread, planning, contents } });
  },

  updateProject: () => nurWebapp('umbenennen'),

  async deleteProject(projekt) {
    await invoke('projekt_lokal_loeschen', { projekt });
    return antwort({});
  },

  /* ── Abstimmungen (PollController, PollResource) ──
   *
   * Sie kommen mit dem Abgleich vom Server: Zähler je Option und die EIGENE
   * Antwort (Abstimmungsstand.php), keine fremden Stimmen. Die eigene Person
   * steht deshalb mit `user_id: null` darin -- dieselbe Kennung, die die
   * Ansichten als `meId` bekommen. Abgestimmt wird für das eigene Konto,
   * vorgemerkt und gleich abgeglichen; die Regeln prüft der Server.
   */

  async getPolls(projekt) {
    const liste = [];
    for (const p of await sachen(projekt, 'poll')) {
      if (p.felder.papierkorb_at) continue;
      liste.push({ ...umfrageKopf(p), options_count: (await sachen(projekt, 'poll_option', p.uuid)).length });
    }
    return antwort(liste);
  },

  async getPoll(projekt, umfrage) {
    const p = await sache(projekt, umfrage);
    if (!p) throw new Error(t('app.fehler.gibtEsNicht.abstimmung'));
    const optionen = (await sachen(projekt, 'poll_option', umfrage)).sort(nachPosition);
    return antwort(umfrageVoll(p, optionen));
  },

  async respondPoll(projekt, umfrage, payload) {
    const optionen = await sachen(projekt, 'poll_option', umfrage);
    const stimme = (option, wert) => invoke('projekt_abstimmen', { projekt, option, antwort: wert });
    if (payload.option_ids) {
      // Umfrage: die gewählten an, alle anderen aus.
      for (const o of optionen) {
        const soll = payload.option_ids.includes(o.uuid) ? 'yes' : null;
        if ((o.felder.my_answer ?? null) !== soll) await stimme(o.uuid, soll);
      }
    } else if (payload.answer) await stimme(payload.option_id, payload.answer);
    else if (payload.status) await stimme(payload.option_id, payload.status);
    else await stimme(payload.option_id, payload.action === 'unclaim' ? null : 'yes');
    await invoke('projekte_abgleichen').catch(() => {});
    return projektApi.getPoll(projekt, umfrage);
  },

  createPoll: () => nurWebapp('abstimmungAnlegen'),
  deletePoll: () => nurWebapp('abstimmungLoeschen'),
  closePoll: () => nurWebapp('abstimmungAbschliessen'),
  duplicatePoll: () => nurWebapp('abstimmungKopieren'),
  addPollOption: () => nurWebapp('eintraegeHinzufuegen'),
  removePollOption: () => nurWebapp('eintraegeEntfernen'),
  togglePollOptionDone: () => nurWebapp('abhaken'),
};

/** Der Kopf einer Abstimmung, wie ihn die Liste zeigt. */
function umfrageKopf(p) {
  const f = p.felder;
  return {
    id: p.uuid,
    // Die Vorlage reist als `kind` (Abgleichsarten.php).
    type: f.kind ?? 'survey',
    title: f.title ?? '',
    description: f.description ?? '',
    status: f.status ?? 'open',
    // Nicht null: Das ist hier die eigene Kennung (`meId`), und wer eine
    // Abstimmung angelegt hat, darf sie verwalten. Wer es war, weiß die App
    // nicht -- verwalten darf dann nur der Eigentümer.
    created_by: -1,
    final_option_id: f.final_option ?? null,
    settings: f.settings ?? {},
    expires_at: f.expires_at ?? null,
    created_at: f.created_at ?? p.geaendert_at,
  };
}

/** Eine Abstimmung wie PollResource -- je Vorlage. */
function umfrageVoll(p, optionen) {
  const kopf = umfrageKopf(p);
  const f = (o) => o.felder;
  const zahl = (o, k) => Number(f(o)[k] ?? 0);
  const meine = (o) => f(o).my_answer ?? null;

  if (kopf.type === 'invite') {
    const options = optionen.map((o) => ({ id: o.uuid, label: f(o).label ?? '', user_id: f(o).mine ? null : -1, invite_status: f(o).invite_status ?? 'pending', position: f(o).position ?? 0 }));
    const anzahl = (s) => options.filter((o) => o.invite_status === s).length;
    return { ...kopf, options, counts: { total: options.length, yes: anzahl('yes'), no: anzahl('no'), invited: anzahl('invited') } };
  }

  if (kopf.type === 'schedule') {
    let beste = null;
    let besteJa = 0;
    const options = optionen.map((o) => {
      const tally = { yes: zahl(o, 'yes'), no: zahl(o, 'no'), maybe: zahl(o, 'maybe') };
      if (tally.yes > besteJa) { besteJa = tally.yes; beste = o.uuid; }
      return { id: o.uuid, starts_at: f(o).starts_at ?? null, ends_at: f(o).ends_at ?? null, position: f(o).position ?? 0, tally };
    });
    const votes = optionen.filter((o) => meine(o)).map((o) => ({ option_id: o.uuid, user_id: null, answer: meine(o) }));
    return { ...kopf, best_option_id: beste, options, members: [], votes };
  }

  if (kopf.type === 'survey') {
    return {
      ...kopf,
      options: optionen.map((o) => ({ id: o.uuid, label: f(o).label ?? '', position: f(o).position ?? 0, count: zahl(o, 'count') })),
      my_option_ids: optionen.filter((o) => meine(o) === 'yes').map((o) => o.uuid),
      voted_count: Number(p.felder.voted ?? 0),
      members_count: 0,
    };
  }

  // signup: Wer sich eingetragen hat, weiß die App nicht -- nur wie viele.
  return {
    ...kopf,
    options: optionen.map((o) => {
      const ich = meine(o) === 'yes';
      const andere = Math.max(0, zahl(o, 'count') - (ich ? 1 : 0));
      const claims = Array.from({ length: andere }, (_, i) => ({ user_id: -(i + 1), name: 'jemand', comment: null }));
      if (ich) claims.unshift({ user_id: null, name: 'du', comment: null });
      return { id: o.uuid, label: f(o).label ?? '', capacity: f(o).capacity ?? null, done: Boolean(f(o).done), created_by: null, position: f(o).position ?? 0, claims };
    }),
  };
}

/** Nur die genannten Schlüssel, die auch da sind. */
function nur(payload, schluessel) {
  return Object.fromEntries(schluessel.filter((k) => k in payload).map((k) => [k, payload[k]]));
}

const leerZuNull = (w) => (w === '' || w === undefined ? null : w);

/** Ein Fach wie SubjectResource -- ohne Heft (das hängt an einem Konto). */
async function fach(projekt, f) {
  const o = f.felder.place ?? null;
  return {
    id: f.uuid,
    project_id: projekt,
    name: f.felder.name ?? '',
    short: f.felder.short ?? null,
    color: f.felder.color ?? null,
    teacher: f.felder.teacher ?? null,
    place_id: o,
    place: await ortVon(projekt, o),
    note_folder_id: null,
    note_folder: null,
    position: f.felder.position ?? 0,
    created_by: null,
  };
}

async function fachVon(projekt, uuid) {
  if (!uuid) return null;
  const f = await sache(projekt, uuid);
  return f ? fach(projekt, f) : null;
}

function fachFelder(payload) {
  const felder = {};
  if ('name' in payload) felder.name = payload.name ?? '';
  for (const k of ['short', 'color', 'teacher']) if (k in payload) felder[k] = leerZuNull(payload[k]);
  if ('place_id' in payload) felder.place = payload.place_id ?? null;
  return felder;
}

const schuljahrKopf = (j) => ({
  id: j.uuid,
  name: j.felder.name ?? '',
  starts_on: j.felder.starts_on ?? null,
  ends_on: j.felder.ends_on ?? null,
  created_by: null,
});

const ferien = (b, jahr) => ({
  id: b.uuid,
  school_year_id: jahr,
  name: b.felder.name ?? '',
  starts_on: b.felder.starts_on ?? null,
  ends_on: b.felder.ends_on ?? null,
  kind: b.felder.kind ?? 'holiday',
});

const wochenplanKopf = (w) => ({
  id: w.uuid,
  name: w.felder.name ?? '',
  // Auf der Leitung `plan_type` -- `type` gehört dort der Art des Eintrags.
  type: w.felder.plan_type ?? 'timetable',
  applies_to: w.felder.applies_to ?? 'all',
  school_year_id: w.felder.school_year ?? null,
  timezone: w.felder.timezone ?? 'Europe/Berlin',
  valid_from: w.felder.valid_from ?? null,
  valid_to: w.felder.valid_to ?? null,
  cycle_weeks: Number(w.felder.cycle_weeks ?? 1),
  cycle_mode: w.felder.cycle_mode ?? 'anchor',
  cycle_anchor: w.felder.cycle_anchor ?? null,
  settings: w.felder.settings ?? null,
  created_by: null,
});

// Drüben steht eine Uhrzeit als „08:00:00" in der Spalte; die Ansicht will „08:00".
const uhr = (w) => (w ? String(w).slice(0, 5) : null);

/** Eine Stunde wie WeekPlanSlotResource. */
async function stunde(projekt, s) {
  const f = s.felder;
  return {
    id: s.uuid,
    week_index: Number(f.week_index ?? 0),
    weekday: Number(f.weekday ?? 1),
    starts_at: uhr(f.starts_at),
    ends_at: uhr(f.ends_at),
    is_all_day: Boolean(f.is_all_day),
    subject_id: f.subject ?? null,
    subject: await fachVon(projekt, f.subject),
    title: f.title ?? null,
    subtitle: f.subtitle ?? null,
    place_id: f.place ?? null,
    place: await ortVon(projekt, f.place),
    color: f.color ?? null,
    // Zuweisungen zeigen auf Konten -- sie reisen nicht.
    assigned_to: null,
    position: f.position ?? 0,
    created_by: null,
  };
}

function stundenFelder(payload) {
  const felder = nur(payload, ['week_index', 'weekday', 'is_all_day', 'position']);
  for (const k of ['starts_at', 'ends_at', 'title', 'subtitle', 'color']) if (k in payload) felder[k] = leerZuNull(payload[k]);
  if ('subject_id' in payload) felder.subject = payload.subject_id ?? null;
  if ('place_id' in payload) felder.place = payload.place_id ?? null;
  return felder;
}

/** Ein Zeitraum wie WeekPlanPeriodResource. */
async function zeitraum(projekt, p) {
  const f = p.felder;
  return {
    id: p.uuid,
    starts_on: f.starts_on ?? null,
    ends_on: f.ends_on ?? null,
    title: f.title ?? '',
    assigned_to: null,
    place_id: f.place ?? null,
    place: await ortVon(projekt, f.place),
    color: f.color ?? null,
    note: f.note ?? '',
    created_by: null,
  };
}

function zeitraumFelder(payload) {
  const felder = nur(payload, ['starts_on', 'ends_on', 'title', 'note']);
  if ('color' in payload) felder.color = leerZuNull(payload.color);
  if ('place_id' in payload) felder.place = payload.place_id ?? null;
  return felder;
}
