<script setup>
/*
 * Die Schale des Programms — aus denselben Teilen wie die Webapp.
 *
 * SEIT DEM 15.09.2026 KEINE ABSCHRIFT MEHR. Zeichen, Kopfzeile, Leiste,
 * Menü-Blatt, Hell/Dunkel und die Modul-Liste kommen aus
 * `packages/oberflaeche` — denselben Dateien, die die Webapp benutzt. Die
 * Abschrift vom 06.09.2026 war neun Tage später sichtbar veraltet (die
 * Wortmarke neben dem Logo, die alten Farben), ohne dass irgendetwas
 * fehlschlug.
 *
 * DIE LEISTE TRÄGT ORTE, DAS MENÜ-BLATT TRÄGT WERKZEUGE — wie drüben.
 *
 * ═══ WO DIESES PROGRAMM ABWEICHT, UND WARUM ═══
 *
 * Kein Router: Die Schale bekommt Einträge ohne `to` und meldet `waehlen`.
 *
 * Der Puls heißt „der letzte Abgleich lief durch", nicht „der Browser ist
 * online". Drei Zustände, und der erste ist kein Fehler:
 *
 *   keine Gegenstelle  ruhig, kein Streifen  — allein arbeiten ist gewählt
 *   verbunden          Zeichen pulsiert      — der letzte Lauf lief durch
 *   eingerichtet+Fehler Streifen „offline"   — der Stand ist womöglich alt
 */
import { ref, computed, watch, onMounted, h } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { CloudOff, Mail, Settings as SettingsIcon, Trash2, LifeBuoy, RefreshCw } from 'lucide-vue-next';

import KopfZeile from '@oberflaeche/schale/KopfZeile.vue';
import LeisteUnten from '@oberflaeche/schale/LeisteUnten.vue';
import MenueBlatt from '@oberflaeche/schale/MenueBlatt.vue';
import ToastHost from '@oberflaeche/base/ToastHost.vue';
import ConfirmHost from '@oberflaeche/base/ConfirmHost.vue';
import { useHelligkeit } from '@oberflaeche/composables/useHelligkeit';
import { useDesign } from '@oberflaeche/composables/useDesign';
import { useToast } from '@oberflaeche/composables/useToast';

import { ANSICHTEN, ORTE, KACHELN } from './module';
import StartAnsicht from './ansichten/StartAnsicht.vue';
import EinstellungenAnsicht from './ansichten/EinstellungenAnsicht.vue';
import NochNicht from './ansichten/NochNicht.vue';
import PapierkorbAnsicht from './ansichten/PapierkorbAnsicht.vue';
import HilfeSeite from '@oberflaeche/hilfe/HilfeSeite.vue';
// Die Nummer DIESER Plattform (tauri.android.conf.json usw.) -- jede zählt
// für sich, deshalb fragt die Seite Tauri und liest keine Datei.
import { getVersion } from '@tauri-apps/api/app';
import PaarungsDialog from './PaarungsDialog.vue';
import EinladungsDialog from './EinladungsDialog.vue';
import { useNahAbgleich } from './nahAbgleich';

// Die Hilfe zeigt hier, was nur das Programm kann (`nur: 'app'` in der
// Gliederung).
const appVersion = ref('');
getVersion().then((v) => { appVersion.value = v; }).catch(() => {});
const HilfeImProgramm = () => h(HilfeSeite, { programm: true, version: appVersion.value });

const { t } = useI18n();
const helligkeit = useHelligkeit(t);
// Aufrufen genügt: Der Import setzt die Design-Klasse am <html>-Element.
useDesign();

const offen = ref('start');
/*
 * `gepaeck` IST DAS FRAGEZEICHEN DER WEBAPP.
 *
 * Drüben öffnet `/messages?an=hannah` das Schreibfeld mit vorgemerktem
 * Empfänger; hier gibt es keine Route, an die sich so etwas hängen liesse.
 * Statt dessen reist die Absicht als zweites Argument von `gehZu` mit.
 *
 * SIE WIRD VERBRAUCHT UND DANN WEGGERÄUMT (`start-verbraucht`) — sonst
 * klappte dasselbe Schreibfeld bei jeder Rückkehr zu den Nachrichten erneut
 * auf, für eine Nachricht, die längst geschrieben ist. Drüben löst
 * `router.replace` dasselbe Problem.
 */
const gepaeck = ref(null);
const nahAbgleich = useNahAbgleich();
const toast = useToast();
const menueOffen = ref(false);
const lage = ref(null);

/*
 * UNGELESENE NACHRICHTEN im Menü und an der Leiste (Tiffy, 02.10.2026: der
 * Zähler fehlte in der App). Die Zahl kommt aus `nachrichten_lage` --
 * Matrix, E-Mail und „Vor Ort", was auf diesem Gerät liegt. Nachgezogen,
 * wenn etwas ankam (Ereignis `openany-nachrichten`, Zähler `direkt` aus
 * `nah_lage`), beim Ortswechsel und sonst alle 15 s.
 */
const ungelesen = ref(0);
async function ungeleseneLesen() {
    try {
        ungelesen.value = (await invoke('nachrichten_lage'))?.ungelesen ?? 0;
    } catch {
        // bleibt beim letzten Stand
    }
}
watch(() => nahAbgleich.nah.value?.direkt, (jetzt, vorher) => {
    if (vorher !== undefined && jetzt !== vorher) ungeleseneLesen();
});

const orte = computed(() => ORTE.map((m) => ({ id: m.id, label: t(m.label), icon: m.icon })));

// Dieselben Werkzeuge wie im Blatt der Webapp, in derselben Reihenfolge.
// Abmelden gibt es nicht, weil es kein Konto gibt, von dem man sich abmelden
// könnte.
const werkzeuge = computed(() => [
    ...(kannAbgleichen.value ? [{ id: 'sync', label: laeuftAbgleich.value ? t('app.schale.gleichtAb') : t('app.schale.abgleichen'), icon: RefreshCw }] : []),
    { id: 'messages', label: t('shell.header.messages'), icon: Mail, zaehler: ungelesen.value || null },
    { id: 'settings', label: t('shell.header.settings'), icon: SettingsIcon },
    { id: 'help', label: t('shell.footer.help'), icon: LifeBuoy },
    { id: 'trash', label: t('common.trash'), icon: Trash2 },
]);

const helligkeitEintrag = computed(() => ({
    id: 'theme', label: helligkeit.name.value, icon: helligkeit.symbol.value,
}));

/** Was in der Mitte steht. */
const ansicht = computed(() => {
    if (offen.value === 'start') return { bau: StartAnsicht };
    if (offen.value === 'settings') return { bau: EinstellungenAnsicht, titel: t('shell.header.settings') };
    if (offen.value === 'trash') return { bau: PapierkorbAnsicht };
    if (offen.value === 'help') return { bau: HilfeImProgramm };

    const eintrag = ANSICHTEN[offen.value];
    const modul = [...ORTE, ...KACHELN].find((m) => m.id === offen.value);
    const titel = offen.value === 'messages' ? t('shell.header.messages') : modul ? t(modul.label) : '';

    return eintrag ? { ...eintrag, titel } : { bau: NochNicht, titel };
});

/** Steht die Verbindung? Ohne Gegenstelle ist die Frage nicht gestellt. */
const verbunden = computed(() =>
    Boolean(lage.value?.verbunden && lage.value?.gekoppelt && !lage.value?.letzter_fehler
        && lage.value?.letzter_lauf));

/** Eingerichtet, aber der letzte Lauf kam nicht durch. */
const offline = computed(() => Boolean(lage.value?.verbunden) && !verbunden.value);

/*
 * „ABGLEICHEN" HEISST ABGLEICHEN -- mit allem, was dieses Gerät kennt.
 *
 * Bis zum 21.09.2026 traf dieser Knopf nur Geräte in der Nähe. Wer eine
 * Instanz eingetragen hatte und darauf drückte, bekam „Noch kein Gerät
 * gepaart" zu sehen, während der Abgleich mit openany.de daneben in den
 * Einstellungen lag und nie lief. Am Tablet fiel genau das auf: hochgeladen
 * in beiden, abgeglichen scheinbar nirgends.
 *
 * Also erst der Server, dann die Geräte -- und was nicht geht, sagt warum.
 */
const serverLaeuft = ref(false);
const kannAbgleichen = computed(() =>
    Boolean(lage.value?.verbunden && lage.value?.gekoppelt) || Boolean(nahAbgleich.nah.value?.gepaarte?.length));
const laeuftAbgleich = computed(() => serverLaeuft.value || Boolean(nahAbgleich.laeuft.value));

/** Ein Abgleich-Bericht als ein Satz, für die Meldung unten. */
function berichtSatz(wer, b) {
    const teile = [t('app.schale.lauf', { wer, geholt: b.gezogen, geschickt: b.geschoben })];
    if (b.konflikte) teile.push(t('app.schale.konflikte', b.konflikte));
    const i = b.inhalte;
    if (i && (i.geholt || i.zu_wenig_platz)) {
        teile.push(t('app.schale.dateienGeladen', i.geholt));
        if (i.zu_wenig_platz) teile.push(t('app.schale.passenNicht', { count: i.zu_wenig_platz }));
    }
    return teile.join(', ');
}

async function mitServerAbgleichen() {
    if (!lage.value?.verbunden || !lage.value?.gekoppelt) return;
    serverLaeuft.value = true;
    try {
        const b = await invoke('abgleichen');
        if (b.fehler?.length) toast.error(t('app.schale.laufFehler', { wer: 'openany.de', fehler: b.fehler[0] }));
        else toast.success(berichtSatz('openany.de', b));
        if (b.gezogen || b.konflikte || b.inhalte?.geholt) neuAufbauen();
        lageLesen();
    } catch (e) {
        toast.error(t('app.schale.laufFehler', { wer: 'openany.de', fehler: String(e) }));
    } finally {
        serverLaeuft.value = false;
    }
}

async function abgleichenAlle() {
    if (laeuftAbgleich.value) return;

    const gepaart = nahAbgleich.erreichbar.value.length;

    if (!kannAbgleichen.value) {
        toast.info(t('app.schale.nichtsZumAbgleichen'));
        return;
    }

    await mitServerAbgleichen();

    // Ein gepaartes Gerät, das gerade nicht da ist, ist kein Fehler -- aber
    // auch kein Grund zu schweigen: Der Versuch sagt es zuverlässiger als die
    // Liste (siehe nahAbgleich.erreichbar).
    if (!gepaart) return;

    for (const e of await nahAbgleich.abgleichenAlle()) {
        const fehler = e.fehler ?? e.bericht.fehler[0];
        if (fehler) {
            toast.error(t('app.schale.laufFehler', { wer: e.name, fehler }));
        } else {
            toast.success(berichtSatz(e.name, e.bericht));
        }
    }
}

/*
 * NACH EINEM ABGLEICH NEU LESEN. Die Ansichten laden beim Einhängen; also
 * wird die offene Ansicht neu eingehängt, wenn sich hier etwas geändert hat.
 *
 * Nicht, während jemand tippt: Der Neuaufbau nähme den Cursor, und die
 * Notiz speicherte beim Aushängen noch einmal über das eben Geholte. Dann
 * wartet es, bis das Feld den Fokus verliert.
 */
const neuAufbau = ref(0);
function schreibtGerade() {
    const el = document.activeElement;
    return Boolean(el && (el.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName)));
}
function neuAufbauen() {
    if (!schreibtGerade()) {
        neuAufbau.value++;
        return;
    }
    document.addEventListener('focusout', () => setTimeout(neuAufbauen, 50), { once: true });
}
watch(nahAbgleich.stand, neuAufbauen);

/*
 * DER AUFFRISCHER hat im Hintergrund abgeglichen (Android, WorkManager),
 * während dieses Fenster offen war. Dann gilt dasselbe wie nach einem
 * Abgleich von Hand: Lage neu lesen, die offene Ansicht neu aufbauen --
 * sonst zeigte sie einen Stand, der eben überholt wurde.
 */
function aufgefrischt() {
    lageLesen();
    neuAufbauen();
}

async function lageLesen() {
    lage.value = await invoke('lage');
}

/*
 * ZURÜCK MUSS ZURÜCK GEHEN, NICHT DAS PROGRAMM SCHLIESSEN.
 *
 * Android gibt die Zurück-Taste an die Webansicht; die geht im Verlauf zurück,
 * und ist dort nichts, schließt sie die App. Ohne Router entstand kein Verlauf
 * — am 15.09.2026 auf dem Tablet: Menü auf, Zurück, App zu. Deshalb legt jeder
 * Ortswechsel einen Eintrag an, und ein offenes Menü einen eigenen, damit
 * Zurück zuerst das Menü schließt.
 */
function gehZu(id, mitgabe = null) {
    gepaeck.value = mitgabe;

    if (id === 'sync') {
        if (menueOffen.value) history.back();
        abgleichenAlle();
        return;
    }
    if (menueOffen.value) {
        menueOffen.value = false;
        history.replaceState({ offen: id }, '');
    } else if (id !== offen.value) {
        history.pushState({ offen: id }, '');
    }
    offen.value = id;
}

function menueUmschalten() {
    if (menueOffen.value) {
        history.back();
    } else {
        menueOffen.value = true;
        history.pushState({ offen: offen.value, menue: true }, '');
    }
}

function zurueck(e) {
    // Auf einem Dialog-Eintrag gelandet, ohne dass ein Dialog offen ist: ein
    // Überbleibsel (Seite gewechselt, während der Dialog offen war). Einfach
    // darüber hinweg – sonst bräuchte Zurück hier einen zweiten Druck.
    if (e.state?.dialog) {
        if (!document.querySelector('[role="dialog"]')) history.back();
        return;
    }
    menueOffen.value = Boolean(e.state?.menue);
    offen.value = e.state?.offen ?? 'start';
}

onMounted(() => {
    helligkeit.anwenden();
    lageLesen();
    nahAbgleich.starten();
    history.replaceState({ offen: offen.value }, '');
    window.addEventListener('popstate', zurueck);
    window.addEventListener('openany-aufgefrischt', aufgefrischt);
    window.addEventListener('openany-nachrichten', ungeleseneLesen);
    ungeleseneLesen();
    setInterval(ungeleseneLesen, 15000);
});
watch(() => offen.value, ungeleseneLesen);
</script>

<template>
  <!-- Unten Platz für die Leiste, die um den Geräterand wächst — nur dort, wo
       es sie gibt (unter `md`). -->
  <div class="min-h-screen flex flex-col font-sans transition-colors duration-300 chat-bg pb-[calc(4rem+var(--rand-unten))] md:pb-0"
       :style="{ paddingTop: 'var(--rand-oben)', paddingLeft: 'var(--rand-links)', paddingRight: 'var(--rand-rechts)' }">

    <!-- Unter der Statusleiste: eine feste Fläche, damit gescrollter Inhalt
         nicht hinter Uhr und Akku durchläuft. -->
    <div class="fixed top-0 inset-x-0 z-[55] bg-flaeche transition-colors duration-300"
         :style="{ height: 'var(--rand-oben)' }"></div>

    <!-- Offline: nur dann. Der Streifen ist eine Auskunft, kein Dauerzustand. -->
    <div v-if="offline" class="sticky z-50" :style="{ top: 'var(--rand-oben)' }">
      <div class="flex items-center justify-center gap-2 px-4 py-2 bg-amber-500 text-white text-sm font-bold">
        <CloudOff class="w-4 h-4 shrink-0" />
        <span>{{ t('app.schale.offline') }}</span>
      </div>
    </div>

    <KopfZeile
      :orte="orte"
      :aktiv="offen"
      :verbunden="verbunden"
      :start-label="t('shell.header.home')"
      @start="gehZu('start')"
      @waehlen="gehZu"
    >
      <template #rechts>
        <button
          @click="helligkeit.weiterschalten()"
          class="pille flex items-center gap-2 px-3 py-1.5 cursor-pointer"
          :title="t('shell.header.themeLabel', { mode: helligkeit.name.value })"
        >
          <span class="pille-chip w-6 h-6 flex items-center justify-center">
            <component :is="helligkeit.symbol.value" class="w-3.5 h-3.5" />
          </span>
          <span class="text-sm font-semibold hidden lg:inline">{{ helligkeit.name.value }}</span>
        </button>
        <button
          v-for="w in werkzeuge"
          :key="w.id"
          @click="gehZu(w.id)"
          :title="w.label"
          class="pille flex items-center justify-center w-9 h-9 cursor-pointer"
          :class="offen === w.id ? 'ring-2 ring-marke' : ''"
        >
          <component :is="w.icon" class="w-4 h-4" :class="w.id === 'sync' && nahAbgleich.laeuft.value ? 'animate-spin' : ''" />
        </button>
      </template>
    </KopfZeile>

    <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-5">
      <component
        :is="ansicht.bau"
        :key="offen + ':' + neuAufbau"
        :titel="ansicht.titel"
        :grund="ansicht.grund"
        :lage="lage"
        :kacheln="KACHELN"
        :start="gepaeck"
        @lage-geaendert="lageLesen"
        @oeffnen="gehZu"
        @start-verbraucht="gepaeck = null"
      />
    </main>

    <MenueBlatt
      v-if="menueOffen"
      :eintraege="werkzeuge"
      :helligkeit="helligkeitEintrag"
      :verbunden="verbunden"
      @schliessen="menueUmschalten"
      @marke="gehZu('start')"
      @eintrag="gehZu"
      @helligkeit="helligkeit.weiterschalten()"
    />

    <LeisteUnten
      :orte="orte"
      :aktiv="offen"
      :menue-offen="menueOffen"
      :menue-label="t('shell.footer.menu')"
      :zaehler="ungelesen"
      @menue="menueUmschalten"
      @waehlen="gehZu"
    />

    <PaarungsDialog />
    <EinladungsDialog />
    <ToastHost />
    <ConfirmHost />
  </div>
</template>
