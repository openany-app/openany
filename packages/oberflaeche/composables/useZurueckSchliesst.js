/*
 * Zurück (auch die Android-Taste) schließt einen Rahmen über der Seite statt
 * die Seite selbst -- wie BaseModal, mit eigenem Verlaufseintrag. Liegt ein
 * weiterer Rahmen darüber, schließt Zurück erst ihn.
 *
 * Aufruf im setup: useZurueckSchliesst(() => emit('close')).
 */
import { onMounted, onBeforeUnmount } from 'vue';
import { eintragen, nochOffen, austragen } from './verlaufsrahmen';

export function useZurueckSchliesst(schliessen) {
  const eintrag = `rahmen-${Math.random().toString(36).slice(2)}`;
  const onPopstate = () => {
    if (!nochOffen(eintrag)) schliessen();
  };
  onMounted(() => {
    eintragen(eintrag);
    window.addEventListener('popstate', onPopstate);
  });
  onBeforeUnmount(() => {
    window.removeEventListener('popstate', onPopstate);
    austragen(eintrag);
  });
}
