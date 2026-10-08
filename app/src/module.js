/*
 * Welche Ansicht zu welchem Modul gehört — das Gegenstück zu den Routen der
 * Webapp.
 *
 * DIE MODULE SELBST (Namen, Symbole, Reihenfolge) KOMMEN AUS DEM PAKET
 * (`@oberflaeche/module`), dieselbe Liste wie in der Webapp. Hier steht nur,
 * was dieses Programm daraus macht.
 *
 * `grund` heißt: Der Ort steht in der Leiste, aber dieses Programm trägt ihn
 * noch nicht. Die Ansicht sagt das selbst, statt eine leere Liste zu zeigen —
 * ein Ort, der aussieht, als hätte man nichts, ist schlimmer als einer, der
 * sagt, dass er noch nichts kann.
 */
import { NAV_MODULES, MODULES } from '@oberflaeche/module';

import NotizenAnsicht from './ansichten/NotizenAnsicht.vue';
import KalenderAnsicht from './ansichten/KalenderAnsicht.vue';
import KontakteAnsicht from './ansichten/KontakteAnsicht.vue';
import SpeicherAnsicht from './ansichten/SpeicherAnsicht.vue';
import NachrichtenAnsicht from './ansichten/NachrichtenAnsicht.vue';
import ProjekteAnsicht from './ansichten/ProjekteAnsicht.vue';
import NochNicht from './ansichten/NochNicht.vue';

export const ANSICHTEN = {
    notes: { bau: NotizenAnsicht },
    calendar: { bau: KalenderAnsicht },
    contacts: { bau: KontakteAnsicht },
    // Dateien und Dokumente auf dem Gerät (Phase 3, Stufe A). Zwischen
    // Geräten und mit openany.de kommen sie mit den Stufen B und D.
    files: { bau: SpeicherAnsicht },
    // Boards, Spalten und Karten aus dem Strom des Projekts
    // (docs/plan-projekte-abgleich.md). Angelegt und eingeladen wird in der
    // Webapp; hier wird gezeigt und verschoben.
    projects: { bau: ProjekteAnsicht },
    // Matrix, Ende-zu-Ende auf diesem Gerät (Phase 4). Ein Chat zwischen
    // eigenen Geräten ohne Netz (5d) ist das nicht — dafür bräuchte es keinen
    // Homeserver.
    messages: { bau: NachrichtenAnsicht },
};

/** Die Orte in der Leiste: dieselben wie in der Webapp. */
export const ORTE = NAV_MODULES;

/** Die Kacheln der Startseite: alles, was dieses Programm heute trägt. */
export const KACHELN = MODULES.filter((m) => ANSICHTEN[m.id] && ANSICHTEN[m.id].bau !== NochNicht);
