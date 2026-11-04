<script setup>
/*
 * Speicher — Dateien und Dokumente, dieselbe Arbeitsfläche wie in der Webapp,
 * mit der lokalen SQLite und der Inhaltsablage dahinter (Phase 3, Stufe A).
 *
 * Die Reiter wie drüben (`frontend/src/views/Files.vue`): Galerie · Dateien ·
 * Dokumente. Die Galerie seit Stufe C.
 *
 * ÖFFNEN: Bilder, Text, Ton und Video zeigt die Webansicht selbst. Was sie
 * nicht kann (PDF, Office …), geht an ein anderes Programm — auf Android über
 * die Auswahl des Systems, am Schreibtisch an das Standardprogramm. Im
 * Betrachter steht „Mit anderem Programm öffnen" auch für alles andere.
 */
import { ref, watch, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Cloud, CloudOff, FileText, Folder as FolderIcon, Image as ImageIcon, Pin, PinOff } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import DateiArbeitsflaeche from '@oberflaeche/speicher/DateiArbeitsflaeche.vue';
import GalerieArbeitsflaeche from '@oberflaeche/galerie/GalerieArbeitsflaeche.vue';
import TexterkennungDialoge from '@oberflaeche/texterkennung/TexterkennungDialoge.vue';
import { useTexterkennungImSpeicher } from '@oberflaeche/texterkennung/useTexterkennungImSpeicher';
import { useSpeicherStart } from '@oberflaeche/composables/useSpeicherStart';
import { lokaleGalerieQuelle } from '../quellen/galerie';
import { lokaleDateienQuelle, dateiAlsBlob, dateiExternOeffnen, dateiHolen, behaltenSetzen, aufServerSetzen } from '../quellen/dateien';

const emit = defineEmits(['oeffnen', 'start-verbraucht']);
// Wer von einer Projekt-Freigabe kommt, bringt den Reiter mit (`{ tab }`).
const props = defineProps({ start: { type: Object, default: null } });
const { t } = useI18n();

const TABS = [
    { id: 'gallery', icon: ImageIcon, label: () => t('albums.albums.title') },
    { id: 'files', icon: FolderIcon, label: () => t('files.title') },
    { id: 'documents', icon: FileText, label: () => t('documents.title') },
];
// Der Reiter aus den Einstellungen (nur auf diesem Gerät; Vorgabe: Dokumente).
const aktiv = ref(props.start?.tab ?? useSpeicherStart().speicherStart.value);
// Gleich ein Dokument öffnen (`{ tab, ordner, datei }`, siehe DateiArbeitsflaeche).
const startDatei = ref(props.start?.datei ? { ordner: props.start.ordner ?? null, datei: props.start.datei } : null);
if (props.start?.tab) emit('start-verbraucht');
// Kommt der Sprung, während der Speicher schon offen ist, ändert sich nur
// `start` -- die Ansicht baut sich nicht neu.
watch(() => props.start, (s) => {
    if (!s?.tab) return;
    aktiv.value = s.tab;
    if (s.datei) startDatei.value = { ordner: s.ordner ?? null, datei: s.datei };
    emit('start-verbraucht');
});

const betrachter = ref(null); // { datei, url, art } | { datei, fehler }

function zeigbar(datei) {
    const mime = (datei.mime || '').toLowerCase();
    if (mime.startsWith('image/')) return 'bild';
    if (mime.startsWith('video/')) return 'video';
    if (mime.startsWith('audio/')) return 'ton';
    if (mime.startsWith('text/') || mime === 'application/json') return 'text';
    return null;
}

function schliessen() {
    if (betrachter.value?.url) URL.revokeObjectURL(betrachter.value.url);
    betrachter.value = null;
}

async function extern(datei) {
    try {
        if (!(await dateiExternOeffnen(datei))) {
            betrachter.value = { datei, fehler: t('app.speicherAnsicht.externFehler') };
        }
    } catch (e) {
        betrachter.value = { datei, fehler: String(e) };
    }
}

async function oeffnen(datei) {
    schliessen();
    // Liegt der Inhalt auf einem anderen Gerät („bei Bedarf"), erst holen.
    if (datei.vorhanden === false) {
        betrachter.value = { datei, laedt: true };
        try {
            await dateiHolen(datei);
            datei.vorhanden = true;
        } catch (e) {
            betrachter.value = { datei, fehler: String(e) };
            return;
        }
        betrachter.value = null;
    }
    const art = zeigbar(datei);
    if (!art) {
        await extern(datei);
        return;
    }
    try {
        const blob = await dateiAlsBlob(datei);
        betrachter.value = art === 'text'
            ? { datei, art, text: await blob.text() }
            : { datei, art, url: URL.createObjectURL(blob) };
    } catch (e) {
        betrachter.value = { datei, fehler: String(e) };
    }
}

const quelle = lokaleDateienQuelle(oeffnen);
const galerie = lokaleGalerieQuelle();

/*
 * „Auf diesem Gerät behalten" (Speicherregel „ausgewählt"): an Ordnern und
 * Alben. Wer markiert, bekommt die fehlenden Inhalte gleich im Hintergrund.
 */
function behaltenAktion(eintrag, neuLaden) {
    const an = !eintrag.behalten;
    return {
        key: 'behalten',
        icon: an ? Pin : PinOff,
        title: an ? t('app.speicherAnsicht.behalten') : t('app.speicherAnsicht.nichtBehalten'),
        onClick: async () => { await behaltenSetzen(eintrag.id, an); neuLaden(); },
    };
}

/*
 * „Auf den Server": an Fotos, Dateien und ganzen Alben, Akten und Ordnern. Was
 * in einem Behälter auf dem Server liegt, folgt dem Behälter und hat keinen
 * eigenen Schalter.
 */
const toast = useToast();
const verbunden = ref(false);
invoke('speicher_lage').then((l) => { verbunden.value = l.verbunden; }).catch(() => {});
function serverAktion(eintrag, neuLaden) {
    if (!verbunden.value || eintrag.auf_server_geerbt) return [];
    // Was in keinem Behälter liegt, bleibt auf diesem Gerät.
    if (eintrag.type && eintrag.type !== 'folder' && !eintrag.parent_id) return [];
    const an = !eintrag.auf_server;
    return [{
        key: 'server',
        icon: an ? Cloud : CloudOff,
        title: an ? t('app.speicherAnsicht.aufServer') : t('app.speicherAnsicht.vomServer'),
        onClick: async () => {
            try {
                const r = await aufServerSetzen(eintrag.id, an);
                if (r.einzig_dort) toast.info(t('app.speicherAnsicht.einzigDort', { anzahl: r.einzig_dort }));
            } catch (e) {
                toast.error(String(e));
            }
            neuLaden();
        },
    }];
}
const dateiErweiterungen = {
    aktionen: (item, kontext) => [
        ...(item.type === 'folder' ? [behaltenAktion(item, kontext.neuLaden)] : []),
        ...serverAktion(item, kontext.neuLaden),
    ],
};

/*
 * TEXTERKENNUNG IN AKTEN -- dieselbe wie in der Webapp, aus dem Paket. Fotos kommen als
 * durchsuchbares PDF an, eingescannte PDFs bekommen „Text erkennen". Die
 * Sprachdaten liegen im Programm (public/tesseract), also auch ohne Netz.
 */
const texterkennung = useTexterkennungImSpeicher(quelle, 'documents');
const aktenErweiterungen = {
    ...texterkennung.erweiterung,
    aktionen: (item, kontext) => [
        ...dateiErweiterungen.aktionen(item, kontext),
        ...texterkennung.erweiterung.aktionen(item, kontext),
    ],
};
const albumAktionen = (album, neuLaden) => [behaltenAktion(album, neuLaden), ...serverAktion(album, neuLaden)];
const bildAktion = (bild, neuLaden) => {
    const aktion = serverAktion(bild, neuLaden)[0];
    // In der Leiste der großen Ansicht steht sie in einem schmalen Feld.
    return aktion ? { ...aktion, title: t(bild.auf_server ? 'app.speicherAnsicht.vomServerKurz' : 'app.speicherAnsicht.aufServerKurz') } : null;
};

onBeforeUnmount(schliessen);
</script>

<template>
  <ModulePage>
    <ModuleHeader :icon="FolderIcon">
      <div class="flex items-center gap-1 sm:gap-1.5 min-w-0">
        <button
          v-for="tab in TABS"
          :key="tab.id"
          @click="aktiv = tab.id"
          :title="tab.label()"
          class="flex items-center justify-center h-9 gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-2 rounded-xl text-sm font-bold transition-all cursor-pointer min-w-0"
          :class="aktiv === tab.id ? 'bg-marke text-white shadow-md shadow-marke/30' : 'text-fliess hover:bg-auflage'"
        >
          <component :is="tab.icon" class="w-4 h-4 shrink-0" />
          <span class="truncate">{{ tab.label() }}</span>
        </button>
      </div>
    </ModuleHeader>

    <!-- v-show statt v-if: Ordner-Position bleibt beim Reiterwechsel. -->
    <section v-show="aktiv === 'gallery'">
      <GalerieArbeitsflaeche :data-source="galerie" lokal :album-aktionen="albumAktionen" :bild-aktion="bildAktion" />
    </section>
    <section v-show="aktiv === 'files'">
      <DateiArbeitsflaeche zone="files" lokal :data-source="quelle" :erweiterungen="dateiErweiterungen" @hilfe="emit('oeffnen', 'help')" />
    </section>
    <section v-show="aktiv === 'documents'">
      <DateiArbeitsflaeche zone="documents" lokal :data-source="quelle" :erweiterungen="aktenErweiterungen" :oeffnen="startDatei" @geoeffnet="startDatei = null" @hilfe="emit('oeffnen', 'help')" />
    </section>

    <TexterkennungDialoge :texterkennung="texterkennung" />

    <BaseModal v-if="betrachter" :title="betrachter.datei.name" size="lg" @close="schliessen">
      <p v-if="betrachter.laedt" class="text-sm text-fliess">{{ t('app.speicherAnsicht.wirdGeholt') }}</p>
      <p v-else-if="betrachter.fehler" class="text-sm text-fliess">{{ betrachter.fehler }}</p>
      <img v-else-if="betrachter.art === 'bild'" :src="betrachter.url" :alt="betrachter.datei.name" class="max-w-full max-h-[70vh] mx-auto rounded-lg" />
      <video v-else-if="betrachter.art === 'video'" :src="betrachter.url" controls class="max-w-full max-h-[70vh] mx-auto"></video>
      <audio v-else-if="betrachter.art === 'ton'" :src="betrachter.url" controls class="w-full"></audio>
      <pre v-else-if="betrachter.art === 'text'" class="text-sm text-fliess whitespace-pre-wrap break-words max-h-[70vh] overflow-auto">{{ betrachter.text }}</pre>
      <template v-if="!betrachter.laedt" #footer>
        <button class="knopf" @click="extern(betrachter.datei)">{{ t('app.speicherAnsicht.extern') }}</button>
      </template>
    </BaseModal>
  </ModulePage>
</template>
