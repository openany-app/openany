<script setup>
/*
 * Nachrichten im Programm — beide Wege, eine Liste.
 *
 * SEIT DEM 24.09.2026 KEINE EIGENE FLÄCHE MEHR. Verlauf, Schreibfeld und
 * Modul-Kopf stehen in `@oberflaeche/nachrichten`, denselben Dateien wie in
 * der Webapp. Vorher war das hier eine Abschrift von MessagesPanel.vue, und
 * die beiden liefen schon auseinander, ohne dass je etwas fehlschlug.
 *
 * ═══ WAS HIER BLEIBT, UND WARUM ═══
 *
 * DER UNTERSCHIED ZUR WEBAPP, und er wird nicht weggeräumt: Drüben meldet
 * sich openany.de am Matrix-Konto an und kann mitlesen, das sagt die
 * Einwilligung dort. HIER ist dieses Gerät selbst das Matrix-Gerät, mit
 * eigenen Schlüsseln; der Server sieht keine Zeile, und der Verlauf reist
 * nicht mit dem Abgleich — er liegt nur hier.
 *
 * Die Anmeldung bei Matrix und die E-Mail-Postfächer stehen seit dem
 * 29.09.2026 in den Einstellungen (`MatrixKachel`, `PostfachKachel`); hier
 * bleiben nur ihre Störungen und ein Hinweis, wenn noch nichts verbunden ist.
 *
 * Neue Nachrichten meldet die Schale über das Ereignis `openany-nachrichten`
 * (kein `listen`: das bräuchte eine Berechtigung, und eine fehlende schweigt).
 */
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2, AlertTriangle, Settings, ShieldAlert, Inbox } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import NachrichtenArbeitsflaeche from '@oberflaeche/nachrichten/NachrichtenArbeitsflaeche.vue';
import { nachrichtenQuelle as quelle, mailQuelle } from '../quellen/nachrichten';
import SpamDialog from './SpamDialog.vue';
import AnfragenDialog from './AnfragenDialog.vue';
import { useNahAbgleich } from '../nahAbgleich';

/*
 * DIE TÜR ZUM ADRESSBUCH FÜHRT ÜBER DIE SCHALE. Kontakte haben absichtlich
 * keinen Punkt in der Leiste (`navPunkt: false` in @oberflaeche/module): Die
 * Leiste trägt Orte, an die man zum Arbeiten geht, ein Adressbuch gehört
 * neben die Nachrichten. Die Fläche meldet nur die Absicht; wohin das führt,
 * weiss hier die Schale (App.vue) und drüben der Router.
 */
const emit = defineEmits(['oeffnen', 'start-verbraucht']);

/*
 * `start` IST DIE ABSICHT, MIT DER JEMAND HIERHER KAM: `{ kanal, kennung }`
 * aus dem Adressbuch. Drüben steht dasselbe als `?an=` in der Adresse.
 *
 * Wer von dort kommt, weiss schon, an wen — ihn den Namen abtippen zu
 * lassen, den er gerade angeklickt hat, wäre die Art von Umweg, wegen der
 * niemand das Adressbuch benutzt.
 */
const props = defineProps({
    start: { type: Object, default: null },
});

const { t } = useI18n();
const flaeche = ref(null);
const lage = ref(null);
// E-Mail (Schritt 2): verbunden wird in den Einstellungen, hier nur der Weg.
const mail = ref(null);
const postfaecher = computed(() => (mail.value?.postfaecher ?? []).map((p) => p.adresse));
const mailFehler = computed(() => (mail.value?.postfaecher ?? []).filter((p) => p.fehler));
// Spamverdacht: ein Hinweis je Postfach, in dem etwas liegt; der Inhalt
// öffnet sich als eigene Liste (SpamDialog), nicht im Verlauf.
const mitSpam = computed(() => (mail.value?.postfaecher ?? []).filter((p) => p.spam > 0));
const spamOffen = ref(null);
async function spamGeaendert() {
    await Promise.all([flaeche.value?.laden(), lageLesen()]);
}
// Anfragen vor Ort (06.10.2026): von Unbekannten, nicht im Verlauf.
const anfragen = ref([]);
const anfragenOffen = ref(false);
async function anfragenLesen() {
    try { anfragen.value = (await quelle.anfragen()).offen; } catch { anfragen.value = []; }
}
const matrixKonten = computed(() => (lage.value?.konten ?? []).map((k) => k.mxid));
const matrixFehler = computed(() => (lage.value?.konten ?? []).filter((k) => k.fehler));

async function lageLesen() {
    try {
        lage.value = await quelle.lage();
    } catch (e) {
        lage.value = { konten: [], mxid: null, fehler: String(e), ungelesen: 0 };
    }
    try { mail.value = await mailQuelle.lage(); } catch { mail.value = null; }
}

const neuGekommen = () => Promise.all([flaeche.value?.laden(), lageLesen(), anfragenLesen()]);

// Vor Ort: Der Zähler `direkt` aus `nah_lage` wächst, wenn eine
// Direktnachricht ankam oder aus dem Postausgang hinausging.
const { nah } = useNahAbgleich();
watch(() => nah.value?.direkt, (jetzt, vorher) => {
    if (vorher !== undefined && jetzt !== vorher) neuGekommen();
});

onMounted(async () => {
    await lageLesen();
    anfragenLesen();
    window.addEventListener('openany-nachrichten', neuGekommen);
    // Beim Öffnen gleich nach neuen Mails sehen -- im Hintergrund, der
    // Verlauf steht schon.
    if (postfaecher.value.length) {
        mailQuelle.abholen().then((neu) => neu && flaeche.value?.laden()).catch(() => lageLesen());
    }

    // Erst nach `lageLesen()`: Ob der Matrix-Weg ueberhaupt zur Wahl steht,
    // haengt am verbundenen Konto -- vorher liefe die Wahl ins Leere.
    if (props.start?.kennung) {
        await nextTick();
        flaeche.value?.oeffneSchreiben(props.start.kennung, props.start.kanal ?? '');
        emit('start-verbraucht');
    }
});
onUnmounted(() => window.removeEventListener('openany-nachrichten', neuGekommen));
</script>

<template>
  <NachrichtenArbeitsflaeche
    ref="flaeche"
    :data-source="quelle"
    :matrix-verbunden="Boolean(lage?.mxid)"
    :matrix-konten="matrixKonten"
    :mail-verbunden="postfaecher.length > 0"
    :mail-postfaecher="postfaecher"
    :bereit="Boolean(lage)"
    nah-verfuegbar
    :openany-verfuegbar="Boolean(lage?.server)"
    :weg-hinweise="{
      matrix: t('settings.messages.channelHintMatrixGeraet'),
      openany: t('settings.messages.channelHintOpenany'),
      email: t('settings.messages.channelHintEmail'),
      nah: t('settings.messages.channelHintNah'),
    }"
    @adressbuch="emit('oeffnen', 'contacts')"
    @neuer-kontakt="emit('oeffnen', 'contacts')"
    @geaendert="lageLesen"
  >
    <template #oben>
      <p v-for="p in mailFehler" :key="p.adresse" class="mb-4 text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
        <span class="min-w-0 break-words">{{ p.adresse }}: {{ p.fehler }}</span>
      </p>
      <p v-if="lage?.fehler" class="mb-4 text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
        <span class="min-w-0 break-words">Matrix: {{ lage.fehler }}</span>
      </p>
      <p v-for="k in matrixFehler" :key="k.mxid" class="mb-4 text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
        <span class="min-w-0 break-words">{{ k.mxid }}: {{ k.fehler }}</span>
      </p>
      <div v-if="mitSpam.length" class="mb-4 flex flex-wrap gap-2">
        <button v-for="p in mitSpam" :key="p.adresse" type="button"
                class="px-3 py-1.5 rounded-full border border-linie bg-auflage text-sm font-bold text-fliess flex items-center gap-1.5 cursor-pointer hover:border-marke"
                @click="spamOffen = p.adresse">
          <ShieldAlert class="w-4 h-4 text-warnung" />
          {{ t('app.spam.knopf') }}<template v-if="postfaecher.length > 1"> · {{ p.adresse }}</template> ({{ p.spam }})
        </button>
      </div>
      <SpamDialog v-if="spamOffen" :adresse="spamOffen" @close="spamOffen = null" @geaendert="spamGeaendert" />
      <div v-if="anfragen.length" class="mb-4 flex flex-wrap gap-2">
        <button type="button"
                class="px-3 py-1.5 rounded-full border border-linie bg-auflage text-sm font-bold text-fliess flex items-center gap-1.5 cursor-pointer hover:border-marke"
                @click="anfragenOffen = true">
          <Inbox class="w-4 h-4 text-warnung" />
          {{ t('app.anfragen.knopf') }} ({{ anfragen.length }})
        </button>
      </div>
      <AnfragenDialog v-if="anfragenOffen" @close="anfragenOffen = false; anfragenLesen()" @geaendert="neuGekommen" />
      <div v-if="!lage" class="py-20 flex justify-center">
        <Loader2 class="w-6 h-6 animate-spin text-leise" />
      </div>

      <!-- OHNE MATRIX UND POSTFACH IST DIE SEITE NICHT LEER. Der interne Weg
           braucht keines; der Hinweis zeigt nur, wo die anderen Wege
           eingerichtet werden. -->
      <div v-else-if="!lage.mxid && !postfaecher.length"
           class="mb-6 rounded-xl border border-linie bg-auflage p-4 flex flex-wrap items-center gap-3 text-sm text-fliess">
        <span class="min-w-0 flex-1">
          {{ t('app.nachrichten.wegeEinrichten') }}
        </span>
        <BaseButton variant="secondary" @click="emit('oeffnen', 'settings')">
          <Settings class="w-4 h-4" /> {{ t('app.nachrichten.einstellungen') }}
        </BaseButton>
      </div>
    </template>
  </NachrichtenArbeitsflaeche>
</template>
