/*
 * Die Datenquelle der Notiz-Arbeitsfläche für eine GETEILTE Mappe im Projekt
 * (docs/plan-notizen-e2e.md, 6c): dieselben Methoden wie lokaleNotizenQuelle,
 * dahinter die Befehle aus src-tauri/src/mappenteilen.rs.
 *
 * Die Notizen holt die App einmal geöffnet vom Server (sie liegen dort nur
 * verschlüsselt) und rechnet Mappen, Suche, Schlagwörter und Rückverweise
 * daraus selbst. Gespeichert wird je Notiz, mit der Basis, auf der die
 * Änderung beruht -- war jemand schneller, legt die App eine Kopie daneben.
 *
 * Mit der Stufe „ansehen" ist alles nur zu lesen. Löschen gibt es in einer
 * fremden Mappe nicht, auch nicht mit „bearbeiten".
 *
 * JEDER `invoke` STEHT AUF EINER ZEILE mit einfachen Schlüsseln (verdrahtung.rs).
 */
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { sitzungOeffnen } from '../sitzung';
import { i18n } from '@oberflaeche/i18n';

const { t } = i18n.global;
const antwort = (data) => ({ data });
const alsFehler = (text) => Object.assign(new Error(text), { response: { status: 422, data: { message: text, errors: { content: [text] } } } });

const TAG = /(?:^|\s)#([\p{L}][\p{L}\p{N}_/-]{0,49})/gu;

export function geteilteMappeQuelle(projekt, freigabe, { beiKonflikt, stufe = null } = {}) {
  let stand = null; // { pfad, name, stufe, notizen }
  const mappenIds = new Map(); // Pfad -> Zahl (die Arbeitsfläche rechnet mit Zahlen)

  const darfSchreiben = () => stand?.stufe === 'edit';
  const zugriff = () => (darfSchreiben() ? 'edit' : 'read');

  const mappeId = (pfad) => {
    if (!mappenIds.has(pfad)) mappenIds.set(pfad, mappenIds.size + 1);
    return mappenIds.get(pfad);
  };
  const pfadVon = (id) => [...mappenIds.entries()].find(([, i]) => i === id)?.[0] ?? stand?.pfad;

  async function laden() {
    stand = await invoke('projekt_mappe_notizen', { projekt, freigabe });
    mappeId(stand.pfad);
    for (const n of stand.notizen) {
      // Unterordner auf dem Weg anlegen, damit die Brotkrumen stimmen.
      const teile = (n.folder ?? stand.pfad).slice(stand.pfad.length).split('/').filter(Boolean);
      let p = stand.pfad;
      for (const teil of teile) { p = `${p}/${teil}`; mappeId(p); }
    }
    return stand;
  }
  const bereit = async () => stand ?? laden();

  const kurz = (n) => ({ id: n.zk_id, title: n.title, note_folder_id: mappeId(n.folder ?? stand.pfad), updated_at: n.updated_at ?? '', access: zugriff() });
  const voll = (n) => ({ ...kurz(n), content: n.content });

  async function speichern(id, titel, inhalt, mappe) {
    if (!darfSchreiben()) throw alsFehler(t('app.mappe.nurAnsehen'));
    const alt = id ? stand.notizen.find((n) => n.zk_id === id) : null;
    const r = await invoke('projekt_mappe_notiz_speichern', { projekt, freigabe, zkId: id ?? null, titel, inhalt, mappe, basis: alt?.hash ?? null }).catch((e) => { throw alsFehler(String(e)); });
    if (r.konflikt) beiKonflikt?.();
    await laden();
    const n = stand.notizen.find((x) => x.zk_id === (r.konflikt ? id : r.zk_id)) ?? stand.notizen.find((x) => x.zk_id === r.zk_id);
    return antwort(n ? voll(n) : null);
  }

  return {
    supportsLibrary: false,
    ai: null,
    speicherTaktMs: 3000,

    getNotes: async (folderId, page, tag, q) => {
      await bereit();
      const pfad = folderId == null ? stand.pfad : pfadVon(folderId);
      const suche = (q ?? '').toLowerCase();
      const items = stand.notizen
        .filter((n) => (n.folder ?? stand.pfad) === pfad)
        .filter((n) => !suche || n.title.toLowerCase().includes(suche) || n.content.toLowerCase().includes(suche))
        .filter((n) => !tag || [...n.content.matchAll(TAG)].some((m) => m[1] === tag))
        .map(kurz);
      return antwort({ items, next_page: null });
    },
    getNote: async (id) => {
      await bereit();
      const n = stand.notizen.find((x) => x.zk_id === id);
      if (!n) throw alsFehler(t('app.mappe.gibtEsNicht'));
      return antwort(voll(n));
    },
    createNote: async (d) => { await bereit(); return speichern(null, d.title ?? '', d.content ?? '', pfadVon(d.note_folder_id)); },
    updateNote: async (id, d) => {
      await bereit();
      const n = stand.notizen.find((x) => x.zk_id === id);
      return speichern(id, d.title ?? n?.title ?? '', d.content ?? n?.content ?? '', n?.folder ?? stand.pfad);
    },
    // Kein deleteNote: Die Arbeitsfläche zeigt dann weder Verschieben noch Löschen.

    // Gemeinsam schreiben vor Ort (docs/plan-whiteboard.md, W3a): nur, wo
    // die Mappe zum Bearbeiten geteilt ist. Ist das Gerät, das sie teilt,
    // nicht in der Nähe, schlägt das fehl -- dann speichert die Arbeitsfläche
    // wie bisher über den Server.
    ...(stufe === 'edit' ? { sitzungOeffnen: (note) => sitzungOeffnen({ projekt, id: note.id, freigabe }) } : {}),

    getNoteFolders: async () => {
      await bereit();
      const folders = [...mappenIds.entries()].map(([pfad, id]) => {
        const eltern = pfad === stand.pfad ? null : pfad.slice(0, pfad.lastIndexOf('/'));
        return { id, name: pfad === stand.pfad ? stand.name : pfad.slice(pfad.lastIndexOf('/') + 1), parent_id: eltern ? mappenIds.get(eltern) ?? null : null, access: zugriff(), druckfassung: null };
      });
      return antwort({ folders });
    },
    // Eine neue Unterordner-Ebene entsteht mit ihrer ersten Notiz.
    createNoteFolder: async (name, parentId) => {
      if (!darfSchreiben()) throw alsFehler(t('app.mappe.nurAnsehen'));
      return antwort({ id: mappeId(`${pfadVon(parentId)}/${name}`) });
    },
    updateNoteFolder: async () => { throw alsFehler(t('app.mappe.nichtLoeschen')); },
    deleteNoteFolder: async () => { throw alsFehler(t('app.mappe.nichtLoeschen')); },

    // Bilder der Mappe (7c): Zuordnung und Bild mit dem Mappenschlüssel; die
    // Vorschauen holt die App vorab, `assetUrl` ist synchron.
    getNoteAssets: async (folderId) => {
      await bereit();
      const mappe = folderId == null ? stand.pfad : pfadVon(folderId);
      return antwort({ items: await invoke('projekt_mappe_anhaenge', { projekt, freigabe, mappe }) });
    },
    assetUrl: (anhang) => (anhang?.pfad ? convertFileSrc(anhang.pfad) : null),
    // Ohne sie scheitert das Laden des Notiz-Kontexts (useNoteAssets) still.
    getNoteCitations: async () => antwort({ items: [] }),

    getNoteTitles: async () => { await bereit(); return antwort({ items: stand.notizen.map((n) => ({ id: n.zk_id, title: n.title })) }); },
    resolveTitle: async (titel) => {
      await bereit();
      const n = stand.notizen.find((x) => x.title.toLowerCase() === String(titel).toLowerCase() || x.zk_id === titel);
      return antwort({ note: n ? { id: n.zk_id, title: n.title } : null });
    },
    getNoteTags: async () => {
      await bereit();
      const zahl = new Map();
      for (const n of stand.notizen) for (const m of n.content.matchAll(TAG)) zahl.set(m[1], (zahl.get(m[1]) ?? 0) + 1);
      return antwort({ items: [...zahl.entries()].map(([tag, count]) => ({ tag, count })) });
    },
    getNoteBacklinks: async (id) => {
      await bereit();
      const ziel = stand.notizen.find((x) => x.zk_id === id);
      if (!ziel) return antwort({ items: [] });
      const muster = [`[[${ziel.title}]]`.toLowerCase(), `[[${ziel.zk_id}`];
      return antwort({ items: stand.notizen.filter((n) => n.zk_id !== id && muster.some((m) => n.content.toLowerCase().includes(m))).map((n) => ({ id: n.zk_id, title: n.title })) });
    },

  };
}
