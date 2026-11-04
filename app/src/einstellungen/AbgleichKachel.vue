<script setup>
/*
 * Die Kachel „Abgleich" — die Verbindung zu openany.de und der Abgleich damit.
 *
 * ALLES HIER IST FREIWILLIG. Wer nichts einträgt, hat ein vollständiges
 * Programm, das nur auf diesem Gerät lebt.
 *
 * Nur im Programm: Die Webapp IST die Instanz und braucht keine Kopplung.
 * Eine Kachel neben Design, Sprache und Startseite aus dem Paket.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { RefreshCw } from 'lucide-vue-next';
import FreischaltungNeu from './FreischaltungNeu.vue';
import ProjekteAufServer from './ProjekteAufServer.vue';
import HilfeLink from '@oberflaeche/base/HilfeLink.vue';

defineProps({ lage: Object });
const { t } = useI18n();
const emit = defineEmits(['lage-geaendert']);

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
 * etwa stündlich, solange das Programm läuft
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

/*
 * WEITERLAUFEN (nur Schreibtisch): beim Anmelden starten und beim Schließen im
 * Menü-Symbol bleiben. So gleicht der Rechner auch ohne offenes Fenster ab.
 */
const weiterlaufen = ref(null);
function weiterlaufenSetzen(e) {
    const an = Boolean(e.target.checked);
    invoke('hintergrund_setzen', { weiterlaufen: an }).catch(() => {});
    weiterlaufen.value = an;
}

// Schreibtisch: die Schalter aus einstellungen.json.
if (!android) {
    invoke('hintergrund_lage')
        .then((l) => { auffrischer.value = l.auffrischen; wachdienst.value = l.sofort; weiterlaufen.value = l.weiterlaufen; })
        .catch(() => {});
}

let takt = null;

/*
 * NUR OPENANY.DE: „Mit openany.de verbinden" -- die Adresse setzt die App
 * selbst
 * (einstellungen.rs, `OPENANY`). Wer nie verbindet, hat weiter ein Programm,
 * das nur auf diesem Gerät lebt.
 *
 * „VERBINDEN" UND NICHT „KOPPELN": Geräte in der Nähe werden „gepaart"; das
 * Wort „koppeln" für openany.de lag zu nah daran.
 */
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
 * werden (etwa wenn die Berechtigungsdatei fehlt). Ohne `catch` wird daraus
 * eine unbehandelte Zusage: Der Knopf tut nichts, und nichts sagt, warum. Ein Knopf, der scheitert, muss es sagen; die
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
         * DER SCHALE SAGEN, DASS SICH ETWAS GETAN HAT. Sonst bliebe der
         * Streifen „offline" stehen (er liest `lage`, die noch den Stand vom
         * Programmstart hat), und die offene Ansicht zeigte das Geholte nicht.
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
 * im Browser. Das Gerät entsteht drüben erst beim ABHOLEN, nicht beim
 * Bestätigen (`Geraetekopplung::abholen` legt es an) -- hört das Fragen auf,
 * während der Mensch im Browser bestätigt, holt niemand die Bestätigung ab.
 *
 * WARUM `onMounted` ALLEIN NICHT REICHT: Android baut die Webansicht nicht immer neu auf. Meistens hält es
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
      <HilfeLink ziel="settings-abgleich" />
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

    <!--
      Zwei Wege: vorne über ein anderes Gerät, dahinter der Weg fürs erste
      Gerät -- Passwort im Browser.
    -->
    <template v-else>
      <p class="text-sm text-leise">{{ t('app.abgleich.verbindenHinweis') }}</p>
      <FreischaltungNeu @lage-geaendert="emit('lage-geaendert')" @passwort="koppeln" />
    </template>

    <p v-if="kopplungsmeldung" class="mt-2 text-sm text-fliess">{{ kopplungsmeldung }}</p>

    <h2 class="mt-6 text-base font-extrabold text-schrift">{{ t('app.abgleich.datenTitel') }}</h2>

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
      nach dem BESTAND statt nach Änderungen -- für den Fall, dass einem
      Gerät etwas fehlt, das es drüben gibt.
    -->
    <p class="mt-2 text-sm text-leise">{{ t('app.abgleich.allesNeuHinweis') }}</p>

    <!--
      WARUM DER KNOPF GRAU IST, steht daneben: Ohne Geräteschlüssel gibt es
      niemanden, mit dem abzugleichen wäre -- aber eine richtige Sperre, die
      schweigt, ist von einem Fehler nicht zu unterscheiden. Man sucht dann den Fehler und nicht den nächsten Schritt.

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

    <label v-if="weiterlaufen !== null" class="mt-3 flex items-start gap-3 text-sm text-fliess md:max-w-[460px]">
      <input type="checkbox" class="mt-0.5" :checked="weiterlaufen" @change="weiterlaufenSetzen">
      <span>
        <span class="font-bold">{{ t('app.abgleich.weiterlaufen') }}</span> — {{ t('app.abgleich.weiterlaufenHinweis') }}
      </span>
    </label>

    <p v-if="!lage?.gekoppelt" class="mt-2 text-sm text-leise">{{ t('app.abgleich.fehltVerbindung') }}</p>

    <!-- Der Schalter „auf openany.de" je Projekt (E3, docs/plan-projekte-eine-art.md). -->
    <ProjekteAufServer v-if="lage?.gekoppelt" />

    <div v-if="bericht" class="mt-2 md:max-w-[460px] bg-flaeche border border-linie p-4 rounded-xl shadow-sm">
      <table class="text-sm text-fliess">
        <tbody>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.gezogen') }}</td><td class="py-0.5 tabular-nums">{{ bericht.gezogen }}</td></tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.geschoben') }}</td><td class="py-0.5 tabular-nums">{{ bericht.geschoben }}</td></tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.uebersprungen') }}</td><td class="py-0.5 tabular-nums">{{ bericht.uebersprungen }}</td></tr>
          <!--
            EINE ZAHL OHNE NAMEN IST KEINE AUSKUNFT: Der Läufer nennt je
            Grund, wie oft; hier steht er eingerückt darunter.
          -->
          <tr v-for="(anzahl, grund) in bericht.uebersprungen_weil ?? {}" :key="grund">
            <td class="pl-4 pr-4 py-0.5 text-leise">{{ grund }}</td>
            <td class="py-0.5 tabular-nums text-leise">{{ anzahl }}</td>
          </tr>
          <tr><td class="pr-4 py-0.5">{{ t('app.abgleich.konflikte') }}</td><td class="py-0.5 tabular-nums">{{ bericht.konflikte }}</td></tr>
          <!--
            DIE INHALTE GEHÖREN IN DEN BERICHT.

            Der Abgleich hat zwei Hälften: Erst zieht er die LISTE (was es
            gibt), dann holt er die BYTES (`Abgleichanzeige.inhalte`). Ein
            Bericht, der die zweite verschweigt, sagt „alles gut", wenn sie
            gerade nicht funktioniert.
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
  </div>
</template>
