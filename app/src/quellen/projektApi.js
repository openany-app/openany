/*
 * Das `api` der Projekt-Ansichten in der App.
 *
 * Die Ansichten (packages/oberflaeche/projekte) rufen Methoden wie `api.getProject(projekt)` auf. Hier stehen
 * dieselben Methoden -- beantwortet aus dem LOKALEN Speicher, in genau der
 * Form, die der Server liefern würde (`{ data }` wie bei axios). Geschrieben
 * wird über die allgemeinen Projektsachen-Befehle; sie merken jede Änderung,
 * und der Abgleich bringt sie zu openany.de (Server-Projekte) bzw. zu den
 * Mitgliedern vor Ort (lokale Projekte).
 *
 * Kennungen sind hier uuids statt Server-Nummern -- die Ansichten vergleichen
 * sie nur miteinander, das reicht.
 *
 * Was nur mit Server geht (Kontosuche), steht hier nicht; eine Ansicht, die
 * danach fragt, bekommt einen Fehler.
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln: Der Test
 * src-tauri/tests/verdrahtung.rs liest genau diese Zeilen.
 */
import { invoke } from '@tauri-apps/api/core';
import { i18n } from '@oberflaeche/i18n';
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


/*
 * [[VERWEISE]] (docs/plan-projektchat-e2e.md): Der Server kennt keine
 * verlinkbaren Ziele (er sieht nur Chiffrat); die Liste ist leer, bis das
 * Gerät sie selbst bildet.
 */
const verweisziele = async () => ({ items: [] });
const verweisKarte = async () => ({});

/*
 * DER CHAT IN DER FORM DER ANSICHT (E4). ProjectChat rechnet mit Zahlen als
 * Kennung (negative für „wird gesendet") und mit `user.id === null` für die
 * eigene Person; die Nachrichten tragen uuids und Personennamen. Hier wird
 * übersetzt: jede uuid bekommt beim ersten Sehen eine feste Zahl, jede
 * andere Person eine eigene.
 */
const chatNummern = new Map();
const personNummern = new Map();
const chatNummer = (uuid) => {
  if (!chatNummern.has(uuid)) chatNummern.set(uuid, chatNummern.size + 1);
  return chatNummern.get(uuid);
};
const chatUuid = (nummer) => [...chatNummern].find(([, n]) => n === nummer)?.[0];
const personNummer = (name) => {
  if (!personNummern.has(name)) personNummern.set(name, personNummern.size + 1);
  return personNummern.get(name);
};
const chatNachricht = (z) => (z ? {
  id: chatNummer(z.id),
  body: z.text,
  created_at: z.at,
  pinned: Boolean(z.angeheftet),
  user: { id: z.ich ? null : personNummer(z.name), name: z.name },
} : null);
// Alles auf einmal (es liegt ja hier); „neuer als" nach der Zahl.
function chatSeite(zeilen, params) {
  const alle = zeilen.map(chatNachricht);
  const messages = params.before_id ? [] : params.after_id ? alle.filter((m) => m.id > params.after_id) : alle;
  return { messages, has_more: false, pinned: alle.filter((m) => m.pinned) };
}

export const projektApi = {
  /* ── Projekte (ProjectController) ── */

  async getProjects() {
    return antwort((await invoke('projekte_liste')).map((p) => ({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal })));
  },

  // Wie ProjectResource -- Mitglieder kennt die App nur bei lokalen
  // Projekten (unterschriebene Liste); die Kennung ist dort die Personen-Id.
  // Der Zähler am Freigaben-Reiter zählt die Einträge der Liste: Freigaben
  // und geteilte Kalender.
  async getProject(projekt) {
    const p = (await invoke('projekte_liste')).find((x) => x.id === projekt);
    if (!p) throw new Error(t('app.fehler.gibtEsNicht.projekt'));
    const kalender = (await invoke('projekt_kalender', { projekt }).catch(() => ({ kalender: [] }))).kalender?.length ?? 0;
    // Ein Server-Projekt: Kopf und Mitglieder vom Server (die Zähler aus dem
    // Speicher, wie bisher).
    if (!p.lokal) {
      const d = await amServer('projekt', projekt).catch(() => null);
      if (d) {
        // Ungelesen im Chat: was hier liegt (E4, ein Chat für alle).
        const unread = await invoke('projekt_chat_ungelesen', { projekt }).catch(() => 0);
        // Den Namen kennt der Server nicht mehr (3c) -- er steht entschlüsselt im Speicher.
        return antwort({ ...serverProjekt(projekt, d), name: p.name, counts: { unread_messages: unread, contents: (d.counts?.contents ?? 0) + kalender } });
      }
    }
    let members = [];
    let contents = 0;
    if (p.lokal) {
      const m = await invoke('projekt_mitglieder', { projekt });
      members = (m.mitglieder ?? []).map((x) => ({ user_id: x.id, name: x.name, role: x.rolle, ich: x.ich }));
      const f = await invoke('projekt_lokale_freigaben', { projekt }).catch(() => null);
      contents = (f?.freigaben ?? []).length;
    }
    contents += kalender;
    // Ungelesen im Chat vor Ort (bis wohin hier gelesen ist).
    const unread = await invoke('projekt_chat_ungelesen', { projekt }).catch(() => 0);
    return antwort({ id: p.id, name: p.name, my_role: p.rolle, lokal: p.lokal, members, counts: { unread_messages: unread, contents } });
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
  // Mit dem Kontonamen: Die App sucht die Person dahinter und unterschreibt
  // den Beitritt (docs/plan-projekte-eine-art.md, E2).
  async addProjectMember(projekt, nutzer, name) {
    return antwort(await amServer('hinzufuegen', projekt, nutzer, { name }));
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
  // Ein Chat für alle Projekte (E4, chateinheit.rs) -- mit und ohne Wolke.
  async getProjectMessages(projekt, params = {}) {
    const zeilen = await invoke('projekt_chat', { projekt });
    // Angesehen = gelesen (der Zähler am Reiter), nicht beim Blättern.
    if (!params.before_id) await invoke('projekt_chat_gelesen', { projekt }).catch(() => {});
    return antwort({ ...chatSeite(zeilen, params), link_targets: await verweisKarte(projekt) });
  },
  async sendProjectMessage(projekt, body, notifyMembers = false) {
    const zeilen = await invoke('projekt_chat_senden', { projekt, text: body, benachrichtigen: Boolean(notifyMembers) });
    // Die eben geschriebene: die jüngste eigene mit diesem Text.
    const meine = [...zeilen].reverse().find((z) => z.ich && z.text === body.trim()) ?? zeilen[zeilen.length - 1];
    return antwort({ ...chatNachricht(meine), link_targets: await verweisKarte(projekt) });
  },
  async pinProjectMessage(projekt, nachricht, an) {
    const uuid = chatUuid(nachricht);
    const zeilen = await invoke('projekt_chat_aktion', { projekt, nachricht: uuid, art: an ? 'anheften' : 'loesen' });
    return antwort({ ...chatNachricht(zeilen.find((z) => z.id === uuid)), link_targets: await verweisKarte(projekt) });
  },
  async deleteProjectMessage(projekt, nachricht) {
    await invoke('projekt_chat_aktion', { projekt, nachricht: chatUuid(nachricht), art: 'loeschen' });
    return antwort(null);
  },
  async getProjectLinkTargets(projekt) {
    return antwort(await verweisziele(projekt));
  },

  // Ein Hub auf openany.de wird dort gelöscht (nur die Eigentümerin), ein
  // lokaler nur auf diesem Gerät.
  async deleteProject(projekt) {
    const p = (await invoke('projekte_liste')).find((x) => x.id === projekt);
    if (p && !p.lokal) {
      await invoke('projekt_server_loeschen', { projekt });
    } else {
      await invoke('projekt_lokal_loeschen', { projekt });
    }
    return antwort({});
  },

  /* ── Das Board (docs/plan-whiteboard.md; eines je Hub) ──
   *
   * Jedes Mitglied speichert seine Fassung; projektwhiteboard.rs führt sie
   * je Element zusammen. Kein Anlegen, kein Löschen, kein Stand.
   */
  async getBoard(projekt) {
    return antwort(await invoke('projekt_board', { projekt }));
  },
  async saveBoard(projekt, inhalt) {
    await invoke('projekt_board_speichern', { projekt, inhalt });
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
