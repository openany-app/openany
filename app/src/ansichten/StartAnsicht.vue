<script setup>
/*
 * Die Startseite — dieselben Kacheln wie in der Webapp (HomeDashboard aus dem
 * Paket): Termine dieser Woche, zuletzt bearbeitete Notizen, Suche im
 * Adressbuch. Welche davon stehen, wählt man in den Einstellungen.
 *
 * Ist nichts gewählt, stehen die Modul-Kacheln da — eine leere Startseite
 * wäre keine Auskunft.
 */
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import HomeDashboard from '@oberflaeche/start/HomeDashboard.vue';
import { useStartModule } from '../startModule';
import { lokaleStartQuelle } from '../quellen/start';

const props = defineProps({ kacheln: { type: Array, default: () => [] } });
const emit = defineEmits(['oeffnen']);

const { t } = useI18n();
const { auswahl } = useStartModule();
const quelle = lokaleStartQuelle();

// Reihenfolge der Registry, wie in der Webapp.
const moduleIds = computed(() => props.kacheln.map((k) => k.id).filter((id) => auswahl.value.includes(id)));

/*
 * Ein Ziel der Webapp (`/notes?note=…`, `{ path: '/contacts', query }`) in
 * einen Ort dieses Programms übersetzen. Mehr als der Ort geht verloren —
 * eine bestimmte Notiz öffnet die Arbeitsfläche hier noch nicht von außen.
 */
const ORTE = { '/calendar': 'calendar', '/notes': 'notes', '/contacts': 'contacts', '/files': 'files', '/messages': 'messages', '/projects': 'projects' };
const REITER = { dokumente: 'documents', dateien: 'files', mediathek: 'gallery' };
function oeffnen(ziel) {
    const adresse = typeof ziel === 'string' ? ziel : ziel?.path ?? '';
    const pfad = adresse.split(/[?#]/)[0];
    const reiter = REITER[adresse.split('#')[1]];
    // Ein bestimmter Hub (`/projects/<id>`): die Hubs, mit ihm geöffnet.
    const hub = pfad.match(/^\/projects\/([^/]+)$/);
    if (hub) emit('oeffnen', 'projects', { projekt: decodeURIComponent(hub[1]) });
    else if (ORTE[pfad]) emit('oeffnen', ORTE[pfad], reiter ? { tab: reiter } : null);
}
</script>

<template>
  <HomeDashboard v-if="moduleIds.length" :module-ids="moduleIds" :data-source="quelle" @oeffnen="oeffnen" />

  <div v-else class="grid gap-px p-[3px] sm:grid-cols-2 lg:grid-cols-3">
    <button
      v-for="k in kacheln"
      :key="k.id"
      class="karte p-4 text-left flex items-start gap-3 cursor-pointer hover:bg-auflage transition-colors"
      @click="emit('oeffnen', k.id)"
    >
      <span class="pille-chip w-10 h-10 flex items-center justify-center shrink-0 text-marke">
        <component :is="k.icon" class="w-5 h-5" />
      </span>
      <span class="min-w-0">
        <span class="block font-extrabold text-schrift">{{ t(k.label) }}</span>
        <span class="block text-sm text-leise mt-0.5">{{ t(k.description) }}</span>
      </span>
    </button>
  </div>
</template>
