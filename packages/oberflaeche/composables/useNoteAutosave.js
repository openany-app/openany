import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from '@oberflaeche/composables/useToast';
import { debounce } from '@oberflaeche/shared/debounce';

// 10 statt 2 Sekunden: Seit Notizen auf dem Server als Dateien liegen,
// schreibt jede Auslösung die GANZE Datei samt fsync. Damit bei längerem Takt
// nichts verloren geht, wird zusätzlich sofort gespeichert, wenn der Tab in
// den Hintergrund geht oder die Seite verschwindet.
//
// DAS IST DIE VORGABE, NICHT DIE REGEL: Eine Datenquelle kann mit
// `speicherTaktMs` einen eigenen Takt nennen. Das Programm tut das (1,5 s) –
// dort ist ein Speichern eine Zeile in der lokalen SQLite, und zehn Sekunden
// „Speichert …" waren nur Wartezeit und ein längeres Fenster, in dem ein hart
// beendetes Programm Text verliert (15.09.2026).
export const AUTOSAVE_MS = 10000;

/**
 * Was ein fehlgeschlagener Speicherversuch bedeutet.
 *
 * Als eigene Funktion, weil hier der einzige Fall steckt, in dem stilles
 * Weitermachen Daten kostet: Bei 409 hat der Server das Überschreiben
 * verweigert, weil die Datei außerhalb von openany geändert wurde (Zettlr,
 * Sync-Client). Wer das als gewöhnlichen Fehler behandelt, lässt den nächsten
 * Tastendruck die fremde Fassung überschreiben.
 *
 * @returns {{external: boolean, message: string}}
 */
export function mapSaveError(err) {
  const status = err?.response?.status;
  const data = err?.response?.data;

  if (status === 409) {
    return { external: true, message: data?.message || '' };
  }

  // Validierungsfehler (422, z. B. das 1-MB-Limit) konkret im Editor anzeigen;
  // alles Übrige bleibt ohne Zusatztext (der Status genügt der Anzeige).
  return {
    external: false,
    message: status === 422 ? (data?.errors?.content?.[0] || data?.message || '') : '',
  };
}

/**
 * Automatisches Speichern der offenen Notiz.
 *
 * Aus NotesWorkspace herausgelöst: Es ist der Teil mit den meisten stillen
 * Annahmen (Entprellung, Zwischenspeichern beim Wegklicken, gesperrter
 * Speicherpfad nach einer Fremdänderung) und gleichzeitig der, der am
 * wenigsten mit dem Explorer drumherum zu tun hat.
 *
 * @param {object}   o
 * @param {object}   o.api           Datenquelle (persönlich oder projekt-scoped);
 *                                   `api.speicherTaktMs` überschreibt AUTOSAVE_MS
 * @param {import('vue').Ref} o.openNote      Die offene Notiz (deep beobachtet)
 * @param {import('vue').Ref} o.noteReadOnly  Nur-Lese-Zugriff: dann nie speichern
 * @param {(data:object) => void} o.applySaved  Serverantwort in die Listen übernehmen
 * @param {(data:object) => void} o.reopenNote  Notiz nach Fremdänderung neu öffnen
 * @param {() => Promise<void>}   o.reloadAll   Kompletter Neuaufbau nach reimport
 */
export function useNoteAutosave({ api, openNote, noteReadOnly, applySaved, reopenNote, reloadAll }) {
  const { t } = useI18n();
  const toast = useToast();

  const saveStatus = ref(null); // null | 'saving' | 'saved' | 'error'
  const saveError = ref('');    // konkrete Server-Meldung (z. B. „Notiz zu groß")
  const externalChange = ref(false);
  const reimporting = ref(false);
  const pendingSave = ref(false);

  const saveNote = async (noteObj) => {
    if (!noteObj || !noteObj.id) return;
    saveStatus.value = 'saving';
    try {
      const res = await api.updateNote(noteObj.id, { title: noteObj.title, content: noteObj.content });
      applySaved(res.data);
      saveStatus.value = 'saved';
      saveError.value = '';
    } catch (err) {
      console.error('Auto-save failed', err);
      const { external, message } = mapSaveError(err);
      if (external) externalChange.value = true;
      saveError.value = message;
      saveStatus.value = 'error';
    }
  };

  const debouncedSave = debounce(saveNote, api.speicherTaktMs ?? AUTOSAVE_MS);

  const flushSave = () => {
    if (!pendingSave.value || !openNote.value || noteReadOnly.value) return;
    pendingSave.value = false;
    debouncedSave.cancel?.();
    saveNote(openNote.value);
  };

  // Von außen geänderte Notiz: Erst nach dem Neuladen darf wieder gespeichert
  // werden, sonst überschreibt der nächste Tastendruck die fremde Fassung doch.
  const reloadAfterExternalChange = async () => {
    if (!openNote.value) return;
    try {
      const res = await api.getNote(openNote.value.id);
      reopenNote(res.data);
      externalChange.value = false;
      saveError.value = '';
      saveStatus.value = null;
    } catch {
      toast.error(t('notes.loadFailed'));
    }
  };

  // „Änderungen einlesen": nach einer Sitzung in Zettlr den ganzen Bestand
  // abgleichen. Nur für persönliche Notizen – im Projektkontext gibt es keinen
  // eigenen Vault-Zweig.
  const reimportFromFiles = async () => {
    reimporting.value = true;
    try {
      const res = await api.reimportNotes();
      toast.success(res.data.message);
      await reloadAll();
    } catch (e) {
      toast.error(e.response?.data?.message || t('notes.reimportFailed'));
    } finally {
      reimporting.value = false;
    }
  };

  const retrySave = () => openNote.value && saveNote(openNote.value);

  watch(openNote, (n, o) => {
    if (n && o && n.id === o.id && !noteReadOnly.value) {
      saveStatus.value = 'saving';
      pendingSave.value = true;
      debouncedSave(n);
    }
  }, { deep: true });

  const onVisibilityChange = () => { if (document.hidden) flushSave(); };

  onMounted(() => {
    document.addEventListener('visibilitychange', onVisibilityChange);
    window.addEventListener('pagehide', flushSave);
  });

  onUnmounted(() => {
    document.removeEventListener('visibilitychange', onVisibilityChange);
    window.removeEventListener('pagehide', flushSave);
    flushSave();
  });

  return {
    saveStatus,
    saveError,
    externalChange,
    reimporting,
    saveNote,
    retrySave,
    flushSave,
    reloadAfterExternalChange,
    reimportFromFiles,
  };
}
