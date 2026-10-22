/*
 * Gemeinsam schreiben in der App (docs/plan-gemeinsam-schreiben.md):
 * einer Sitzung beitreten und den Provider aus dem gemeinsamen Paket mit
 * den Tauri-Befehlen verbinden (src-tauri/src/sitzungsbefehle.rs). Ob der
 * Knoten dieses Gerät ist oder ein anderes im Netz, entscheidet die Schale;
 * hier reist `geraet` nur mit.
 *
 * `projekt` gesetzt: `id` ist die Kennung der Notiz im Projekt. Sonst die
 * zk_id einer eigenen Notiz. `freigabe` gesetzt (Server-Projekt, geteilte
 * Mappe, docs/plan-whiteboard.md W3a): `id` ist die zk_id, Knoten ist das
 * Gerät in der Nähe, das die Mappe teilt.
 *
 * Zurück kommt, was Editor und Anwesenheitsleiste brauchen:
 *   editor       → MarkdownEditor `sitzung`
 *   anwesende    → Anwesende `teilnehmer` (reaktiv)
 *   aus          → Grund für „nur lesen“, sonst '' (reaktiv)
 *   ichSpeichere → ob dieses Gerät gerade speichert (reaktiv)
 *   speichern(titel, inhalt), schliessen()
 */
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { SitzungsProvider } from '@oberflaeche/editor/sitzungsProvider';

// `ausgangstext(doc, inhalt)`: wie eine leere Sitzung ihren Anfang bekommt.
// Ohne: Markdown für den Editor (`editor.ausgangstext`). Ein Whiteboard
// (`id` = `whiteboard:<uuid>`, W3b) legt seine Elemente selbst an.
export async function sitzungOeffnen({ projekt = null, id, freigabe = null, ausgangstext: anfang = null }) {
  const start = await invoke('sitzung_beitreten', { projekt, id, freigabe });
  const { geraet, notiz, kennung } = start;
  const p = start.projekt;
  const transport = {
    senden: (aenderungen, awareness) =>
      invoke('sitzung_senden', { geraet, projekt: p, notiz, kennung, aenderungen, awareness }),
    warten: (ab) => invoke('sitzung_warten', { geraet, projekt: p, notiz, kennung, ab }),
    zusammenfassen: (stand, bis) =>
      invoke('sitzung_zusammenfassen', { geraet, projekt: p, notiz, kennung, stand, bis }),
    verlassen: () => invoke('sitzung_verlassen', { geraet, projekt: p, notiz, kennung }),
  };

  const ausgangstext = ref(null);
  const anwesende = ref(start.teilnehmer ?? []);
  const aus = ref('');
  const ichSpeichere = ref(start.schreiber === kennung);

  const provider = new SitzungsProvider({
    start,
    transport,
    ausgangstext: anfang ?? ((_doc, inhalt) => { ausgangstext.value = inhalt; }),
    onAnwesende: (t) => { anwesende.value = t; },
    onSchreiber: (ja) => { ichSpeichere.value = ja; },
    onAus: (grund) => { aus.value = grund || 'aus'; },
  });
  const ich = (start.teilnehmer ?? []).find((t) => t.kennung === kennung);
  provider.laufen();

  return {
    start,
    kennung,
    editor: {
      doc: provider.doc,
      awareness: provider.awareness,
      ich: { name: ich?.name ?? '', color: start.farbe },
      ausgangstext,
    },
    anwesende,
    aus,
    ichSpeichere,
    // In einem Server-Projekt gibt es keine lokale Projektsache nachzuziehen
    // (`id` leer); die Mappe reist mit dem Abgleich des Besitzers.
    speichern: (titel, inhalt) =>
      invoke('sitzung_speichern', { geraet, projekt: p, notiz, kennung, titel, inhalt, id: projekt && !freigabe ? id : null, freigabe: start.freigabe || null }),
    schliessen: () => provider.zerstoeren(),
  };
}
