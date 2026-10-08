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

const SCHLUESSEL = 'openany_start_module';
const TRAEGT = KACHELN.map((m) => m.id);

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
