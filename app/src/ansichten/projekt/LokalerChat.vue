<script setup>
/*
 * Chat vor Ort (01.10.2026). Was man schreibt, geht sofort an jedes Mitglied
 * in der Nähe; wer nicht da ist, bekommt es beim nächsten Abgleich. Neues von
 * anderen meldet der Zähler `chat` aus `nah_lage` (alle 3 s).
 */
import { ref, watch, nextTick, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Send } from 'lucide-vue-next';
import { useToast } from '@oberflaeche/composables/useToast';
import { projekteQuelle as quelle } from '../../quellen/projekte';
import { useNahAbgleich } from '../../nahAbgleich';

const props = defineProps({ project: { type: Object, required: true } });
const toast = useToast();
const { t, locale } = useI18n();
const { nah } = useNahAbgleich();

const chat = ref([]);
const text = ref('');
const sendet = ref(false);
const ende = ref(null);
// Schreiben darf nur, wer (noch) Mitglied ist.
const darf = ref(false);

async function laden() {
  try {
    chat.value = await quelle.chat(props.project.id);
    // Wer den Chat vor sich hat, hat ihn gelesen.
    quelle.chatGelesen(props.project.id).catch(() => {});
    await nextTick();
    ende.value?.scrollIntoView({ block: 'nearest' });
  } catch {
    // Ohne Chat bleibt der Rest des Projekts nutzbar.
  }
}
async function senden() {
  const t = text.value.trim();
  if (!t) return;
  sendet.value = true;
  try {
    chat.value = [...chat.value, await quelle.chatSenden(props.project.id, t)];
    text.value = '';
    await nextTick();
    ende.value?.scrollIntoView({ block: 'nearest' });
  } catch (e) {
    toast.error(String(e));
  } finally {
    sendet.value = false;
  }
}
watch(() => nah.value?.chat, (jetzt, vorher) => {
  if (vorher !== undefined && jetzt !== vorher) laden();
});
onMounted(() => {
  laden();
  quelle.mitglieder(props.project.id).then((m) => { darf.value = Boolean(m.bin_mitglied); }).catch(() => {});
});
const zeitKurz = (iso) => new Date(iso).toLocaleString(locale.value, { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
</script>

<template>
  <div class="space-y-2 md:max-w-[720px]">
    <p class="text-xs text-leise">{{ t('app.projekte.chatHinweis') }}</p>
    <div class="karte shadow-sm p-3 space-y-2 max-h-[60vh] overflow-y-auto">
      <p v-if="!chat.length" class="text-sm text-leise">{{ t('app.projekte.keineNachrichten') }}</p>
      <div v-for="n in chat" :key="n.id" class="flex" :class="n.ich ? 'justify-end' : 'justify-start'">
        <div class="max-w-[80%] rounded-xl px-3 py-2 text-sm"
             :class="n.ich ? 'bg-marke-leise text-schrift' : 'bg-auflage text-schrift'">
          <div class="text-[11px] text-leise">{{ n.ich ? t('app.projekte.duKurz') : n.name }} · {{ zeitKurz(n.at) }}</div>
          <div class="whitespace-pre-wrap break-words">{{ n.text }}</div>
        </div>
      </div>
      <div ref="ende" />
    </div>
    <form v-if="darf" class="flex items-end gap-2" @submit.prevent="senden">
      <textarea v-model="text" rows="1" class="feld flex-1 resize-none" :placeholder="t('app.projekte.nachricht')"
                @keydown.enter.exact.prevent="senden" />
      <button type="submit" class="knopf-wichtig shrink-0 flex items-center gap-1.5" :disabled="sendet || !text.trim()">
        <Send class="w-4 h-4" /> {{ t('app.projekte.senden') }}
      </button>
    </form>
  </div>
</template>
