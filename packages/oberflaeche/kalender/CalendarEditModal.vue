<script setup>
/*
 * Einen bestehenden Kalender bearbeiten -- Name, Farbe und Abo-Adresse.
 *
 * HIER STAND NUR DIE FARBE. Die Abo-Adresse liess sich beim Anlegen
 * eintragen und danach **nie wieder ansehen oder ändern** -- obwohl der
 * Server das Ändern längst erlaubt (`CalendarController::update` nimmt
 * `sync_url` entgegen). Es fehlte allein die Oberfläche.
 *
 * Am 17.09.2026 gefragt: „Was mir nicht so gut gefällt: dass ich nirgendwo
 * einsehen kann, welche URL ich importiert habe." Eine Adresse, die man
 * einträgt und nie wiedersieht, ist eine Angabe, der man ausgeliefert ist:
 * Bei einem Tippfehler bleibt nur, den Kalender zu löschen und neu
 * anzulegen -- und dabei gehen alle Termine mit.
 *
 * DIE ADRESSE IST EIN GEHEIMNIS IN ADRESSFORM. Wer sie hat, liest den
 * Kalender mit; sie trägt bei FamilyWall & Co. den Zugang im Pfad. Deshalb
 * steht sie nicht offen da, sondern hinter einem Klick -- wie ein Passwort
 * in einem Passwortfeld. Sichtbar für den, der hinsieht, nicht für den, der
 * daneben sitzt.
 */
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { PRESET_COLORS } from '@oberflaeche/shared/calendarColors';
import { Pencil, Check, Eye, EyeOff } from 'lucide-vue-next';

const props = defineProps({
  calendar: { type: Object, required: true },
  saving: { type: Boolean, default: false },
  /** Ohne Server gibt es keine Abos -- die App blendet das Feld aus. */
  kannAbo: { type: Boolean, default: true },
});
const emit = defineEmits(['save', 'close']);

const { t } = useI18n();

const name = ref(props.calendar.name);
const color = ref(props.calendar.color);
const syncUrl = ref(props.calendar.sync_url || '');
const offen = ref(false);

/** Der Wirt allein -- so viel darf immer dastehen, das nennt die Quelle. */
const wirt = computed(() => {
  try {
    return new URL(syncUrl.value).host;
  } catch {
    return '';
  }
});

function speichern() {
  emit('save', {
    name: name.value.trim() || props.calendar.name,
    color: color.value,
    // Leer heisst ausdruecklich „kein Abo mehr" -- `null`, nicht `''`:
    // Der Server prueft `nullable|url`, und ein leerer Text waere keine.
    sync_url: syncUrl.value.trim() || null,
  });
}
</script>

<template>
  <BaseModal size="sm" @close="emit('close')">
    <template #header>
      <div class="flex items-center gap-2">
        <Pencil class="w-5 h-5 text-marke" />
        <h3 class="text-lg font-bold text-schrift">{{ t('calendar.editModal.title') }}</h3>
      </div>
    </template>

    <div class="space-y-4">
      <div class="space-y-1.5">
        <label class="block text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.editModal.name') }}</label>
        <input v-model="name" type="text" class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-linie rounded-xl focus:outline-hidden focus:ring-2 focus:ring-marke text-sm font-semibold text-schrift">
      </div>

      <div class="space-y-2">
        <label class="block text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.editModal.color') }}</label>
        <div class="grid grid-cols-6 gap-2">
          <button v-for="c in PRESET_COLORS" :key="c.value" type="button" @click="color = c.value" class="w-10 h-10 rounded-full flex items-center justify-center transition-transform hover:scale-110 cursor-pointer shadow-sm border-2" :class="color === c.value ? 'border-slate-900 dark:border-white scale-110' : 'border-transparent'" :style="{ backgroundColor: c.value }" :title="c.label">
            <Check v-if="color === c.value" class="w-5 h-5 text-white" />
          </button>
        </div>
      </div>

      <div v-if="kannAbo" class="space-y-1.5">
        <label class="block text-xs font-bold uppercase tracking-wider text-leise">{{ t('calendar.editModal.feedUrl') }}</label>

        <div class="flex items-center gap-2">
          <input v-model="syncUrl" :type="offen ? 'text' : 'password'"
                 :placeholder="t('calendar.editModal.feedPlaceholder')"
                 autocomplete="off" spellcheck="false"
                 class="flex-1 min-w-0 px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-linie rounded-xl focus:outline-hidden focus:ring-2 focus:ring-marke text-sm font-mono text-schrift">
          <button type="button" @click="offen = !offen"
                  class="p-2 text-slate-400 hover:text-marke shrink-0"
                  :title="offen ? t('calendar.editModal.hide') : t('calendar.editModal.show')">
            <EyeOff v-if="offen" class="w-4 h-4" />
            <Eye v-else class="w-4 h-4" />
          </button>
        </div>

        <p v-if="wirt" class="text-xs text-leise">{{ t('calendar.editModal.from', { host: wirt }) }}</p>
        <p class="text-xs text-leise">{{ t('calendar.editModal.feedHint') }}</p>
      </div>
    </div>

    <div class="flex justify-end gap-2 pt-4">
      <BaseButton variant="ghost" @click="emit('close')">{{ t('common.cancel') }}</BaseButton>
      <BaseButton :loading="saving" @click="speichern">{{ t('common.save') }}</BaseButton>
    </div>
  </BaseModal>
</template>
