<script setup>
/*
 * Mitglieder eines Projekts ohne Server (01.10.2026,
 * docs/konzept-lokale-mitgliedschaften.md): wer dabei ist, laut
 * unterschriebener Liste. Eingeladen wird vor Ort -- beide Geräte zeigen
 * dieselben 6 Ziffern (EinladungsDialog.vue). Eigene, gepaarte Geräte stehen
 * nicht in der Auswahl; sie gehören zur selben Person.
 *
 * Entfernen darf nur, wer Eigentümer ist; die anderen erfahren es beim
 * nächsten Abgleich. Austreten geht nur vor Ort: Der Austritt geht erst an
 * ein Mitglied in der Nähe, dann ist das Projekt hier fort.
 */
import { ref, computed, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Radar, AlertTriangle, X, Trash2 } from 'lucide-vue-next';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { useNahAbgleich } from '../../nahAbgleich';

const props = defineProps({ project: { type: Object, required: true } });
const emit = defineEmits(['weg']);
const toast = useToast();
const { t } = useI18n();
const { confirmDelete } = useConfirm();

const mitglieder = ref(null);
const laden = () => quelle.mitglieder(props.project.id).then((m) => { mitglieder.value = m; }).catch((e) => toast.error(String(e)));
onMounted(laden);

const { nah, suchen: nahSuchen, einladen: nahEinladen } = useNahAbgleich();
const einladenOffen = ref(false);
const binEigentuemer = computed(() => Boolean(mitglieder.value?.mitglieder?.some((m) => m.ich && m.rolle === 'owner')));
const einladbar = computed(() => (nah.value?.geraete ?? []).filter((g) => g.openany && !g.gepaart));

async function einladen(fingerabdruck) {
  try {
    await nahEinladen(props.project.id, fingerabdruck);
    einladenOffen.value = false;
  } catch (e) {
    toast.error(String(e));
  }
}
// Ist eine Einladung abgeschlossen (sie verschwindet), die Liste neu lesen.
watch(() => nah.value?.einladungen?.length ?? 0, (jetzt, vorher) => {
  if (jetzt < vorher) laden();
});

async function entfernen(m) {
  if (!(await confirmDelete(t('app.projekte.entfernenFrage', { name: m.name || t('app.projekte.diesesMitglied'), projekt: props.project.name })))) return;
  try {
    await quelle.mitgliedEntfernen(props.project.id, m.id);
    await laden();
  } catch (e) {
    toast.error(String(e));
  }
}

async function austreten() {
  if (!(await confirmDelete(t('app.projekte.austretenFrage', { projekt: props.project.name })))) return;
  try {
    const bei = await quelle.austreten(props.project.id);
    toast.success(t('app.projekte.ausgetreten', { bei }));
    emit('weg');
  } catch (e) {
    toast.error(String(e));
  }
}

// Wer nicht Eigentümer ist, löscht nur die eigene Kopie. (Der Eigentümer
// löscht oben in der Titelzeile.)
async function hierLoeschen() {
  if (!(await confirmDelete(t('app.projekte.hierLoeschenFrage', { projekt: props.project.name })))) return;
  try {
    await quelle.lokalLoeschen(props.project.id);
    emit('weg');
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div class="space-y-3 md:max-w-[720px]">
    <div v-if="mitglieder === null" class="flex justify-center py-6">
      <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-marke"></div>
    </div>
    <p v-else-if="mitglieder.fehler" class="text-sm text-rose-600 dark:text-rose-400">
      {{ t('app.projekte.listeUngueltig', { fehler: mitglieder.fehler }) }}
    </p>
    <ul v-else class="karte shadow-sm divide-y divide-linie">
      <li v-for="m in mitglieder.mitglieder" :key="m.id" class="p-3 flex items-center gap-2 text-sm">
        <span class="font-bold text-schrift">{{ m.name || t('app.projekte.ohneNamen') }}</span>
        <span v-if="m.ich" class="text-leise">{{ t('app.projekte.du') }}</span>
        <span class="ml-auto text-xs text-leise">{{ m.rolle === 'owner' ? t('app.projekte.eigentuemer') : t('app.projekte.mitglied') }}</span>
        <button v-if="m.entfernbar" type="button" class="text-xs font-bold text-rose-600 dark:text-rose-400 cursor-pointer"
                @click="entfernen(m)">{{ t('app.allgemein.entfernen') }}</button>
      </li>
    </ul>

    <button v-if="binEigentuemer" type="button" class="knopf-wichtig flex items-center gap-1.5" @click="einladenOffen = true; nahSuchen()">
      <Radar class="w-4 h-4" /> {{ t('app.projekte.einladen') }}
    </button>
    <p v-if="mitglieder?.einziges_geraet" class="text-sm text-amber-700 dark:text-amber-400 flex items-start gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
      <span>{{ t('app.projekte.einzigesGeraet') }}</span>
    </p>

    <button v-if="mitglieder?.kann_austreten" type="button" class="flex items-center gap-1.5 text-sm font-bold text-rose-600 dark:text-rose-400 cursor-pointer" @click="austreten">
      <X class="w-4 h-4" /> {{ t('app.projekte.austreten') }}
    </button>
    <button v-if="project.my_role !== 'owner'" type="button" class="flex items-center gap-1.5 text-sm font-bold text-rose-600 dark:text-rose-400 cursor-pointer" @click="hierLoeschen">
      <Trash2 class="w-4 h-4" /> {{ t('app.projekte.hierLoeschen') }}
    </button>

    <!-- Wen einladen? Geräte in der Nähe, ohne die eigenen. -->
    <BaseModal v-if="einladenOffen" :title="t('app.projekte.einladen')" @close="einladenOffen = false">
      <div class="space-y-3">
        <p class="text-sm text-leise">{{ t('app.projekte.einladenHinweis') }}</p>
        <p v-if="!einladbar.length" class="text-sm text-fliess">{{ t('app.projekte.niemand') }}</p>
        <ul v-else class="karte divide-y divide-linie">
          <li v-for="g in einladbar" :key="g.fingerabdruck" class="p-3 flex items-center gap-3">
            <span class="flex-1 min-w-0">
              <span class="block font-bold text-schrift truncate">{{ g.name }}</span>
              <span class="block text-xs text-leise font-mono">{{ g.fingerabdruck.slice(0, 12) }}</span>
            </span>
            <BaseButton class="shrink-0" @click="einladen(g.fingerabdruck)">{{ t('app.projekte.einladenKnopf') }}</BaseButton>
          </li>
        </ul>
        <BaseButton variant="secondary" @click="nahSuchen()">{{ t('app.projekte.suchen') }}</BaseButton>
      </div>
    </BaseModal>
  </div>
</template>
