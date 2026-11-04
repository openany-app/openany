/*
 * Welche Module auf der Startseite als Kachel stehen — auf DIESEM Gerät.
 *
 * Die Webapp speichert das im Konto (`/user/modules`). Das Programm hat kein
 * Konto, das es fragen müsste, und die Auswahl ist Ansichtssache wie
 * Hell/Dunkel: Sie reist nicht mit dem Abgleich.
 *
 * Vorgabe: alles, was das Programm trägt. Eine leere Auswahl ist erlaubt —
 * dann zeigt die Startseite eine Übersicht statt Kacheln.
 */
import { ref } from 'vue';
import { KACHELN } from './module';

// Mit der kompakten Startseite (alle Kacheln vorgewählt) neu begonnen:
// Eine ältere Auswahl gilt nicht mehr, jede Kachel steht einmal da.
const SCHLUESSEL = 'openany_start_module_2';
// Nur, wofür start_uebersicht (lib.rs) Daten liefert. Die Nachrichten kommen
// aus mehreren Quellen (Direkt, Matrix, E-Mail) und haben noch keine
// Kachel-Daten -- als Widget stünde dort nichts.
const OHNE_WIDGET = ['messages'];
const TRAEGT = KACHELN.map((m) => m.id).filter((id) => !OHNE_WIDGET.includes(id));

const gelesen = (() => {
    try {
        const roh = JSON.parse(localStorage.getItem(SCHLUESSEL));
        return Array.isArray(roh) ? roh.filter((id) => TRAEGT.includes(id)) : null;
    } catch {
        return null;
    }
})();

const auswahl = ref(gelesen ?? [...TRAEGT]);

export function useStartModule() {
    const speichern = async (ids) => {
        auswahl.value = ids.filter((id) => TRAEGT.includes(id));
        try {
            localStorage.setItem(SCHLUESSEL, JSON.stringify(auswahl.value));
        } catch { /* dann eben nur für diese Sitzung */ }
        return auswahl.value;
    };

    return { auswahl, traegt: TRAEGT, speichern };
}
