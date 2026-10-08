<script setup>
/*
 * Eine FREMDE Freigabe im Projekt ansehen (Weg 1, 30.09.2026): Ordner und
 * Akten, Alben, Notiz-Mappen, die ein anderes Mitglied freigegeben hat.
 *
 * NUR LESEN, UND NUR MIT NETZ. Der Inhalt kommt beim Öffnen vom Server --
 * dieselben Antworten, die die Webapp zeigt. Bearbeiten bleibt der Webapp
 * (Tiffy, 30.09.2026). „Übernehmen" legt eine Kopie in den EIGENEN Speicher,
 * einsortiert unter „Aus Projekten".
 *
 * Notiz-Mappen zeigen vorerst nur die Titel: Eine einzelne fremde Notiz liest
 * der Server nur über einen Weg, den Geräteschlüssel nicht haben.
 */
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { Loader2, AlertTriangle, Folder, FileText, Image as ImageIcon, ChevronLeft, Download, NotebookText } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import PdfBetrachter from '@oberflaeche/speicher/PdfBetrachter.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../quellen/projekte';
import { einsortieren, mimeAus } from '../quellen/ablage';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps({
    projekt: { type: String, required: true }, // uuid
    freigabe: { type: Object, required: true }, // FreigabeAnzeige
});
const emit = defineEmits(['close']);
const toast = useToast();

// Der Weg hinein: [{ art, id, name }], die Freigabe selbst zuerst.
const weg = ref([{ art: props.freigabe.art, id: props.freigabe.id, name: props.freigabe.name }]);
const hier = computed(() => weg.value[weg.value.length - 1]);
const inhalt = ref(null);
const fehler = ref('');
const vorschauen = ref({}); // Bild-id -> Blob-URL, oder false, wenn sie nicht kam
const bild = ref(null); // { name, url }
const pdf = ref(null); // { name, pfad }
const arbeitet = ref(null);

async function laden() {
    inhalt.value = null;
    fehler.value = '';
    try {
        inhalt.value = await quelle.freigabeInhalt(props.projekt, hier.value.art, hier.value.id);
        if (hier.value.art === 'album') vorschauenLaden();
    } catch (e) {
        fehler.value = String(e);
    }
}

async function vorschauenLaden() {
    for (const m of inhalt.value?.media ?? []) {
        if (vorschauen.value[m.id]) continue;
        try {
            const bytes = await quelle.freigabeDatei(m.thumb_url);
            vorschauen.value = { ...vorschauen.value, [m.id]: URL.createObjectURL(new Blob([bytes])) };
        } catch {
            // Ohne Vorschau ein Symbol statt eines Rads, das ewig dreht.
            vorschauen.value = { ...vorschauen.value, [m.id]: false };
        }
    }
}

function hinein(art, id, name) {
    weg.value = [...weg.value, { art, id, name }];
    laden();
}

function zurueck() {
    weg.value = weg.value.slice(0, -1);
    laden();
}

// Die Einträge eines Ordners: Unterordner zuerst, wie der Server sie liefert.
const dateien = computed(() => inhalt.value?.files ?? []);
const dateiPfad = (d) => `/api/projects/${props.projekt}/files/${d.id}/download`;

async function oeffnen(d) {
    const mime = mimeAus(d.name);
    if (mime === 'application/pdf') {
        pdf.value = { name: d.name, pfad: dateiPfad(d) };
    } else if (mime.startsWith('image/')) {
        arbeitet.value = d.id;
        try {
            const bytes = await quelle.freigabeDatei(dateiPfad(d));
            bild.value = { name: d.name, url: URL.createObjectURL(new Blob([bytes], { type: mime })) };
        } catch (e) {
            toast.error(String(e));
        } finally {
            arbeitet.value = null;
        }
    } else {
        await uebernehmen(d.name, dateiPfad(d));
    }
}

async function uebernehmen(name, pfad) {
    arbeitet.value = pfad;
    try {
        const bytes = await quelle.freigabeDatei(pfad);
        const ort = await einsortieren(new File([bytes], name, { type: mimeAus(name) }), 'Aus Projekten');
        toast.success(`Übernommen: ${ort}`);
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
    }
}

async function bildOeffnen(m) {
    arbeitet.value = m.id;
    try {
        const bytes = await quelle.freigabeDatei(m.url);
        bild.value = { name: m.name, url: URL.createObjectURL(new Blob([bytes], { type: mimeAus(m.name) })), download: m.download_url };
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
    }
}

function bildSchliessen() {
    if (bild.value?.url) URL.revokeObjectURL(bild.value.url);
    bild.value = null;
}

onMounted(laden);
onBeforeUnmount(() => {
    Object.values(vorschauen.value).forEach((u) => URL.revokeObjectURL(u));
    bildSchliessen();
});
</script>

<template>
  <BaseModal :title="hier.name" size="lg" @close="emit('close')">
    <div class="space-y-3">
      <p class="text-xs text-leise">
        {{ t('app.freigabe.von', { name: freigabe.von || t('app.freigabe.vonMitglied') }) }}
      </p>
      <button v-if="weg.length > 1" type="button" class="flex items-center gap-1 text-sm font-bold text-marke cursor-pointer" @click="zurueck">
        <ChevronLeft class="w-4 h-4" /> {{ weg[weg.length - 2].name }}
      </button>

      <div v-if="!inhalt && !fehler" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <p v-else-if="fehler" class="text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
        <span class="min-w-0 break-words">{{ t('app.freigabe.nichtGeladen', { fehler }) }}</span>
      </p>

      <!-- Ordner und Akten -->
      <template v-else-if="hier.art === 'file_folder'">
        <p v-if="!dateien.length" class="text-sm text-fliess">{{ t('app.freigabe.leer') }}</p>
        <ul v-else class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="d in dateien" :key="d.id" class="flex items-center gap-3 p-3">
            <button type="button" class="flex-1 min-w-0 flex items-center gap-3 text-left cursor-pointer"
                    @click="d.type === 'folder' ? hinein('file_folder', d.id, d.name) : oeffnen(d)">
              <Folder v-if="d.type === 'folder'" class="w-5 h-5 shrink-0 text-marke" />
              <ImageIcon v-else-if="mimeAus(d.name).startsWith('image/')" class="w-5 h-5 shrink-0 text-sky-500" />
              <FileText v-else class="w-5 h-5 shrink-0 text-rose-500" />
              <span class="min-w-0">
                <span class="block text-sm font-medium text-schrift truncate">{{ d.name }}</span>
                <span v-if="d.type !== 'folder'" class="block text-xs text-leise">{{ d.size }}</span>
              </span>
            </button>
            <Loader2 v-if="arbeitet === d.id || arbeitet === dateiPfad(d)" class="w-4 h-4 animate-spin text-leise" />
            <button v-else-if="d.type !== 'folder'" type="button" class="text-leise hover:text-marke cursor-pointer"
                    :title="t('app.freigabe.uebernehmenTitel')" :aria-label="t('app.freigabe.uebernehmenTitel')"
                    @click="uebernehmen(d.name, dateiPfad(d))">
              <Download class="w-4 h-4" />
            </button>
          </li>
        </ul>
      </template>

      <!-- Alben -->
      <template v-else-if="hier.art === 'album'">
        <ul v-if="inhalt.children?.length" class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="c in inhalt.children" :key="c.id">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer" @click="hinein('album', c.id, c.name)">
              <ImageIcon class="w-5 h-5 shrink-0 text-marke" />
              <span class="text-sm font-medium text-schrift">{{ c.name }}</span>
              <span class="ml-auto text-xs text-leise">{{ c.media_count }}</span>
            </button>
          </li>
        </ul>
        <p v-if="!inhalt.media?.length && !inhalt.children?.length" class="text-sm text-fliess">{{ t('app.freigabe.leer') }}</p>
        <div class="grid grid-cols-3 sm:grid-cols-4 gap-2">
          <button v-for="m in inhalt.media" :key="m.id" type="button"
                  class="aspect-square rounded-lg overflow-hidden bg-auflage flex items-center justify-center cursor-pointer"
                  :title="m.name" @click="bildOeffnen(m)">
            <img v-if="vorschauen[m.id]" :src="vorschauen[m.id]" :alt="m.name" class="w-full h-full object-cover" />
            <ImageIcon v-else-if="vorschauen[m.id] === false" class="w-6 h-6 text-leise" />
            <Loader2 v-else class="w-4 h-4 animate-spin text-leise" />
          </button>
        </div>
      </template>

      <!-- Notiz-Mappen: vorerst die Titel -->
      <template v-else-if="hier.art === 'note_folder'">
        <ul class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="f in inhalt.subfolders ?? []" :key="`m${f.id}`">
            <button type="button" class="w-full flex items-center gap-3 p-3 text-left cursor-pointer" @click="hinein('note_folder', f.id, f.name)">
              <Folder class="w-5 h-5 shrink-0 text-marke" /><span class="text-sm font-medium text-schrift">{{ f.name }}</span>
            </button>
          </li>
          <li v-for="n in inhalt.notes ?? []" :key="`n${n.id}`" class="flex items-center gap-3 p-3">
            <NotebookText class="w-5 h-5 shrink-0 text-leise" /><span class="text-sm text-schrift">{{ n.title }}</span>
          </li>
        </ul>
        <p class="text-xs text-leise">{{ t('app.freigabe.notizenNurWebapp') }}</p>
      </template>
    </div>

    <!-- Ein Bild groß -->
    <div v-if="bild" class="fixed inset-0 z-[110] bg-black/90 flex flex-col" @click.self="bildSchliessen">
      <!-- Unter die Statusleiste rutscht die Zeile sonst (edge-to-edge). -->
      <div class="flex items-center justify-between gap-2 p-3 pt-[max(0.75rem,env(safe-area-inset-top))] mt-6 text-white">
        <span class="text-sm truncate">{{ bild.name }}</span>
        <div class="flex gap-4 text-sm font-bold">
          <button v-if="bild.download" type="button" class="cursor-pointer" @click="uebernehmen(bild.name, bild.download)">{{ t('app.freigabe.uebernehmen') }}</button>
          <button type="button" class="cursor-pointer" @click="bildSchliessen">{{ t('common.close') }}</button>
        </div>
      </div>
      <img :src="bild.url" :alt="bild.name" class="flex-1 min-h-0 object-contain" />
    </div>

    <PdfBetrachter v-if="pdf" :name="pdf.name" :laden="() => quelle.freigabeDatei(pdf.pfad)" @close="pdf = null" />
  </BaseModal>
</template>
