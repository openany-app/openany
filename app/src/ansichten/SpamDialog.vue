<script setup>
/*
 * Spamverdacht eines Postfachs -- eine eigene Liste, NICHT im Verlauf
 * (Tiffy, 29.09.2026).
 *
 * Die Liste kommt bei jedem Öffnen frisch vom Mailserver (die neuesten 50)
 * und wird nirgends gespeichert. „Kein Spam" verschiebt die Mail in den
 * Posteingang; dann steht sie im Verlauf wie jede andere. Anhänge zeigt die
 * Liste nur mit Namen -- aus dem Spam-Ordner öffnet die App nichts.
 */
import { ref, onMounted } from 'vue';
import { Loader2, AlertTriangle, Inbox, Trash2, Paperclip } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { mailQuelle } from '../quellen/nachrichten';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps({
    adresse: { type: String, required: true },
});
const emit = defineEmits(['close', 'geaendert']);

const toast = useToast();
const liste = ref(null);
const fehler = ref('');
// Welche Mail gerade verschoben oder gelöscht wird (uid).
const arbeitet = ref(null);

async function laden() {
    fehler.value = '';
    try {
        liste.value = await mailQuelle.spam(props.adresse);
    } catch (e) {
        liste.value = [];
        fehler.value = String(e);
    }
}

async function keinSpam(m) {
    arbeitet.value = m.uid;
    try {
        await mailQuelle.keinSpam(props.adresse, m.uid);
        liste.value = liste.value.filter((x) => x.uid !== m.uid);
        toast.success(t('app.spam.verschoben'));
        emit('geaendert');
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
    }
}

async function loeschen(m) {
    arbeitet.value = m.uid;
    try {
        await mailQuelle.spamLoeschen(props.adresse, m.uid);
        liste.value = liste.value.filter((x) => x.uid !== m.uid);
        emit('geaendert');
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = null;
    }
}

const zeit = (iso) => new Date(iso).toLocaleString('de-DE', { dateStyle: 'short', timeStyle: 'short' });

onMounted(laden);
</script>

<template>
  <BaseModal :title="t('app.spam.titel', { adresse })" size="lg" @close="emit('close')">
    <div class="space-y-3">
      <p class="text-sm text-leise">
        {{ t('app.spam.hinweis') }}
      </p>

      <div v-if="!liste" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <p v-else-if="fehler" class="text-sm text-warnung flex items-start gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" /><span class="min-w-0 break-words">{{ fehler }}</span>
      </p>
      <p v-else-if="!liste.length" class="text-sm text-fliess py-4 text-center">{{ t('app.spam.keiner') }}</p>

      <ul v-else class="space-y-3">
        <li v-for="m in liste" :key="m.uid" class="rounded-xl border border-linie p-3 space-y-1.5">
          <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 text-xs">
            <span class="font-bold text-fliess break-all">{{ m.von_name ? `${m.von_name} <${m.von}>` : m.von }}</span>
            <span class="text-leise shrink-0">{{ zeit(m.zeit) }}</span>
          </div>
          <p class="text-sm font-extrabold text-schrift break-words">{{ m.betreff || t('app.spam.ohneBetreff') }}</p>
          <p v-if="m.text" class="text-sm text-fliess break-words line-clamp-3 whitespace-pre-line">{{ m.text }}</p>
          <p v-if="m.anhaenge.length" class="text-xs text-leise flex items-start gap-1">
            <Paperclip class="w-3.5 h-3.5 shrink-0 mt-0.5" /><span class="min-w-0 break-all">{{ m.anhaenge.join(', ') }}</span>
          </p>
          <div class="flex flex-wrap justify-end gap-4 pt-1 text-sm font-bold">
            <button type="button" class="text-marke flex items-center gap-1 cursor-pointer disabled:opacity-50"
                    :disabled="arbeitet !== null" @click="keinSpam(m)">
              <Loader2 v-if="arbeitet === m.uid" class="w-4 h-4 animate-spin" /><Inbox v-else class="w-4 h-4" /> {{ t('app.spam.keinSpam') }}
            </button>
            <button type="button" class="text-rose-600 dark:text-rose-400 flex items-center gap-1 cursor-pointer disabled:opacity-50"
                    :disabled="arbeitet !== null" @click="loeschen(m)">
              <Trash2 class="w-4 h-4" /> {{ t('common.delete') }}
            </button>
          </div>
        </li>
      </ul>
    </div>
  </BaseModal>
</template>
