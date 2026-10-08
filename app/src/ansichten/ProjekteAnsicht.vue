<script setup>
/*
 * Projekte -- seit dem 02.10.2026 mit der Projektseite der Webapp
 * (packages/oberflaeche/projekte/ProjektSeite.vue). Ihr `api` beantwortet
 * die App aus dem lokalen Speicher (quellen/projektApi.js); was nur ohne
 * Server geht (Mitglieder vor Ort, Chat vor Ort, Freigaben vor Ort), setzt
 * sie in die Reiter ein (./projekt/). Die alte Ansicht liegt im Tag
 * `app-projekte-alt-2026-10-01`.
 *
 * Server-Projekte: Planung und Freigaben wie gehabt; Chat und Mitglieder
 * gibt es (noch) nur in der Webapp -- die Reiter fehlen dann (`ohne`), statt
 * nur auf die Webapp zu verweisen (local-first, 02.10.2026).
 */
import { ref, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Briefcase, ArrowDownUp, Plus, Radar, Crown, Loader2, AlertTriangle } from 'lucide-vue-next';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import ProjektSeite from '@oberflaeche/projekte/ProjektSeite.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../quellen/projekte';
import LokaleMitglieder from './projekt/LokaleMitglieder.vue';
import LokalerChat from './projekt/LokalerChat.vue';
import LokaleInhalte from './projekt/LokaleInhalte.vue';
import ServerInhalte from './projekt/ServerInhalte.vue';
import { useNahAbgleich } from '../nahAbgleich';

const toast = useToast();
const { t } = useI18n();
const emit = defineEmits(['oeffnen']);

const projekte = ref(null);
const offenes = ref(null); // Eintrag aus projekte_liste
// Nach einem Abgleich baut sich die Seite neu auf (frische Zähler, Listen).
const stand = ref(0);
const laeuft = ref(false);
// Lokale Projekte: bin ich (noch) Mitglied?
const binMitglied = ref(true);
const seite = ref(null);

// Kam eine Chat-Nachricht vor Ort an, zieht der Zähler am Reiter nach.
const { nah } = useNahAbgleich();
watch(() => nah.value?.chat, (jetzt, vorher) => {
  if (vorher !== undefined && jetzt !== vorher) seite.value?.zaehlerAktualisieren({ still: true });
});

async function laden() {
  try {
    projekte.value = await quelle.liste();
  } catch (e) {
    projekte.value = [];
    toast.error(String(e));
  }
}
onMounted(laden);

function oeffnen(p) {
  offenes.value = p;
  binMitglied.value = true;
  if (p.lokal) quelle.mitglieder(p.id).then((m) => { binMitglied.value = Boolean(m.fehler || m.bin_mitglied); }).catch(() => {});
}

async function zurueck() {
  offenes.value = null;
  await laden();
}

/*
 * LOKALE PROJEKTE (01.10.2026, docs/konzept-lokale-mitgliedschaften.md):
 * ohne Konto, nur auf diesem Gerät, mit einer unterschriebenen
 * Mitgliederliste. Eingeladen wird vor Ort.
 */
const neuOffen = ref(false);
const neuName = ref('');
const legtAn = ref(false);
async function anlegen() {
  legtAn.value = true;
  try {
    const id = await quelle.lokalAnlegen(neuName.value);
    neuOffen.value = false;
    neuName.value = '';
    await laden();
    const neu = projekte.value.find((p) => p.id === id);
    if (neu) oeffnen(neu);
  } catch (e) {
    toast.error(String(e));
  } finally {
    legtAn.value = false;
  }
}

async function abgleichen() {
  laeuft.value = true;
  try {
    // Server-Projekte -- ohne Kopplung gibt es da nichts, und das ist kein
    // Grund, die lokalen auszulassen.
    const lokaleDa = (projekte.value ?? []).some((p) => p.lokal);
    try {
      for (const lauf of await quelle.abgleichen()) {
        if (lauf.fehler?.length) toast.error(t('app.projekte.laufFehler', { name: lauf.name, fehler: lauf.fehler[0] }));
        else toast.success(t('app.projekte.laufServer', { name: lauf.name, geholt: lauf.gezogen, geschickt: lauf.geschoben }));
      }
    } catch (e) {
      if (!lokaleDa) toast.error(String(e));
    }
    // Lokale Projekte mit Mitgliedern in der Nähe.
    if (lokaleDa) {
      const laeufe = await quelle.nahAbgleichen();
      for (const l of laeufe) {
        if (l.fehler) toast.error(t('app.projekte.laufNahFehler', { name: l.name, mit: l.mit, fehler: l.fehler }));
        else if (l.entfernt) toast.info(t('app.projekte.laufEntfernt', { name: l.name }));
        else {
          const was = [t('app.projekte.notizen', l.notizen)];
          if (l.chat) was.push(t('app.projekte.nachrichten', l.chat));
          if (l.planung) was.push(t('app.projekte.planung', l.planung));
          if (l.neue_mitglieder) was.push(t('app.projekte.neueMitglieder', l.neue_mitglieder));
          toast.success(t('app.projekte.laufNah', { name: l.name, mit: l.mit, was: was.join(', ') }));
        }
      }
      if (!laeufe.length) toast.info(t('app.projekte.niemandNah'));
    }
    await laden();
    if (offenes.value) {
      const noch = projekte.value.find((p) => p.id === offenes.value.id);
      if (noch) oeffnen(noch);
      else offenes.value = null;
    }
    stand.value += 1;
  } catch (e) {
    toast.error(String(e));
  } finally {
    laeuft.value = false;
  }
}
</script>

<template>
  <ProjektSeite v-if="offenes" ref="seite" :key="`${offenes.id}:${stand}`" :project-id="offenes.id"
                :ohne="offenes.lokal ? ['umbenennen'] : ['umbenennen', 'loeschen', 'chat', 'members']" @zurueck="zurueck">
    <template #kopf>
      <button type="button" :disabled="laeuft" :title="t('app.projekte.abgleichen')" @click="abgleichen"
        class="flex items-center justify-center h-8 gap-1.5 px-2.5 py-1.5 bg-marke text-white text-sm font-bold rounded-xl transition-colors disabled:opacity-50">
        <ArrowDownUp class="w-4 h-4" :class="laeuft ? 'animate-pulse' : ''" />
      </button>
    </template>

    <template #hinweis>
      <div v-if="offenes.lokal" class="my-3 space-y-2 md:max-w-[720px]">
        <p class="text-sm text-leise flex items-start gap-2">
          <Radar class="w-4 h-4 shrink-0 mt-0.5 text-marke" />
          <span>{{ t('app.projekte.nurVorOrtHinweis') }}</span>
        </p>
        <p v-if="!binMitglied"
           class="p-3 rounded-xl border border-rose-300 dark:border-rose-800 text-sm text-rose-700 dark:text-rose-300 flex items-start gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
          <span>{{ t('app.projekte.nichtMehrMitglied') }}</span>
        </p>
      </div>
      <div v-else class="mt-3" />
    </template>

    <!-- Bei Server-Projekten blendet `ohne` diese Reiter aus. -->
    <template #chat="{ project }">
      <LokalerChat :project="project" />
    </template>

    <template #mitglieder="{ project }">
      <LokaleMitglieder :project="project" @weg="zurueck" />
    </template>

    <template #inhalte="{ project }">
      <LokaleInhalte v-if="project.lokal" :project="project" />
      <ServerInhalte v-else :project="project" @oeffnen="(...a) => emit('oeffnen', ...a)" />
    </template>
  </ProjektSeite>

  <ModulePage v-else>
    <ModuleHeader :icon="Briefcase">
      <BaseButton variant="secondary" groesse="kopf" class="shrink-0" :disabled="laeuft" @click="abgleichen">
        <ArrowDownUp class="w-4 h-4" :class="laeuft ? 'animate-pulse' : ''" />
        <span>{{ t('app.projekte.abgleichen') }}</span>
      </BaseButton>
      <BaseButton groesse="kopf" class="shrink-0" @click="neuOffen = true">
        <Plus class="w-4 h-4" />
        <span>{{ t('app.projekte.projekt') }}</span>
      </BaseButton>
    </ModuleHeader>

    <div v-if="projekte === null" class="py-20 flex justify-center">
      <Loader2 class="w-6 h-6 animate-spin text-leise" />
    </div>

    <!-- Noch nichts da: Das ist kein Fehler, sondern der Zustand vor dem
         ersten Abgleich. -->
    <div v-else-if="!projekte.length" class="karte shadow-sm py-16 px-6 text-center space-y-3">
      <div class="w-16 h-16 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
        <Briefcase class="w-8 h-8" />
      </div>
      <p class="text-sm text-fliess">{{ t('app.projekte.keine') }}</p>
      <p class="text-sm text-leise max-w-sm mx-auto">{{ t('app.projekte.keineHinweis') }}</p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      <button
        v-for="p in projekte" :key="p.id" type="button"
        class="text-left karte shadow-sm p-6 space-y-3 hover:border-marke hover:shadow-md transition-all cursor-pointer"
        @click="oeffnen(p)"
      >
        <div class="flex items-start justify-between gap-3">
          <h3 class="font-extrabold text-lg text-schrift truncate">{{ p.name }}</h3>
          <span v-if="p.rolle === 'owner'"
            class="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
          ><Crown class="w-3 h-3" /> {{ t('app.projekte.eigentuemer') }}</span>
        </div>
        <div class="flex items-center gap-4 text-xs font-bold text-slate-400 dark:text-slate-500 pt-1">
          <span v-if="p.lokal" class="flex items-center gap-1 text-marke"><Radar class="w-3.5 h-3.5" /> {{ t('app.projekte.nurVorOrt') }}</span>
          <span v-else>openany.de</span>
          <span>{{ t('app.projekte.inPlanung', { anzahl: p.boards + p.roadmaps + p.ortsgruppen + p.abstimmungen }) }}</span>
        </div>
      </button>
    </div>

    <!-- Ein lokales Projekt anlegen -->
    <BaseModal v-if="neuOffen" :title="t('app.projekte.neuTitel')" @close="neuOffen = false">
      <form class="space-y-3" @submit.prevent="anlegen">
        <label class="block">
          <span class="block text-xs font-bold text-leise mb-1">{{ t('app.projekte.name') }}</span>
          <input v-model="neuName" class="feld" :placeholder="t('app.projekte.namePlatzhalter')" autocorrect="off" required>
        </label>
        <p class="text-sm text-leise">{{ t('app.projekte.neuHinweis') }}</p>
        <div class="flex justify-end gap-2">
          <BaseButton variant="secondary" type="button" @click="neuOffen = false">{{ t('common.cancel') }}</BaseButton>
          <BaseButton type="submit" :disabled="legtAn || !neuName.trim()">{{ t('app.projekte.anlegen') }}</BaseButton>
        </div>
      </form>
    </BaseModal>
  </ModulePage>
</template>
