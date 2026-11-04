/*
 * Zurück (auch die Android-Taste) geht in einer Ordneransicht eine Ebene hoch
 * und verlässt die Ansicht erst ganz oben.
 *
 * JE EBENE EIN VERLAUFSEINTRAG, ANGELEGT BEIM BETRETEN: Android überspringt
 * beim Zurückgehen Einträge, von denen aus die Seite ohne Tipp einen neuen
 * angelegt hat. Ein Eintrag, der nach jedem Zurück neu gelegt würde, wirkte
 * deshalb nur einmal; der nächste Druck verließe die App.
 *
 * Aufruf im setup: useZurueckEineEbene(() => anzahlEbenenUnterHalbDerWurzel, hoch).
 */
import { onBeforeUnmount, onMounted, watch } from 'vue';
import { abmelden, austragen, eintragen, nochOffen } from './verlaufsrahmen';

export function useZurueckEineEbene(tiefe, hoch) {
  const kennung = Math.random().toString(36).slice(2);
  const name = (ebene) => `ebene-${kennung}-${ebene}`;
  let liegen = 0;

  const nachziehen = () => {
    const soll = tiefe();
    while (liegen < soll) eintragen(name(++liegen), { ebene: true });
    // Über den Pfad nach oben gesprungen: die übersprungenen Einträge abräumen.
    if (liegen > soll) {
      const zuviel = liegen - soll;
      while (liegen > soll) abmelden(name(liegen--));
      history.go(-zuviel);
    }
  };
  const onPopstate = () => {
    if (liegen === 0 || nochOffen(name(liegen))) return;
    abmelden(name(liegen--));
    hoch();
  };

  watch(tiefe, nachziehen, { flush: 'post' });
  onMounted(() => {
    window.addEventListener('popstate', onPopstate);
    nachziehen();
  });
  onBeforeUnmount(() => {
    window.removeEventListener('popstate', onPopstate);
    while (liegen > 1) abmelden(name(liegen--));
    if (liegen === 1) austragen(name(1));
  });
}
