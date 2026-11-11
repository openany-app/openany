/*
 * Den Inhalt der Hubs abgleichen: Chat, Board, geteilte Mappen und Kalender,
 * Mitgliederliste -- auf openany.de und, für lokale Hubs, vor Ort.
 *
 * Teil des einen Abgleichs (App.vue `abgleichenAlle`): Welche Hubs es gibt
 * und wie sie heißen, kommt mit dem Konto; was darin steht, kommt hier. Der
 * Knopf in den Hubs und der im Menü tun deshalb dasselbe.
 *
 * Zurück: ob sich etwas geändert hat.
 */
import { projekteQuelle as quelle } from './quellen/projekte';

export async function hubsAbgleichen({ t, toast, gekoppelt }) {
    let geaendert = false;
    const projekte = await quelle.liste().catch(() => []);
    if (!projekte.length) return false;

    if (gekoppelt && projekte.some((p) => !p.lokal)) {
        try {
            for (const lauf of await quelle.abgleichen()) {
                if (lauf.fehler?.length) {
                    toast.error(t('app.projekte.laufFehler', { name: lauf.name, fehler: lauf.fehler[0] }));
                } else if (lauf.gezogen || lauf.geschoben) {
                    geaendert ||= Boolean(lauf.gezogen);
                    toast.success(t('app.projekte.laufServer', { name: lauf.name, geholt: lauf.gezogen, geschickt: lauf.geschoben }));
                }
            }
        } catch (e) {
            toast.error(String(e));
        }
    }

    if (projekte.some((p) => p.lokal)) {
        for (const l of await quelle.nahAbgleichen().catch(() => [])) {
            if (l.fehler) {
                toast.error(t('app.projekte.laufNahFehler', { name: l.name, mit: l.mit, fehler: l.fehler }));
            } else if (l.entfernt) {
                geaendert = true;
                toast.info(t('app.projekte.laufEntfernt', { name: l.name }));
            } else {
                geaendert = true;
                const was = [t('app.projekte.notizen', l.notizen)];
                if (l.chat) was.push(t('app.projekte.nachrichten', l.chat));
                if (l.planung) was.push(t('app.projekte.planung', l.planung));
                if (l.neue_mitglieder) was.push(t('app.projekte.neueMitglieder', l.neue_mitglieder));
                toast.success(t('app.projekte.laufNah', { name: l.name, mit: l.mit, was: was.join(', ') }));
            }
        }
    }

    return geaendert;
}
