<script setup>
/*
 * Die Sicherheitsnummer mit einem Konto (docs/plan-adressbuch-e2e.md, 2c).
 *
 * Zwölf Fünfergruppen aus den Konto-Schlüsseln beider Seiten -- auf beiden
 * Geräten gleich. Stimmen sie überein, hat der Server keinem von beiden
 * einen falschen Schlüssel untergeschoben. „Stimmt überein" hält den
 * jetzigen Schlüssel im Kontakt fest (verschlüsselt, auf allen eigenen
 * Geräten); ändert er sich drüben, fällt das Häkchen weg.
 *
 * Die Quelle bringt `sicherheitsnummer(name)`,
 * `sicherheitsnummerBestaetigen(name)` und `sicherheitsnummerAufheben(name)`.
 */
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ShieldCheck, ShieldAlert, ShieldQuestion, Loader2 } from 'lucide-vue-next';
import BaseModal from '../base/BaseModal.vue';
import BaseButton from '../base/BaseButton.vue';
import { useToast } from '../composables/useToast';

const props = defineProps({
  name: { type: String, required: true },
  quelle: { type: Object, required: true },
});
const emit = defineEmits(['close', 'geaendert']);

const { t } = useI18n();
const toast = useToast();
const s = ref(null);
const fehler = ref('');
const arbeitet = ref(false);

onMounted(async () => {
  try {
    s.value = await props.quelle.sicherheitsnummer(props.name);
  } catch (e) {
    fehler.value = String(e?.message ?? e);
  }
});

async function tun(was) {
  arbeitet.value = true;
  try {
    s.value = was === 'bestaetigen'
      ? await props.quelle.sicherheitsnummerBestaetigen(props.name)
      : await props.quelle.sicherheitsnummerAufheben(props.name);
    if (was === 'bestaetigen') toast.success(t('app.sicherheit.bestaetigt', { name: props.name }));
    emit('geaendert', s.value);
  } catch (e) {
    toast.error(String(e?.message ?? e));
  } finally {
    arbeitet.value = false;
  }
}
</script>

<template>
  <BaseModal :title="t('app.sicherheit.mit', { name })" @close="emit('close')">
    <div v-if="fehler" class="text-sm text-rose-600 dark:text-rose-400">{{ fehler }}</div>
    <div v-else-if="!s" class="py-8 flex justify-center"><Loader2 class="w-6 h-6 animate-spin text-leise" /></div>
    <div v-else class="space-y-4">
      <p v-if="s.stand === 'ohne_schluessel'" class="text-sm text-leise">{{ t('app.sicherheit.ohneSchluessel', { name }) }}</p>

      <template v-else>
        <p class="text-sm text-fliess">{{ t('app.sicherheit.erklaerung') }}</p>
        <!-- Vier Zeilen zu drei Gruppen: so liest man sie einander vor. -->
        <div class="grid grid-cols-3 gap-x-4 gap-y-2 p-4 rounded-xl bg-auflage font-mono text-lg sm:text-xl text-schrift text-center tabular-nums select-all">
          <span v-for="(g, i) in s.nummer.split(' ')" :key="i">{{ g }}</span>
        </div>

        <p v-if="s.stand === 'geprueft'" class="flex items-start gap-2 text-sm text-marke font-bold">
          <ShieldCheck class="w-4 h-4 shrink-0 mt-0.5" /> {{ t('app.sicherheit.geprueft', { name }) }}
        </p>
        <p v-else-if="s.stand === 'geaendert'" class="flex items-start gap-2 text-sm text-amber-700 dark:text-amber-400 font-bold">
          <ShieldAlert class="w-4 h-4 shrink-0 mt-0.5" /> {{ t('app.sicherheit.geaendert', { name }) }}
        </p>
        <p v-else class="flex items-start gap-2 text-sm text-leise">
          <ShieldQuestion class="w-4 h-4 shrink-0 mt-0.5" /> {{ t('app.sicherheit.offen') }}
          <template v-if="!s.kontakt">{{ t('app.sicherheit.neuerKontakt', { name }) }}</template>
        </p>

        <div class="flex flex-wrap justify-end gap-2">
          <BaseButton v-if="s.stand === 'geprueft'" variant="secondary" type="button" :disabled="arbeitet" @click="tun('aufheben')">
            {{ t('app.sicherheit.aufheben') }}
          </BaseButton>
          <BaseButton v-else type="button" :disabled="arbeitet" @click="tun('bestaetigen')">
            <ShieldCheck class="w-4 h-4" /> {{ t('app.sicherheit.bestaetigen') }}
          </BaseButton>
        </div>
      </template>
    </div>
  </BaseModal>
</template>
