<script setup>
/*
 * Die Kachel „Abgleich" — die Instanz, die Kopplung, der Abgleich.
 *
 * ALLES HIER IST FREIWILLIG. Wer nichts einträgt, hat ein vollständiges
 * Programm, das nur auf diesem Gerät lebt.
 *
 * Nur im Programm: Die Webapp IST die Instanz und braucht keine Kopplung.
 * Bis zum 15.09.2026 war das die ganze Einstellungsseite; jetzt steht sie
 * als eine Kachel neben Design, Sprache und Startseite aus dem Paket.
 */
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { RefreshCw, Radar } from 'lucide-vue-next';
import { useNahAbgleich } from '../nahAbgleich';

const props = defineProps({ lage: Object });
const { t } = useI18n();
const emit = defineEmits(['lage-geaendert']);

const form = ref({ geraetename: '' });

const kopplung = ref(null);
const kopplungsmeldung = ref('');
const bericht = ref(null);
const laeuft = ref(false);
const fehler = ref('');

/*
 * DER AUFFRISCHER. Unter Android ein Abgleich mit openany.de im Hintergrund,
 * etwa stündlich, nur mit Netz und nicht bei leerem Akku; im Mobilfunk nur
 * die Liste, die Dateien erst im WLAN. Der Schalter gehört dort der Schale
 * (`MainActivity.kt`), weil der Plan beim System lebt. Auf dem Schreibtisch
 * (seit 08.10.2026) etwa stündlich, solange das Programm läuft
 * (hintergrunddienste.rs) — der Wunsch steht in einstellungen.json.
 */
const android = Boolean(window.openanyAuffrischer);
const auffrischer = ref(android ? Boolean(window.openanyAuffrischer.an()) : null);
function auffrischerSetzen(e) {
    const an = Boolean(e.target.checked);
    if (android) window.openanyAuffrischer.setzen(an);
    else invoke('hintergrund_setzen', { auffrischen: an }).catch(() => {});
    auffrischer.value = an;
}

/*
 * DER WACHDIENST (nur Android): „Sofort benachrichtigen". Eine dauerhafte
 * Leitung zu openanys ntfy statt des stündlichen Auffrischens -- ohne
 * Google. Er trägt einen Dauerhinweis, deshalb ist er aus, bis jemand ihn
 * will (docs/plan-app-neuaufsatz.md, Phase 6). Der Schalter gehört wie der
 * des Auffrischers der Schale.
 */
const wachdienst = ref(window.openanyWachdienst ? Boolean(window.openanyWachdienst.an()) : null);
function wachdienstSetzen(e) {
    const an = Boolean(e.target.checked);
    if (window.openanyWachdienst) window.openanyWachdienst.setzen(an);
    else invoke('hintergrund_setzen', { sofort: an }).catch(() => {});
    wachdienst.value = an;
}

// Schreibtisch: beide Schalter aus einstellungen.json.
if (!android) {
    invoke('hintergrund_lage')
        .then((l) => { auffrischer.value = l.auffrischen; wachdienst.value = l.sofort; })
        .catch(() => {});
}

let takt = null;

/*
 * Geraete in der Naehe (APK 0.1): Wer ist im selben WLAN oder Hotspot?
 * Finden, paaren, abgleichen — ohne Server. Der Zustand gehört der Schale
 * (`nahAbgleich.js`); der Paarungsdialog steht dort auch, damit eine Anfrage
 * auf jeder Seite erscheint.
 */
const {
    nah, fehler: nahFehler, laeuft: nahLaeuft, berichte: nahBerichte,
    lesen: nahLesen, suchen: nahSuchen, paaren, vergessen, abgleichen: mitGeraetAbgleichen, offenMit,
} = useNahAbgleich();

/*
 * DU ALS PERSON (lokale Mitgliedschaften, Schritt 1, 01.10.2026). Hinter
 * jedem Gerät steht eine Person; eigene, gepaarte Geräte sind dieselbe. Wer
 * dich später vor Ort in ein Projekt einlädt, lädt die Person ein -- dann
 * sind alle deine Geräte dabei. Der Name gilt für alle, sobald sie sich
 * abgleichen.
 */
/*
 * ANFRAGEN VON UNBEKANNTEN (06.10.2026): Vorgabe aus. Dann kommt von
 * Geräten, deren Person hier weder bekannt noch per 6 Ziffern bestätigt ist,
 * nichts an. An: höchstens drei Nachrichten zu je 500 Zeichen, im
 * Anfragen-Ordner der Nachrichten (direktbefehle.rs).
 */
const anfragenErlaubt = ref(false);
async function anfragenLesen() {
    try { anfragenErlaubt.value = Boolean((await invoke('nah_anfragen')).erlaubt); } catch { /* bleibt aus */ }
}
async function anfragenSetzen(e) {
    const an = Boolean(e.target.checked);
    try {
        await invoke('nah_anfragen_erlauben', { an });
        anfragenErlaubt.value = an;
    } catch (err) {
        nahFehler.value = String(err);
        e.target.checked = anfragenErlaubt.value;
    }
}

const personenname = ref('');
let personennameBearbeitet = false;
watch(() => nah.value?.person?.name, (name) => {
    if (!personennameBearbeitet) personenname.value = name ?? '';
}, { immediate: true });
async function personennameSpeichern() {
    nahFehler.value = '';
    try {
        await invoke('nah_person_name', { name: personenname.value });
        personennameBearbeitet = false;
        await nahLesen();
    } catch (e) {
        nahFehler.value = String(e);
    }
}

async function nahAbgleichen(fingerabdruck) {
    nahFehler.value = '';
    try {
        await mitGeraetAbgleichen(fingerabdruck);
    } catch (e) {
        nahFehler.value = String(e);
    }
}
function zeitpunkt(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? '' : d.toLocaleString([], { dateStyle: 'short', timeStyle: 'short' });
}

/*
 * NUR OPENANY.DE (Tiffy, 08.10.2026). Hier stand ein Formular für eine
 * beliebige Instanz, ihr anyid und eine eigene Wurzel-CA. Jetzt gibt es nur
 * „Mit openany.de verbinden" -- die Adresse setzt die App selbst
 * (einstellungen.rs, `OPENANY`). Wer nie verbindet, hat weiter ein Programm,
 * das nur auf diesem Gerät lebt.
 *
 * „VERBINDEN" UND NICHT „KOPPELN": Geräte in der Nähe werden „gepaart"; das
 * Wort „koppeln" für openany.de lag zu nah daran.
 */
function ausLage() {
    if (!props.lage) return;
    form.value = { geraetename: props.lage.geraetename };
}

/** Den Namen dieses Geräts speichern (Abschnitt „Geräte in der Nähe"). */
async function speichern() {
    fehler.value = '';

    try {
        await invoke('einstellungen_speichern', { geraetename: form.value.geraetename });
        emit('lage-geaendert');
    } catch (e) {
        fehler.value = String(e);
    }
}

async function koppeln() {
    fehler.value = '';
    kopplungsmeldung.value = '';

    try {
        kopplung.value = await invoke('openany_verbinden');
        emit('lage-geaendert');
        takt = setInterval(abholen, kopplung.value.intervall * 1000);
    } catch (e) {
        fehler.value = String(e);
    }
}

/*
 * Ein Fehler beim Öffnen darf nicht ins Leere fallen.
 *
 * `openUrl` ist ein Plugin-Aufruf, und Plugin-Aufrufe können abgewiesen
 * werden — am 16.09.2026 wurden sie es, weil die Berechtigungsdatei fehlte.
 * Ohne `catch` wurde daraus eine unbehandelte Zusage: Der Knopf tat nichts,
 * und nichts sagte, warum. Ein Knopf, der scheitert, muss es sagen; die
 * Adresse steht ohnehin daneben, also ist der Weg nicht zu Ende.
 */
async function browserOeffnen() {
    try {
        await openUrl(kopplung.value.bestaetigen_url);
    } catch (e) {
        kopplungsmeldung.value = t('app.abgleich.browserFehler', { fehler: String(e) });
    }
}

async function abholen() {
    try {
        const stand = await invoke('kopplung_abholen');

        if (stand.stand === 'wartet') return;

        aufhoeren();

        if (stand.stand === 'gekoppelt') {
            kopplungsmeldung.value = t('app.abgleich.gekoppelt', { name: stand.name });
        } else {
            // Abgelaufen, erfunden oder schon abgeholt. Welcher der drei
            // Fälle es war, geht niemanden etwas an, der rät.
            kopplungsmeldung.value = t('app.abgleich.codeUngueltig');
        }

        emit('lage-geaendert');
    } catch (e) {
        aufhoeren();
        fehler.value = String(e);
    }
}

function aufhoeren() {
    clearInterval(takt);
    takt = null;
    kopplung.value = null;
}

async function abmelden() {
    await invoke('abmelden');
    emit('lage-geaendert');
}

async function abgleichen() {
    fehler.value = '';
    laeuft.value = true;
    bericht.value = null;

    try {
        bericht.value = await invoke('abgleichen');

        /*
         * DER SCHALE SAGEN, DASS SICH ETWAS GETAN HAT.
         *
         * Das fehlte, und es kostete zweierlei. Erstens blieb der Streifen
         * „offline" stehen: Er liest `lage`, und die stand noch auf dem Stand
         * vom Programmstart — als es noch keinen Lauf gab. Ein fehlerfrei
         * gelaufener Abgleich meldete also weiter „der Stand ist womöglich
         * alt". Zweitens wurde die offene Ansicht nicht neu aufgebaut; das
         * Geholte lag im Speicher und war nicht zu sehen.
         *
         * Am 16.09.2026 sah das zusammen so aus, als sei der Abgleich
         * gescheitert und als seien die Bilder nicht angekommen. Beides war
         * falsch — die Zahlen im Bericht daneben stimmten die ganze Zeit.
         */
        emit('lage-geaendert');
    } catch (e) {
        fehler.value = String(e);
    } finally {
        laeuft.value = false;
    }
}

/*
 * Erst die Marke zurücksetzen, dann gleich abgleichen — zwei Klicks für eine
 * Absicht wären einer zu viel, und der erste allein sähe aus, als sei nichts
 * geschehen: Er ändert eine Zahl, die niemand sieht.
 */
async function alles() {
    fehler.value = '';

    try {
        await invoke('bestand_neu_holen');
    } catch (e) {
        fehler.value = String(e);
        return;
    }

    await abgleichen();
}

function bytes(n) {
    if (n === null || n === undefined) return t('app.abgleich.unbegrenzt');

    const einheiten = ['B', 'KB', 'MB', 'GB', 'TB'];
    let i = 0;

    while (n >= 1024 && i < einheiten.length - 1) { n /= 1024; i++; }

    return `${n.toFixed(i ? 1 : 0)} ${einheiten[i]}`;
}

/*
 * EINE LAUFENDE KOPPLUNG WIEDERFINDEN — beim Aufbau UND beim Zurückkommen.
 *
 * Die Kopplung geht über zwei Programme: Hier steht der Code, bestätigt wird
 * im Browser. Genau dazwischen liegt der Bruch.
 *
 * WAS AM 16.09.2026 IM ZUGRIFFSPROTOKOLL STAND, sechs Versuche lang dasselbe:
 *
 *     201 POST /geraet/kopplung          die App beginnt
 *     202 POST /geraet/kopplung/abholen  sie fragt: wartet
 *     202 POST /geraet/kopplung/abholen  sie fragt: wartet
 *         ← hier wechselt der Mensch in den Browser. DAS FRAGEN HOERT AUF.
 *     200 POST /einstellungen/geraete/pruefen
 *     302 POST /einstellungen/geraete/bestaetigen   er bestätigt
 *         ← danach nie wieder ein `abholen`
 *
 * Das Gerät entsteht drüben erst beim ABHOLEN, nicht beim Bestätigen
 * (`Geraetekopplung::abholen` legt es an). Die Bestätigung war also jedes Mal
 * richtig — sie hat nur niemand abgeholt.
 *
 * WARUM `onMounted` ALLEIN NICHT REICHT, und das war mein Irrtum beim ersten
 * Versuch: Android baut die Webansicht nicht immer neu auf. Meistens hält es
 * sie bloss an — die Komponente bleibt, `setInterval` steht still, und
 * `onMounted` feuert nie wieder. Der Aufbau ist der seltenere Fall; der
 * häufige ist die Rückkehr.
 *
 * Deshalb hängt es an der SICHTBARKEIT, die beide Fälle abdeckt. Und deshalb
 * wird der Takt neu gesetzt statt vorausgesetzt: Ein Zeitgeber, der im
 * Hintergrund angehalten wurde, ist kein verlässlicher Zeitgeber mehr.
 */
async function laufendeKopplungAufnehmen() {
    try {
        const laufend = await invoke('kopplung_laeuft');

        if (!laufend) {
            // Drüben ist nichts mehr offen -- dann darf hier auch kein Kasten
            // mit einem Code stehen, der nichts mehr bedeutet.
            if (kopplung.value) aufhoeren();
            return;
        }

        kopplung.value = laufend;

        // Erst räumen, dann neu setzen: Sonst laufen nach dem zweiten
        // Zurückkommen zwei Takte nebeneinander und fragen doppelt.
        clearInterval(takt);
        takt = setInterval(abholen, laufend.intervall * 1000);

        // Sofort einmal fragen: Bestätigt wurde vermutlich GERADE, während
        // dieses Fenster fort war. Erst den Takt abzuwarten hiesse, den
        // Menschen nach seiner Bestätigung noch einmal warten zu lassen.
        abholen();
    } catch (e) {
        fehler.value = String(e);
    }
}

function beiSichtbarkeit() {
    if (document.visibilityState === 'visible') laufendeKopplungAufnehmen();
}

onMounted(() => {
    ausLage();
    nahLesen();
    anfragenLesen();
    laufendeKopplungAufnehmen();
    document.addEventListener('visibilitychange', beiSichtbarkeit);
});

onUnmounted(() => {
    aufhoeren();
    document.removeEventListener('visibilitychange', beiSichtbarkeit);
});
</script>

<template>
  <div class="karte p-6 shadow-sm space-y-4 lg:col-span-2">
    <h3 class="text-lg font-extrabold text-schrift flex items-center gap-2">
      <RefreshCw class="w-5 h-5 text-marke" /> {{ t('app.abgleich.titel') }}
    </h3>
    <h2 class="mt-6 text-base font-extrabold text-schrift">{{ t('app.abgleich.verbindungTitel') }}</h2>

    <template v-if="lage?.gekoppelt">
      <p class="text-sm text-fliess">{{ t('app.abgleich.verbundenAls', { name: lage.geraetename || t('app.abgleich.diesesGeraet') }) }}</p>
      <div class="flex items-center gap-2 mt-2">
        <button @click="abmelden" class="knopf">{{ t('app.abgleich.trennen') }}</button>
      </div>
      <!--
        „Trennen" entfernt die Ausweise nur hier. Drüben leben sie weiter, bis
        sie dort widerrufen werden -- ein Programm, das beides verwechselt,
        lässt nach einem Geräteverlust einen Zugang offen, von dem der Mensch
        glaubt, er sei zu.
      -->
      <p class="mt-2 text-sm text-leise">{{ t('app.abgleich.trennenHinweis') }}</p>
    </template>

    <template v-else-if="kopplung">
      <div class="mt-2 md:max-w-[460px] text-center bg-flaeche border border-linie p-4 rounded-xl shadow-sm">
        <!--
          `select-all`: Ein Fingertipp markiert den ganzen Code. Ohne das war
          er nur abzulesen — und wer die Adresse in den Browser kopierte,
          konnte den Code nicht mitnehmen und musste ihn im Kopf behalten,
          während er das Fenster wechselte.
        -->
        <div class="font-mono text-3xl tracking-[0.12em] text-marke mb-1.5 select-all">{{ kopplung.code }}</div>
        <p class="text-sm text-fliess">{{ t('app.abgleich.codeHinweis') }}</p>

        <!--
          DIE ADRESSE STEHT DA, nicht nur ein Knopf.

          Am 16.09.2026 stand jemand vor diesem Kasten: ein Code, die
          Aufforderung „im Browser eintippen" — und kein Wort darüber, WO.
          Der Knopf daneben tat nichts (die Berechtigungsdatei fehlte), und
          damit war der Weg zu Ende, ohne dass etwas nach einem Fehler aussah.

          Ein Knopf ist eine Bequemlichkeit; die Adresse ist die Auskunft. Wer
          sie lesen kann, kommt auch dann weiter, wenn der Knopf klemmt oder
          kein Browser eingerichtet ist — und weiß vorher, wohin er geschickt
          wird. Auswählbar, damit sie sich auf ein anderes Gerät übertragen
          lässt: Der Code gilt für DIESE Kopplung, nicht für dieses Gerät.
        -->
        <p class="mt-2 text-sm text-fliess">
          <span class="text-leise">{{ t('app.abgleich.dieseSeite') }}</span><br>
          <span class="font-mono break-all select-all">{{ kopplung.bestaetigen_url }}</span>
        </p>

        <div class="flex items-center justify-center gap-2 mt-2">
          <button @click="browserOeffnen" class="knopf">{{ t('app.abgleich.browserOeffnen') }}</button>
          <button @click="aufhoeren" class="knopf">{{ t('common.cancel') }}</button>
        </div>
        <p class="mt-2 text-sm text-leise">{{ t('app.abgleich.bestaetigungHinweis') }}</p>
      </div>
    </template>

    <template v-else>
      <p class="text-sm text-leise">{{ t('app.abgleich.verbindenHinweis') }}</p>
      <div class="flex items-center gap-2 mt-2">
        <button @click="koppeln" class="knopf-wichtig">{{ t('app.abgleich.verbinden') }}</button>
      </div>
    </template>

    <p v-if="kopplungsmeldung" class="mt-2 text-sm text-fliess">{{ kopplungsmeldung }}</p>

    <h2 class="mt-6 text-base font-extrabold text-schrift">{{ t('app.abgleich.titel') }}</h2>

    <div class="flex items-center gap-2 mt-2">
      <button class="knopf-wichtig" :disabled="!lage?.gekoppelt || laeuft" @click="abgleichen">
        {{ laeuft ? t('app.allgemein.laeuft') : t('app.allgemein.jetztAbgleichen') }}
      </button>
      <button class="knopf" :disabled="!lage?.gekoppelt || laeuft" @click="alles">
        {{ t('app.abgleich.allesNeu') }}
      </button>
    </div>
    <!--
      „Alles neu holen" setzt die Marke auf null; der nächste Lauf fragt dann
      nach dem BESTAND statt nach Änderungen. Gebraucht für Geräte, die vor
      dem 16.09.2026 gekoppelt wurden: Der Server lieferte damals auch bei
      Marke null nur sein Änderungsprotokoll, und das reichte nicht bis zum
      Anfang. Ihre Lücke bliebe sonst bestehen, obwohl sie drüben zu ist.
    -->
    <p class="mt-2 text-sm text-leise">{{ t('app.abgleich.allesNeuHinweis') }}</p>

    <!--
      WARUM DER KNOPF GRAU IST, steht daneben.

      Am 16.09.2026 stand jemand davor und kam nicht weiter: Der Knopf war
      tot, und nichts sagte, woran es lag. Die Bedingung war richtig — ohne
      Geräteschlüssel gibt es niemanden, mit dem abzugleichen wäre —, aber
      eine richtige Sperre, die schweigt, ist von einem Fehler nicht zu
      unterscheiden. Man sucht dann den Fehler und nicht den nächsten Schritt.

      Zwei Zustände, zwei Sätze: Wer noch keine Instanz eingetragen hat, ist
      einen Schritt weiter zurück als wer nur die Kopplung offen hat, und
      beide sollen lesen, was IHNEN fehlt.
    -->
    <label v-if="auffrischer !== null" class="mt-3 flex items-start gap-3 text-sm text-fliess md:max-w-[460px]">
      <input type="checkbox" class="mt-0.5" :checked="auffrischer" @change="auffrischerSetzen">
      <span>
        <span class="font-bold">{{ t('app.abgleich.hintergrund') }}</span> — {{ t(android ? 'app.abgleich.hintergrundHinweis' : 'app.abgleich.hintergrundHinweisDesktop') }}
      </span>
    </label>

    <label v-if="wachdienst !== null" class="mt-3 flex items-start gap-3 text-sm text-fliess md:max-w-[460px]">
      <input type="checkbox" class="mt-0.5" :checked="wachdienst" @change="wachdienstSetzen">
      <span>
        <span class="font-bold">{{ t('app.abgleich.sofort') }}</span> — {{ t(android ? 'app.abgleich.sofortHinweis' : 'app.abgleich.sofortHinweisDesktop') }}
      </span>
    </label>

    <p v-if="!lage?.gekoppelt" class="mt-2 text-sm text-leise">{{ t('app.abgleich.fehltVerbindung') }}</p>

    <div v-if="bericht" class="mt-2 md:max-w-[460px] bg-flaeche border border-linie p-4 rounded-xl shadow-sm">
      <table class="text-sm text-fliess">
        <tbody>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.gezogen') }}</td><td class="py-0.5 tabular-nums">{{ bericht.gezogen }}</td></tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.geschoben') }}</td><td class="py-0.5 tabular-nums">{{ bericht.geschoben }}</td></tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.uebersprungen') }}</td><td class="py-0.5 tabular-nums">{{ bericht.uebersprungen }}</td></tr>
          <!--
            EINE ZAHL OHNE NAMEN IST KEINE AUSKUNFT.

            Am 16.09.2026 stand hier „übersprungen 72", dann 33, dann 7 —
            und hinter drei der Ursachen steckten Fehler, die nur deshalb
            Stunden kosteten, weil die Zahl nichts sagte. Seitdem nennt der
            Läufer je Grund, wie oft; hier steht er eingerückt darunter.
          -->
          <tr v-for="(anzahl, grund) in bericht.uebersprungen_weil ?? {}" :key="grund">
            <td class="pl-4 pr-4 py-0.5 text-leise">{{ grund }}</td>
            <td class="py-0.5 tabular-nums text-leise">{{ anzahl }}</td>
          </tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.konflikte') }}</td><td class="py-0.5 tabular-nums">{{ bericht.konflikte }}</td></tr>
          <!--
            DIE INHALTE GEHÖREN IN DEN BERICHT.

            Der Abgleich hat zwei Hälften: Erst zieht er die LISTE (was es
            gibt), dann holt er die BYTES. Die Tabelle zeigte nur die erste.
            Was die zweite tat, kam zwar mit — `Abgleichanzeige.inhalte` gibt
            es seit jeher — wurde aber nirgends angezeigt.

            Am 16.09.2026 hat uns das einen halben Nachmittag gekostet: Die
            Bilder kamen ohne Vorschau an, im Bericht stand nichts dazu, und
            die einzige Spur lag im Zugriffsprotokoll des Servers. Ein
            Bericht, der die Hälfte verschweigt, die gerade nicht
            funktioniert, ist schlechter als keiner — er sagt „alles gut".
          -->
          <tr v-if="bericht.inhalte">
            <td class="pr-4 py-0.5">{{ t('app.abgleich.inhalteGeholt') }}</td>
            <td class="py-0.5 tabular-nums">{{ bericht.inhalte.geholt }}</td>
          </tr>
          <tr v-if="bericht.inhalte?.nicht_da">
            <td class="pr-4 py-0.5">{{ t('app.abgleich.nichtDa') }}</td>
            <td class="py-0.5 tabular-nums">{{ bericht.inhalte.nicht_da }}</td>
          </tr>
          <tr v-if="bericht.inhalte?.zu_wenig_platz">
            <td class="pr-4 py-0.5">{{ t('app.abgleich.keinPlatz') }}</td>
            <td class="py-0.5 tabular-nums">{{ bericht.inhalte.zu_wenig_platz }}</td>
          </tr>
          <tr v-if="bericht.belegt !== null">
            <td class="pr-4 py-0.5">{{ t('app.abgleich.speicher') }}</td>
            <td class="py-0.5 tabular-nums">{{ t('app.abgleich.belegtVon', { belegt: bytes(bericht.belegt), grenze: bytes(bericht.grenze) }) }}</td>
          </tr>
        </tbody>
      </table>

      <p v-if="bericht.konflikte" class="mt-2 text-sm text-fliess">{{ t('app.abgleich.konflikteHinweis', bericht.konflikte) }}</p>

      <p v-for="f in bericht.fehler" :key="f" class="mt-1 text-sm text-rose-600 dark:text-rose-400">{{ f }}</p>

      <!--
        Auch die Fehler der zweiten Hälfte. Ein Inhalt, der nicht ankam, ist
        kein kleinerer Fehler als eine Liste, die nicht ankam — er ist nur der
        leisere: Die Sache steht da, sie ist bloß leer.
      -->
      <p v-for="f in bericht.inhalte?.fehler ?? []" :key="f"
         class="mt-1 text-sm text-rose-600 dark:text-rose-400">{{ t('app.abgleich.inhaltFehler', { fehler: f }) }}</p>
    </div>

    <p v-if="fehler" class="mt-2 text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</p>

    <h2 class="mt-6 text-base font-extrabold text-schrift">{{ t('app.abgleich.ablage') }}</h2>
    <p class="text-sm text-leise font-mono break-all">{{ lage?.ordner }}</p>
    <p v-if="lage?.tresor" class="mt-1 text-sm text-leise">{{ t('app.abgleich.tresorJa') }}</p>
    <p v-else class="mt-1 text-sm text-rose-600 dark:text-rose-400">{{ t('app.abgleich.tresorNein') }}</p>

    <h2 class="mt-6 text-base font-extrabold text-schrift flex items-center gap-2">
      <Radar class="w-4.5 h-4.5 text-marke" /> {{ t('app.nah.titel') }}
    </h2>
    <p class="text-sm text-leise">{{ t('app.nah.hinweis') }}</p>
    <label class="flex items-start gap-3 text-sm text-fliess md:max-w-[460px]">
      <input type="checkbox" class="mt-0.5" :checked="anfragenErlaubt" @change="anfragenSetzen">
      <span>
        <span class="font-bold">{{ t('app.nah.anfragenErlauben') }}</span> — {{ t('app.nah.anfragenHinweis') }}
      </span>
    </label>
    <!-- DER NAME STEHT HIER und nicht mehr unter „Die Instanz": Er gilt auch
         ohne Server, und hier sieht man ihn gleich in der Liste der anderen.
         Gespeichert wird er mit den übrigen Einstellungen; ein neuer Name
         gilt sofort, ohne Neustart. -->
    <label class="block md:max-w-[460px]">
      <span class="block text-xs font-bold text-leise mb-1">{{ t('app.nah.geraetename') }}</span>
      <span class="flex items-center gap-2">
        <input v-model="form.geraetename" class="feld" :placeholder="t('app.nah.geraetenamePlatzhalter')" autocorrect="off">
        <button class="knopf-wichtig shrink-0" @click="speichern">{{ t('common.save') }}</button>
      </span>
    </label>
    <p v-if="nah?.fingerabdruck" class="text-xs text-leise">
      {{ t('app.nah.fingerabdruck') }} <span class="font-mono">{{ nah.fingerabdruck }}</span>
    </p>

    <template v-if="nah?.person">
      <label class="block md:max-w-[460px] mt-2">
        <span class="block text-xs font-bold text-leise mb-1">{{ t('app.nah.deinName') }}</span>
        <span class="flex items-center gap-2">
          <input v-model="personenname" class="feld" :placeholder="t('app.nah.deinNamePlatzhalter')" autocorrect="off"
                 @input="personennameBearbeitet = true">
          <button class="knopf-wichtig shrink-0" @click="personennameSpeichern">{{ t('common.save') }}</button>
        </span>
      </label>
      <p class="text-xs text-leise">{{ t('app.nah.deinNameHinweis') }}</p>
      <p class="text-xs text-leise">
        {{ t('app.nah.deineGeraete') }} <span class="font-bold text-fliess">{{ nah.person.geraete.map((g) => g.name || '?').join(' · ') }}</span>
        · {{ t('app.nah.personenId') }} <span class="font-mono">{{ nah.person.personen_id }}</span>
      </p>
    </template>
    <p v-if="nah?.fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ nah.fehler }}</p>
    <p v-else-if="nah && !nah.laeuft" class="text-sm text-leise">{{ t('app.nah.startet') }}</p>
    <p v-if="nahFehler" class="text-sm text-rose-600 dark:text-rose-400">{{ nahFehler }}</p>

    <ul v-if="nah?.geraete?.length" class="karte divide-y divide-linie">
      <li v-for="g in nah.geraete" :key="g.fingerabdruck" class="px-4 py-3 flex items-center gap-3">
        <span class="w-2 h-2 rounded-full shrink-0" :class="g.openany ? 'bg-marke' : 'bg-slate-400'"></span>
        <span class="min-w-0 flex-1">
          <span class="block font-bold text-schrift truncate">{{ g.name }}</span>
          <span class="block text-xs text-leise truncate">
            {{ g.openany ? 'openany' : (g.modell || 'LocalSend') }} · {{ g.adresse }} · <span class="font-mono">{{ g.fingerabdruck.slice(0, 12) }}</span>
          </span>
        </span>
        <span v-if="g.gepaart" class="text-xs font-bold text-marke shrink-0">{{ t('app.nah.gepaart') }}</span>
        <button v-else-if="g.openany && !offenMit(g.fingerabdruck)" class="knopf shrink-0" @click="paaren(g.fingerabdruck)">{{ t('app.nah.paaren') }}</button>
      </li>
    </ul>
    <p v-else-if="nah?.laeuft" class="text-sm text-leise">{{ t('app.nah.niemand') }}</p>
    <div class="flex items-center gap-2">
      <button class="knopf" :disabled="!nah?.laeuft" @click="nahSuchen">{{ t('app.nah.suchen') }}</button>
    </div>

    <template v-if="nah?.gepaarte?.length">
      <h3 class="mt-4 text-sm font-extrabold text-schrift">{{ t('app.nah.gepaarteTitel') }}</h3>
      <ul class="karte divide-y divide-linie md:max-w-[460px]">
        <li v-for="g in nah.gepaarte" :key="'gepaart-' + g.fingerabdruck" class="px-4 py-3">
          <div class="flex items-center gap-3">
            <span class="w-2 h-2 rounded-full shrink-0" :class="g.da ? 'bg-emerald-500' : 'bg-slate-400'"></span>
            <span class="min-w-0 flex-1">
              <span class="block font-bold text-schrift truncate">{{ g.name }}</span>
              <span class="block text-xs text-leise truncate">{{ g.da ? t('app.nah.inDerNaehe') : t('app.nah.nichtInDerNaehe') }} · <span class="font-mono">{{ g.fingerabdruck.slice(0, 12) }}</span></span>
              <span v-if="g.letzter_lauf" class="block text-xs text-leise truncate">{{ t('app.nah.zuletzt', { zeit: zeitpunkt(g.letzter_lauf) }) }}</span>
            </span>
          </div>
          <div class="flex flex-wrap items-center gap-2 mt-2 pl-5">
            <button class="knopf-wichtig" :disabled="!!nahLaeuft" @click="nahAbgleichen(g.fingerabdruck)">
              {{ nahLaeuft === g.fingerabdruck ? t('app.allgemein.laeuft') : t('app.allgemein.jetztAbgleichen') }}
            </button>
            <button class="knopf" :disabled="!!nahLaeuft" @click="vergessen(g.fingerabdruck)">{{ t('app.nah.vergessen') }}</button>
          </div>
          <div v-if="nahBerichte[g.fingerabdruck]" class="mt-2 ml-5 text-sm text-fliess">
            {{ t('app.nah.bericht', { geholt: nahBerichte[g.fingerabdruck].gezogen, geschickt: nahBerichte[g.fingerabdruck].geschoben }) }}<template v-if="nahBerichte[g.fingerabdruck].konflikte"> · {{ t('app.nah.berichtKonflikte', { count: nahBerichte[g.fingerabdruck].konflikte, name: g.name }, nahBerichte[g.fingerabdruck].konflikte) }}</template>
            <p v-for="(anzahl, grund) in nahBerichte[g.fingerabdruck].uebersprungen_weil ?? {}" :key="grund" class="mt-1 text-leise">
              {{ t('app.nah.uebersprungen', { grund, anzahl }) }}
            </p>
            <p v-for="f in nahBerichte[g.fingerabdruck].fehler" :key="f" class="mt-1 text-rose-600 dark:text-rose-400">{{ f }}</p>
          </div>
          <p v-else-if="g.letzter_fehler" class="mt-2 ml-5 text-sm text-rose-600 dark:text-rose-400">{{ g.letzter_fehler }}</p>
        </li>
      </ul>
    </template>
  </div>
</template>
