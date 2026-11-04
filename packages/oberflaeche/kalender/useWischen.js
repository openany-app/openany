// Wischen auf Handy und Tablet: meldet eine klare Geste nach links, rechts,
// oben oder unten. Klar heißt: mindestens 50 Pixel weit und deutlich mehr in
// die eine Richtung als in die andere -- ein schräges Scrollen löst nichts
// aus. Ab `md` (Pfeile im Kopf) bleibt es still.
import { nextTick, onMounted, onUnmounted, ref } from 'vue';

const MINDESTWEG = 50;
const VORRANG = 1.5;

export function useWischen(elRef, melden) {
  let start = null;

  const beginn = (e) => {
    if (e.touches.length !== 1) { start = null; return; }
    start = { x: e.touches[0].clientX, y: e.touches[0].clientY };
  };
  const ende = (e) => {
    if (!start || window.matchMedia('(min-width: 768px)').matches) { start = null; return; }
    const t = e.changedTouches[0];
    const dx = t.clientX - start.x;
    const dy = t.clientY - start.y;
    start = null;
    if (Math.abs(dx) >= MINDESTWEG && Math.abs(dx) > VORRANG * Math.abs(dy)) melden(dx < 0 ? 'links' : 'rechts');
    else if (Math.abs(dy) >= MINDESTWEG && Math.abs(dy) > VORRANG * Math.abs(dx)) melden(dy < 0 ? 'hoch' : 'runter');
  };

  let el = null;
  onMounted(() => {
    el = elRef.value;
    el?.addEventListener('touchstart', beginn, { passive: true });
    el?.addEventListener('touchend', ende, { passive: true });
  });
  onUnmounted(() => {
    el?.removeEventListener('touchstart', beginn);
    el?.removeEventListener('touchend', ende);
  });
}

// Blättern mit Bewegung: Die alte Seite gleitet in Wischrichtung hinaus, die
// neue kommt von der Gegenseite. Das Raster bleibt dabei dasselbe Element, so
// behält die Stundenachse ihre Scrollposition. `laeuft` sperrt das Nachmessen
// der Rasterhöhe, denn eine verschobene Fläche liefert falsche Maße.
const AUSSCHLAG = { links: ['-25%', 0], rechts: ['25%', 0], hoch: [0, '-25%'], runter: [0, '25%'] };
const VERSATZ_NULL = 'translate(0, 0)';

export function useBlaettern(elRef) {
  const laeuft = ref(false);
  let letzter = 0;

  const blaettern = async (richtung, aendern) => {
    const el = elRef.value;
    if (!el?.animate || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      aendern();
      return;
    }
    const lauf = ++letzter;
    laeuft.value = true;
    // Ein zweiter Wisch mitten im ersten bricht nur die Bewegung ab; das
    // Blättern des ersten findet trotzdem statt, sonst ginge ein Schritt verloren.
    el.getAnimations().forEach((a) => a.cancel());

    const [x, y] = AUSSCHLAG[richtung];
    const raus = el.animate(
      [{ transform: VERSATZ_NULL, opacity: 1 }, { transform: `translate(${x}, ${y})`, opacity: 0 }],
      { duration: 120, easing: 'ease-in', fill: 'forwards' },
    );
    await raus.finished.catch(() => {});
    aendern();
    await nextTick();
    if (lauf !== letzter) return;

    raus.cancel();
    const gegen = (w) => (w === 0 ? 0 : w.startsWith('-') ? w.slice(1) : `-${w}`);
    const rein = el.animate(
      [{ transform: `translate(${gegen(x)}, ${gegen(y)})`, opacity: 0 }, { transform: VERSATZ_NULL, opacity: 1 }],
      { duration: 200, easing: 'ease-out' },
    );
    await rein.finished.catch(() => {});
    if (lauf === letzter) laeuft.value = false;
  };

  return { blaettern, laeuft };
}
