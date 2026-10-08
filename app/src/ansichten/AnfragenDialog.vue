<script setup>
/*
 * Der Anfragen-Ordner (06.10.2026): Nachrichten vor Ort von Personen, die
 * hier weder bekannt noch bestätigt sind. Sie stehen nicht im Verlauf.
 *
 * KEIN „ANNEHMEN". Tiffy wollte entweder Annehmen oder 6 Ziffern -- und dann
 * die 6 Ziffern: Bekannt wird jemand nur, wenn man sich trifft und beide
 * Geräte dieselben Ziffern zeigen. Bis dahin höchstens drei Nachrichten zu
 * je 500 Zeichen, und antworten geht nicht. Bestätigen gibt es deshalb nur,
 * wenn ein Gerät der Person gerade in der Nähe ist.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2, UserCheck, Ban } from 'lucide-vue-next';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { nachrichtenQuelle as quelle } from '../quellen/nachrichten';

const emit = defineEmits(['close', 'geaendert']);
const { t, locale } = useI18n();
const toast = useToast();
const { confirmDialog } = useConfirm();

const lage = ref(null);
const arbeitet = ref('');

async function laden() {
    try { lage.value = await quelle.anfragen(); } catch (e) { lage.value = { erlaubt: false, offen: [] }; toast.error(String(e)); }
}
onMounted(laden);
defineExpose({ laden });

const name = (a) => a.name || t('app.anfragen.ohneName');
const zeit = (iso) => new Date(iso).toLocaleString(locale.value, { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });

// Der 6-Ziffern-Dialog erscheint über jeder Seite (EinladungsDialog.vue),
// sobald das andere Gerät geantwortet hat; hier nur der Anstoß.
async function bestaetigen(a) {
    arbeitet.value = a.person;
    try {
        await quelle.kontaktBestaetigen(a.person);
        emit('close');
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = '';
    }
}

async function blockieren(a) {
    const ok = await confirmDialog(t('settings.messages.nahBlockierenText', { name: name(a) }), {
        title: t('settings.messages.nahBlockieren'),
        confirmLabel: t('settings.messages.nahBlockieren'),
    });
    if (!ok) return;
    arbeitet.value = a.person;
    try {
        await quelle.blockieren(a.person);
        toast.success(t('settings.messages.nahBlockiert', { name: name(a) }));
        await laden();
        emit('geaendert');
    } catch (e) {
        toast.error(String(e));
    } finally {
        arbeitet.value = '';
    }
}
</script>

<template>
  <BaseModal :title="t('app.anfragen.titel')" size="lg" @close="emit('close')">
    <div class="space-y-4">
      <p class="text-sm text-leise">{{ t('app.anfragen.hinweis') }}</p>
      <div v-if="!lage" class="flex justify-center py-6"><Loader2 class="w-5 h-5 animate-spin text-leise" /></div>
      <p v-else-if="!lage.offen.length" class="text-sm text-fliess py-4 text-center">{{ t('app.anfragen.keine') }}</p>
      <ul v-else class="space-y-3">
        <li v-for="a in lage.offen" :key="a.person" class="rounded-xl border border-linie p-3 space-y-2">
          <div class="flex flex-wrap items-baseline justify-between gap-2">
            <span class="font-bold text-schrift break-all">{{ name(a) }}</span>
            <span class="px-2 py-0.5 rounded-full font-extrabold text-[10px] bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300">
              {{ t('settings.messages.nahUnbekannt') }}
            </span>
          </div>
          <div v-for="(n, i) in a.texte" :key="i" class="text-sm text-fliess">
            <span class="text-xs text-leise">{{ zeit(n.zeit) }}</span>
            <p class="whitespace-pre-wrap break-words">{{ n.text }}</p>
          </div>
          <div class="flex flex-wrap items-center justify-end gap-4 pt-1 text-sm font-bold">
            <span v-if="!a.da" class="text-xs font-medium text-leise mr-auto">{{ t('app.anfragen.nichtDa') }}</span>
            <button type="button" class="text-marke flex items-center gap-1 cursor-pointer disabled:opacity-50"
                    :disabled="!a.da || arbeitet !== ''" @click="bestaetigen(a)">
              <Loader2 v-if="arbeitet === a.person" class="w-4 h-4 animate-spin" /><UserCheck v-else class="w-4 h-4" />
              {{ t('app.anfragen.bestaetigen') }}
            </button>
            <button type="button" class="text-rose-600 dark:text-rose-400 flex items-center gap-1 cursor-pointer disabled:opacity-50"
                    :disabled="arbeitet !== ''" @click="blockieren(a)">
              <Ban class="w-4 h-4" /> {{ t('settings.messages.nahBlockieren') }}
            </button>
          </div>
        </li>
      </ul>
    </div>
  </BaseModal>
</template>
