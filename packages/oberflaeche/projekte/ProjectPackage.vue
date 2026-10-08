<script setup>
// Der Paket-Rundlauf eines Projekts: als ZIP herunterladen, in Zettlr
// bearbeiten, wieder einlesen.
//
// Das Einlesen läuft ausdrücklich in ZWEI Schritten. Der erste rechnet nur
// und zeigt, was geschähe; erst der zweite schreibt. Der Grund steht im
// Backend (PackageImport::preview): Für manche Fälle taugt keine Regel – ein
// Bild liegt im Paket, hier wurde es gelöscht. Wieder anlegen kann genauso
// falsch sein wie es wegzulassen, und nur der weiß es, der gelöscht hat.
//
// Deshalb ist die Liste zwischen den Schritten kein Bericht zum Abnicken,
// sondern die eigentliche Arbeit: Wo etwas zu entscheiden ist, steht dort
// eine Auswahl. Alles andere läuft stumm durch – bei vierhundert
// unveränderten Notizen will niemand vierhundert Häkchen setzen.
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from './umgebung';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { downloadBlob, sanitizeFilename } from '@oberflaeche/shared/download';
import { useToast } from '@oberflaeche/composables/useToast';
import {
  Download, Upload, FilePlus2, FilePen, FileCheck, FileWarning,
  FileQuestion, RotateCcw, Ban,
} from 'lucide-vue-next';

const props = defineProps({
  projectId: { type: [Number, String], required: true },
  projectName: { type: String, default: '' },
  // Ohne Schreibrechte auf mindestens einer Freigabe lehnt das Backend das
  // Zurückspielen ab – dann gar nicht erst anbieten.
  canWrite: { type: Boolean, default: false },
});

const { t } = useI18n();
const toast = useToast();

const isExporting = ref(false);
const isUploading = ref(false);
const isApplying = ref(false);
const fileInput = ref(null);

// Ergebnis der Vorschau: Kennung, Zeilen, Zusammenfassung.
const preview = ref(null);
// Antworten je Zeile: Schlüssel -> Wahl. Vorbelegt mit der Vorgabe.
const answers = ref({});

const exportPackage = async () => {
  isExporting.value = true;
  try {
    const res = await api.exportProjectPackage(props.projectId);
    downloadBlob(new Blob([res.data]), `${sanitizeFilename(props.projectName, 'projekt')}.zip`);
  } catch {
    toast.error(t('shares.package.exportFailed'));
  } finally {
    isExporting.value = false;
  }
};

const pickFile = () => fileInput.value?.click();

const onFilePicked = async (event) => {
  const file = event.target.files?.[0];
  // Zurücksetzen, sonst löst dieselbe Datei ein zweites Mal kein change aus.
  event.target.value = '';
  if (!file) return;

  isUploading.value = true;
  try {
    const res = await api.previewProjectImport(props.projectId, file);
    preview.value = res.data;
    // Die Vorgabe des Servers übernehmen: Wer den Dialog wegklickt, ohne
    // etwas anzufassen, bekommt das Vorsichtige.
    answers.value = Object.fromEntries(
      (res.data.items || []).filter((i) => i.choices?.length).map((i) => [i.key, i.choice]),
    );
  } catch (e) {
    toast.error(e.response?.data?.message || t('shares.package.previewFailed'));
  } finally {
    isUploading.value = false;
  }
};

const applyImport = async () => {
  isApplying.value = true;
  try {
    const res = await api.applyProjectImport(props.projectId, preview.value.token, answers.value);
    toast.success(t('shares.package.applied', {
      updated: res.data.summary.updated,
      created: res.data.summary.created + res.data.summary.restored,
      conflicts: res.data.summary.conflict,
    }));
    preview.value = null;
  } catch (e) {
    toast.error(e.response?.data?.message || t('shares.package.applyFailed'));
  } finally {
    isApplying.value = false;
  }
};

// Zeilen, bei denen es etwas zu entscheiden gibt – sie stehen oben und sind
// der Grund, warum es diesen Dialog überhaupt gibt.
const decisions = computed(() => (preview.value?.items || []).filter((i) => i.choices?.length));
// Der Rest wird nur gezählt. Eine Liste aus vierhundert „unverändert" liest
// niemand, und sie verdeckte das Wesentliche.
const quiet = computed(() => {
  const s = preview.value?.summary || {};
  return [
    { key: 'created', count: s.created, icon: FilePlus2, tone: 'text-emerald-600 dark:text-emerald-400' },
    { key: 'updated', count: s.updated, icon: FilePen, tone: 'text-marke' },
    { key: 'unchanged', count: s.unchanged, icon: FileCheck, tone: 'text-slate-400' },
    { key: 'kept_local', count: s.kept_local, icon: FileCheck, tone: 'text-slate-400' },
    { key: 'skipped', count: s.skipped, icon: Ban, tone: 'text-amber-600 dark:text-amber-400' },
  ].filter((z) => z.count > 0);
});

const ACTION_ICON = {
  conflict: FileWarning,
  missing: FileQuestion,
  restored: RotateCcw,
};
</script>

<template>
  <div class="flex flex-wrap items-center gap-2">
    <BaseButton variant="secondary" :loading="isExporting" @click="exportPackage">
      <Download class="w-4 h-4" />
      {{ t('shares.package.export') }}
    </BaseButton>

    <BaseButton v-if="canWrite" variant="secondary" :loading="isUploading" @click="pickFile">
      <Upload class="w-4 h-4" />
      {{ t('shares.package.import') }}
    </BaseButton>
    <input ref="fileInput" type="file" accept=".zip,application/zip" class="hidden" @change="onFilePicked" />
  </div>

  <!-- persistent: Der Dialog hält das ausgepackte Paket am Leben. Ein
       versehentlicher Klick daneben sollte die Arbeit nicht wegwerfen. -->
  <BaseModal
    v-if="preview"
    :title="t('shares.package.previewTitle')"
    size="lg"
    persistent
    @close="preview = null"
  >
    <div class="space-y-4">
      <p v-if="preview.degraded" class="text-sm rounded-xl px-3 py-2 bg-amber-50 dark:bg-amber-500/10 text-amber-800 dark:text-amber-300">
        {{ t('shares.package.degraded') }}
      </p>

      <!-- Was zu entscheiden ist – zuerst. -->
      <div v-if="decisions.length" class="space-y-3">
        <div
          v-for="item in decisions" :key="item.key"
          class="rounded-xl border border-linie px-3 py-2.5 space-y-2"
        >
          <div class="flex items-start gap-2">
            <component :is="ACTION_ICON[item.action]" class="w-4 h-4 mt-0.5 shrink-0 text-amber-600 dark:text-amber-400" />
            <div class="min-w-0">
              <div class="text-sm font-bold text-schrift break-words">{{ item.name }}</div>
              <div class="text-xs text-leise">{{ t(`shares.package.action.${item.action}`) }}</div>
            </div>
          </div>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="choice in item.choices" :key="choice"
              type="button"
              @click="answers[item.key] = choice"
              class="px-2.5 py-1 rounded-lg text-xs font-bold transition-colors cursor-pointer"
              :class="answers[item.key] === choice
                ? 'bg-marke text-white'
                : 'bg-auflage text-fliess hover:bg-slate-200 dark:hover:bg-slate-700'"
            >
              {{ t(`shares.package.choice.${choice}`) }}
            </button>
          </div>
        </div>
      </div>

      <p v-else class="text-sm text-leise">
        {{ t('shares.package.nothingToDecide') }}
      </p>

      <!-- Der Rest: gezählt, nicht aufgezählt. -->
      <div v-if="quiet.length" class="flex flex-wrap gap-x-4 gap-y-1 pt-1 border-t border-linie">
        <span v-for="z in quiet" :key="z.key" class="inline-flex items-center gap-1.5 text-xs font-medium" :class="z.tone">
          <component :is="z.icon" class="w-3.5 h-3.5" />
          {{ t(`shares.package.summary.${z.key}`, { count: z.count }) }}
        </span>
      </div>

      <p class="text-xs text-slate-400">{{ t('shares.package.expiresIn', { minutes: preview.expires_in_minutes }) }}</p>
    </div>

    <template #footer>
      <BaseButton variant="secondary" @click="preview = null">{{ t('common.cancel') }}</BaseButton>
      <BaseButton :loading="isApplying" @click="applyImport">{{ t('shares.package.apply') }}</BaseButton>
    </template>
  </BaseModal>
</template>
