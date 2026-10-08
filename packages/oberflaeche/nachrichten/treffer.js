/**
 * Ein Text in Stücke zerlegt, die Treffer der Suche markiert -- ohne Rücksicht
 * auf Groß/klein. Für `v-for`, nie für `v-html`.
 *
 * @returns {{ t: string, treffer: boolean }[]}
 */
export function trefferTeile(text, suche) {
  const s = String(text ?? '');
  const q = String(suche ?? '').trim().toLowerCase();
  if (!q) return [{ t: s, treffer: false }];
  const klein = s.toLowerCase();
  const teile = [];
  let i = 0;
  for (let j = klein.indexOf(q); j >= 0; j = klein.indexOf(q, i)) {
    if (j > i) teile.push({ t: s.slice(i, j), treffer: false });
    teile.push({ t: s.slice(j, j + q.length), treffer: true });
    i = j + q.length;
  }
  if (i < s.length) teile.push({ t: s.slice(i), treffer: false });
  return teile;
}
