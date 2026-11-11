<script setup>
/*
 * openany als Teilen-Ziel (docs/plan-viele-dateien.md, Skizze C): ein
 * kleines Blatt „80 Fotos nach …“ mit der Albumliste und „Neues Album“.
 * Dokumente gehen nach Dokumente, alles andere nach Dateien (Frage 8).
 * Danach geht die App wieder zu; die Schlange läuft im Dienst weiter.
 *
 * Erscheint, sobald etwas bereitliegt -- beim Start (die App wurde zum
 * Teilen geöffnet) und auf `openany-teilen` (sie lief schon). Schließen
 * (Kreuz, Zurück) verwirft nichts: Das Blatt kommt beim nächsten Öffnen
 * wieder. Fort ist es nur mit „Verwerfen“.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { geteiltQuelle } from '../quellen/uebernahme';
import { lokaleGalerieQuelle } from '../quellen/galerie';

const { t } = useI18n();
const toast = useToast();
const quelle = geteiltQuelle();
const galerie = lokaleGalerieQuelle();

const stand = ref(null);
const alben = ref([]);
// '' = oberste Ebene der Mediathek, 'neu' = neues Album, sonst die uuid.
const ziel = ref('');
const neuerName = ref('');
const schickt = ref(false);

const gesamt = computed(() => (stand.value ? stand.value.medien + stand.value.dokumente + stand.value.sonst : 0));

async function nachsehen() {
  if (!quelle) return;
  try {
    const s = await quelle.stand();
    const n = s.medien + s.dokumente + s.sonst;
    if (!n) { stand.value = null; return; }
    if (s.medien) alben.value = (await galerie.getAlbums()).data.items ?? [];
    stand.value = s;
  } catch {
    stand.value = null;
  }
}

async function uebernehmen() {
  schickt.value = true;
  try {
    const neu = ziel.value === 'neu' ? neuerName.value.trim() : null;
    const album = ziel.value && ziel.value !== 'neu' ? ziel.value : null;
    await quelle.uebernehmen(album, neu);
    stand.value = null;
    toast.success(t('app.geteilt.eingereiht', { n: gesamt.value }, gesamt.value));
    quelle.fertig();
  } catch (e) {
    toast.error(typeof e === 'string' ? e : t('app.geteilt.fehler'));
  } finally {
    schickt.value = false;
  }
}

async function verwerfen() {
  await quelle.verwerfen().catch(() => {});
  stand.value = null;
}

let abmelden = null;
onMounted(() => {
  if (!quelle) return;
  abmelden = quelle.beobachten(nachsehen);
  nachsehen();
});
onUnmounted(() => abmelden?.());
</script>

<template>
  <BaseModal v-if="stand" :title="t('app.geteilt.titel', { n: gesamt }, gesamt)" @close="stand = null">
    <div class="space-y-3 text-sm">
      <template v-if="stand.medien">
        <p class="font-semibold text-schrift">{{ t('app.geteilt.medienNach', { n: stand.medien }, stand.medien) }}</p>
        <div class="space-y-2 max-h-[40vh] overflow-y-auto">
          <label class="flex items-center gap-3">
            <input v-model="ziel" type="radio" value="" />
            <span class="text-schrift">{{ t('app.geteilt.ohneAlbum') }}</span>
          </label>
          <label class="flex items-center gap-3">
            <input v-model="ziel" type="radio" value="neu" />
            <span class="text-schrift">{{ t('app.geteilt.neuesAlbum') }}</span>
          </label>
          <input v-if="ziel === 'neu'" v-model="neuerName" type="text" class="feld w-full" :placeholder="t('app.geteilt.albumName')" />
          <label v-for="a in alben" :key="a.id" class="flex items-center gap-3">
            <input v-model="ziel" type="radio" :value="a.id" />
            <span class="text-schrift break-all">{{ a.name }}</span>
          </label>
        </div>
      </template>
      <p v-if="stand.dokumente" class="text-fliess">{{ t('app.geteilt.dokumente', { n: stand.dokumente }, stand.dokumente) }}</p>
      <p v-if="stand.sonst" class="text-fliess">{{ t('app.geteilt.sonst', { n: stand.sonst }, stand.sonst) }}</p>
    </div>
    <template #footer>
      <BaseButton variant="secondary" @click="verwerfen">{{ t('app.geteilt.verwerfen') }}</BaseButton>
      <BaseButton :disabled="ziel === 'neu' && !neuerName.trim()" :loading="schickt" @click="uebernehmen">{{ t('app.geteilt.uebernehmen') }}</BaseButton>
    </template>
  </BaseModal>
</template>
