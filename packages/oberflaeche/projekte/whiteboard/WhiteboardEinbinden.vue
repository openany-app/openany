<script setup>
// „Einbinden" im Whiteboard (docs/plan-whiteboard.md, W2): die Freigaben des
// Projekts, hineingehen, antippen -- dann ist es eingefügt und der Dialog zu.
//
// Was eine Freigabe enthält, liefert die Anwendung in einer Form für alle
// Arten (api.getWhiteboardFreigabe, in der App whiteboardFreigaben.js).
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  Loader2, ChevronLeft, ChevronRight, Folder, Image as ImageIcon, NotebookText, FileText,
} from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { api } from '../umgebung';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
});
// einfuegen: { verweis, bild? } -- bild ({ dataURL, mimeType }) ist schon da,
// wenn die Vorschau im Dialog geladen war.
const emit = defineEmits(['einfuegen', 'close']);
const { t } = useI18n();

const freigaben = ref(null);
const offen = ref(null); // die gewählte Freigabe
const stand = ref(null); // ihr Inhalt
const weg = ref([]); // [{ uuid, name }], die Wurzel zuerst
const fehler = ref('');
const vorschauen = ref({}); // uuid -> { dataURL, mimeType } | false

const ICONS = { album: ImageIcon, ordner: Folder, mappe: NotebookText };
const hier = computed(() => weg.value[weg.value.length - 1]);
const ebenen = computed(() => (stand.value?.ebenen ?? [])
  .filter((e) => e.eltern === hier.value?.uuid)
  .sort((a, b) => String(a.name).localeCompare(String(b.name), 'de')));
const eintraege = computed(() => (stand.value?.eintraege ?? [])
  .filter((e) => e.eltern === hier.value?.uuid)
  .sort((a, b) => String(a.name).localeCompare(String(b.name), 'de')));
const bilder = computed(() => eintraege.value.filter((e) => e.art === 'bild'));
const andere = computed(() => eintraege.value.filter((e) => e.art !== 'bild'));

onMounted(async () => {
  try {
    freigaben.value = (await api.getWhiteboardFreigaben(props.projectId)).data;
  } catch (e) {
    fehler.value = String(e);
  }
});

async function oeffnen(f) {
  offen.value = f;
  stand.value = null;
  fehler.value = '';
  try {
    stand.value = (await api.getWhiteboardFreigabe(props.projectId, f.uuid, f.art)).data;
    weg.value = [{ uuid: stand.value.wurzel, name: stand.value.name }];
    vorschauenLaden();
  } catch (e) {
    fehler.value = String(e);
  }
}

async function vorschauenLaden() {
  for (const b of bilder.value) {
    if (vorschauen.value[b.uuid] !== undefined) continue;
    vorschauen.value = { ...vorschauen.value, [b.uuid]: null };
    try {
      const bild = (await api.getWhiteboardBild(props.projectId, offen.value.uuid, b.uuid)).data;
      vorschauen.value = { ...vorschauen.value, [b.uuid]: bild };
    } catch {
      vorschauen.value = { ...vorschauen.value, [b.uuid]: false };
    }
  }
}

function hinein(e) {
  weg.value = [...weg.value, { uuid: e.uuid, name: e.name }];
  vorschauenLaden();
}

function zurueck() {
  if (weg.value.length > 1) {
    weg.value = weg.value.slice(0, -1);
  } else {
    offen.value = null;
    stand.value = null;
    fehler.value = '';
  }
}

function waehlen(e) {
  const verweis = { art: e.art, freigabe: offen.value.uuid, uuid: e.uuid, name: e.name, ...(e.mime ? { mime: e.mime } : {}) };
  emit('einfuegen', { verweis, bild: vorschauen.value[e.uuid] || null });
}
</script>

<template>
  <BaseModal :title="offen ? (hier?.name ?? offen.name) : t('planning.whiteboard.einbinden')" size="lg" @close="emit('close')">
    <div class="space-y-3">
      <button v-if="offen" type="button" class="flex items-center gap-1 text-sm font-bold text-marke cursor-pointer" @click="zurueck">
        <ChevronLeft class="w-4 h-4" /> {{ weg.length > 1 ? weg[weg.length - 2].name : t('planning.whiteboard.alleFreigaben') }}
      </button>

      <p v-if="fehler" class="text-sm text-rose-500 break-words">{{ fehler }}</p>

      <!-- Die Freigaben des Projekts -->
      <template v-else-if="!offen">
        <div v-if="freigaben === null" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
        <p v-else-if="!freigaben.length" class="text-sm text-leise">{{ t('planning.whiteboard.keineFreigaben') }}</p>
        <ul v-else class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="f in freigaben" :key="f.uuid">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer hover:bg-auflage" @click="oeffnen(f)">
              <component :is="ICONS[f.art] ?? Folder" class="w-5 h-5 shrink-0 text-marke" />
              <span class="flex-1 text-sm font-medium text-schrift truncate">{{ f.name }}</span>
              <ChevronRight class="w-4 h-4 text-leise" />
            </button>
          </li>
        </ul>
      </template>

      <!-- In einer Freigabe -->
      <div v-else-if="!stand" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <template v-else>
        <ul v-if="ebenen.length" class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="e in ebenen" :key="e.uuid">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer hover:bg-auflage" @click="hinein(e)">
              <component :is="offen.art === 'album' ? ImageIcon : Folder" class="w-5 h-5 shrink-0 text-marke" />
              <span class="flex-1 text-sm font-medium text-schrift truncate">{{ e.name }}</span>
              <ChevronRight class="w-4 h-4 text-leise" />
            </button>
          </li>
        </ul>

        <div v-if="bilder.length" class="grid grid-cols-3 sm:grid-cols-4 gap-2">
          <button v-for="b in bilder" :key="b.uuid" type="button" :title="b.name"
            class="aspect-square rounded-lg overflow-hidden bg-auflage flex items-center justify-center cursor-pointer hover:ring-2 hover:ring-marke"
            @click="waehlen(b)">
            <img v-if="vorschauen[b.uuid]" :src="vorschauen[b.uuid].dataURL" :alt="b.name" class="w-full h-full object-cover" />
            <ImageIcon v-else-if="vorschauen[b.uuid] === false" class="w-6 h-6 text-leise" />
            <Loader2 v-else class="w-4 h-4 animate-spin text-leise" />
          </button>
        </div>

        <ul v-if="andere.length" class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="e in andere" :key="e.uuid">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer hover:bg-auflage" @click="waehlen(e)">
              <NotebookText v-if="e.art === 'notiz'" class="w-5 h-5 shrink-0 text-marke" />
              <FileText v-else class="w-5 h-5 shrink-0 text-rose-500" />
              <span class="flex-1 text-sm font-medium text-schrift truncate">{{ e.name }}</span>
            </button>
          </li>
        </ul>

        <p v-if="!ebenen.length && !eintraege.length" class="text-sm text-leise">{{ t('planning.whiteboard.leer') }}</p>
      </template>
    </div>
  </BaseModal>
</template>
