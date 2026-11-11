<script setup>
/*
 * Neue Fotos automatisch übernehmen (docs/plan-viele-dateien.md, D5):
 * openany als Foto-Sicherung. Was die Kamera neu aufnimmt, kommt von selbst
 * in die Mediathek -- aus den gewählten Gerätealben in gleichnamige Alben,
 * ab dem Einschalten.
 *
 * Ehrlich in drei Sätzen: Android drosselt Arbeit im Hintergrund (Verzug),
 * auf openany.de zählt es zum Speicher, und neben Google Fotos laden sonst
 * beide hoch.
 *
 * `quelle.fotosVomGeraet`: erlaubt, erlauben, beimErlauben, uebersicht,
 * automatischStand, automatischSetzen.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from 'lucide-vue-next';
import BaseModal from '../base/BaseModal.vue';
import BaseButton from '../base/BaseButton.vue';
import { useToast } from '../composables/useToast';

const props = defineProps({
  quelle: { type: Object, required: true },
  // Aus den Einstellungen mit dem Schalter geöffnet: gleich auf „an“.
  einschalten: { type: Boolean, default: false },
});
const emit = defineEmits(['close']);

const { t } = useI18n();
const toast = useToast();
const geraet = props.quelle.fotosVomGeraet;

const erlaubt = ref(geraet.erlaubt());
const lage = ref(null);
const alben = ref([]);
const an = ref(false);
const gewaehlt = ref(new Set());
const ohneKopie = ref(true);
const nurWlan = ref(false);
const nurLaden = ref(false);
const schickt = ref(false);

async function laden() {
  erlaubt.value = geraet.erlaubt();
  if (erlaubt.value === 'keine') return;
  try {
    const [l, u] = await Promise.all([geraet.automatischStand(), geraet.uebersicht()]);
    lage.value = l;
    alben.value = u.alben.map((a) => a.name).filter(Boolean);
    an.value = l.an || props.einschalten;
    nurWlan.value = l.nur_wlan;
    nurLaden.value = l.nur_laden;
    ohneKopie.value = l.verbunden ? (l.ohne_kopie ?? true) : false;
    // Vorgabe: die Kamera.
    gewaehlt.value = new Set(l.an ? l.alben : alben.value.filter((n) => /camera|kamera/i.test(n)));
  } catch (e) {
    toast.error(typeof e === 'string' ? e : t('albums.vomGeraet.fehler'));
  }
}

const bereit = computed(() => lage.value && !schickt.value && (!an.value || gewaehlt.value.size > 0));

function umschalten(name) {
  const s = new Set(gewaehlt.value);
  if (s.has(name)) s.delete(name); else s.add(name);
  gewaehlt.value = s;
}

async function speichern() {
  schickt.value = true;
  try {
    await geraet.automatischSetzen({
      an: an.value,
      alben: [...gewaehlt.value],
      nurWlan: nurWlan.value,
      nurLaden: nurLaden.value,
      ohneKopie: ohneKopie.value,
    });
    toast.success(t(an.value ? 'albums.automatisch.eingeschaltet' : 'albums.automatisch.ausgeschaltet'));
    emit('close');
  } catch (e) {
    toast.error(typeof e === 'string' ? e : t('albums.vomGeraet.fehler'));
  } finally {
    schickt.value = false;
  }
}

let abmelden = null;
onMounted(() => {
  abmelden = geraet.beimErlauben(() => laden());
  laden();
});
onUnmounted(() => abmelden?.());
</script>

<template>
  <BaseModal :title="t('albums.automatisch.titel')" size="lg" @close="emit('close')">
    <div v-if="erlaubt === 'keine'" class="space-y-4 text-sm text-fliess">
      <p>{{ t('albums.vomGeraet.zugriffText') }}</p>
      <BaseButton @click="geraet.erlauben()">{{ t('albums.vomGeraet.zugriff') }}</BaseButton>
    </div>

    <div v-else-if="!lage" class="flex justify-center py-10">
      <Loader2 class="w-8 h-8 text-marke animate-spin" />
    </div>

    <div v-else class="space-y-3 text-sm">
      <label class="flex items-center gap-3">
        <input v-model="an" type="checkbox" />
        <span class="font-semibold text-schrift">{{ t('albums.automatisch.an') }}</span>
      </label>

      <template v-if="an">
        <p class="text-fliess">{{ t('albums.automatisch.ausAlben') }}</p>
        <ul class="ml-7 space-y-2 max-h-[30vh] overflow-y-auto">
          <li v-for="name in alben" :key="name">
            <label class="flex items-center gap-3">
              <input type="checkbox" :checked="gewaehlt.has(name)" @change="umschalten(name)" />
              <span class="text-schrift break-all">{{ name }}</span>
            </label>
          </li>
        </ul>

        <div v-if="lage.verbunden" class="space-y-2">
          <label class="flex items-center gap-3">
            <input v-model="ohneKopie" type="radio" :value="true" />
            <span class="text-schrift">{{ t('albums.vomGeraet.wahlServer') }}</span>
          </label>
          <label class="flex items-center gap-3">
            <input v-model="ohneKopie" type="radio" :value="false" />
            <span class="text-schrift">{{ t('albums.vomGeraet.wahlKopie') }}</span>
          </label>
        </div>

        <label class="flex items-center gap-3">
          <input v-model="nurWlan" type="checkbox" />
          <span class="text-schrift">{{ t('albums.automatisch.nurWlan') }}</span>
        </label>
        <label class="flex items-center gap-3">
          <input v-model="nurLaden" type="checkbox" />
          <span class="text-schrift">{{ t('albums.automatisch.nurLaden') }}</span>
        </label>

        <ul class="pt-3 border-t border-slate-100 dark:border-slate-800 space-y-1 text-leise list-disc pl-5">
          <li>{{ t('albums.automatisch.verzug') }}</li>
          <li v-if="ohneKopie && lage.verbunden">{{ t('albums.automatisch.speicher') }}</li>
          <li>{{ t('albums.automatisch.google') }}</li>
        </ul>
      </template>
    </div>

    <template #footer>
      <BaseButton variant="secondary" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton v-if="erlaubt !== 'keine'" :disabled="!bereit" :loading="schickt" @click="speichern">{{ t('common.save') }}</BaseButton>
    </template>
  </BaseModal>
</template>
