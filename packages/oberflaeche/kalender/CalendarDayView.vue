<script setup>
// Die Tagesansicht: „was braucht das Kind morgen?"
//
// Bewusst KEIN Raster. Ein Wochenraster beantwortet „wann ist Mathe" und zeigt
// am Vorabend drei Kästchen mit Fachnamen – also genau das, was man ohnehin
// weiß. Gesucht ist keine Übersicht, sondern eine DURCHSICHT: die Stunden der
// Reihe nach, und an jeder das, was dafür zu tun ist.
//
// Die Kopfzeile beantwortet die erste Frage des Abends: Ist das Kind morgen
// überhaupt da? Sie kommt aus der Betreuungsschicht, und genau deshalb gehört
// diese Ansicht in den Kalender und nicht ins Projekt – nur hier liegen beide
// Schichten übereinander.
import { ref, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  X, ChevronLeft, ChevronRight, Plus, NotebookText, AlertTriangle,
  Square, MapPin, Loader2, ExternalLink,
} from 'lucide-vue-next';
import { formatDateLong, formatWeekdayLong, formatTime } from '@oberflaeche/shared/date';

const props = defineProps({
  date: { type: String, required: true },       // YYYY-MM-DD
  bloecke: { type: Array, required: true },
  istLeer: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
  saving: { type: Boolean, default: false },
});
const emit = defineEmits(['close', 'shift', 'open-in-project', 'open-heft', 'add-homework']);

const { t } = useI18n();

// Die offene Schnelleingabe: { projectId, target, subjectId, subjectName }.
const eingabe = ref(null);
const titel = ref('');
const eingabefeld = ref(null);

const zeit = (wert) => (wert ? formatTime(`${props.date}T${wert}`) : '');

// „bei dir" ist die Auskunft, um die es geht – der Name des anderen nur die
// zweitbeste. Ohne Zuweisung bleibt der Titel des Abschnitts („Ferienwoche").
const betreuungsText = (b) => {
  if (b.beiMir) return t('calendar.day.careWithYou');
  if (b.name) return t('calendar.day.careWith', { name: b.name });
  return b.title || '';
};

async function eingabeOeffnen(block, stunde = null) {
  eingabe.value = {
    projectId: block.projectId,
    target: block.homeworkTarget,
    subjectId: stunde?.subject?.id ?? null,
    subjectName: stunde?.title ?? null,
  };
  titel.value = '';
  await nextTick();
  // Innerhalb eines v-for sammelt Vue Refs in einem Array – hier ist immer
  // genau ein Formular offen, aber die Form der Referenz hängt daran, wo es
  // steht.
  const feld = Array.isArray(eingabefeld.value) ? eingabefeld.value[0] : eingabefeld.value;
  feld?.focus();
}

function absenden() {
  const text = titel.value.trim();
  if (!text || !eingabe.value?.target) return;
  emit('add-homework', { ...eingabe.value, title: text });
  titel.value = '';
}
</script>

<template>
  <div class="fixed inset-0 z-[100] flex items-start justify-center p-4 sm:p-8 bg-slate-900/40 backdrop-blur-sm overflow-y-auto" @click.self="emit('close')">
    <div class="bg-flaeche w-full max-w-2xl rounded-xl shadow-2xl overflow-hidden">

      <!-- Kopf: Tag und Blätterpfeile -->
      <div class="flex items-center gap-2 px-5 py-4 border-b border-linie">
        <button @click="emit('shift', -1)" class="p-1 rounded-xl text-slate-500 hover:bg-auflage"><ChevronLeft class="w-5 h-5" /></button>
        <div class="min-w-0">
          <h2 class="text-lg font-bold dark:text-white truncate">{{ formatWeekdayLong(date) }}</h2>
          <p class="text-xs font-semibold text-slate-400">{{ formatDateLong(date) }}</p>
        </div>
        <button @click="emit('shift', 1)" class="p-1 rounded-xl text-slate-500 hover:bg-auflage"><ChevronRight class="w-5 h-5" /></button>
        <button @click="emit('close')" class="ml-auto p-1 rounded-full bg-auflage dark:text-white"><X class="w-5 h-5" /></button>
      </div>

      <div v-if="loading" class="p-10 flex justify-center"><Loader2 class="w-6 h-6 animate-spin text-marke" /></div>

      <p v-else-if="istLeer" class="p-10 text-center text-sm font-semibold text-slate-400">{{ t('calendar.day.empty') }}</p>

      <div v-else class="divide-y divide-linie">
        <section v-for="block in bloecke" :key="block.projectId" class="p-5">

          <!-- Projekt und Betreuung: „Schule – Lena … bei dir" -->
          <div class="flex items-baseline justify-between gap-3 mb-3">
            <h3 class="text-sm font-extrabold text-fliess truncate">{{ block.projectName }}</h3>
            <span v-for="(b, i) in block.betreuung" :key="i"
              class="shrink-0 text-xs font-bold px-2 py-0.5 rounded-full text-white"
              :style="{ backgroundColor: b.color || '#6366f1' }">
              {{ betreuungsText(b) }}<template v-if="b.starts_at"> · {{ zeit(b.starts_at) }}</template>
            </span>
          </div>

          <!-- Die Stunden des Tages -->
          <ul class="space-y-1">
            <li v-for="stunde in block.stunden" :key="stunde.link.kind + '-' + stunde.link.id"
              class="group flex items-start gap-3 rounded-xl px-2 py-1.5 hover:bg-slate-50 dark:hover:bg-slate-800/50">
              <span class="w-11 shrink-0 pt-0.5 text-xs font-bold text-slate-400 tabular-nums">{{ stunde.is_all_day ? '' : zeit(stunde.starts_at) }}</span>
              <span class="w-1 self-stretch rounded-full shrink-0" :style="{ backgroundColor: stunde.color || '#cbd5e1' }"></span>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  <button @click="emit('open-in-project', { link: stunde.link, projectId: block.projectId })"
                    class="text-sm font-bold text-slate-800 dark:text-slate-100 hover:text-marke">{{ stunde.title }}</button>
                  <span v-if="stunde.place" class="text-xs text-slate-400 flex items-center gap-0.5"><MapPin class="w-3 h-3" />{{ stunde.place.name }}</span>
                  <!-- „Heft ›" nur, wenn die Freigabe für DIESEN Betrachter
                       trägt – das entscheidet der Server, hier steht nur, ob
                       er eine Mappe mitgeschickt hat. -->
                  <button v-if="stunde.subject?.note_folder"
                    @click="emit('open-heft', { folderId: stunde.subject.note_folder.id, projectId: block.projectId })"
                    class="text-xs font-bold text-marke flex items-center gap-0.5 hover:underline">
                    <NotebookText class="w-3 h-3" /> {{ t('calendar.day.notebook') }}
                  </button>
                  <button v-if="block.homeworkTarget" @click="eingabeOeffnen(block, stunde)"
                    class="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-marke p-0.5 rounded-lg"
                    :title="t('calendar.day.addHomework')"><Plus class="w-3.5 h-3.5" /></button>
                </div>
                <p v-if="stunde.subtitle" class="text-xs text-slate-400">{{ stunde.subtitle }}</p>

                <!-- Was an dieser Stunde hängt -->
                <button v-for="item in stunde.items" :key="item.kind + '-' + item.id"
                  @click="emit('open-in-project', { link: item.link, projectId: block.projectId })"
                  class="mt-1 w-full flex items-center gap-1.5 text-left text-xs font-semibold text-fliess hover:text-marke">
                  <AlertTriangle v-if="item.kind === 'milestone'" class="w-3.5 h-3.5 shrink-0 text-amber-500" />
                  <Square v-else class="w-3.5 h-3.5 shrink-0 text-slate-400" />
                  <span class="truncate">{{ item.title }}</span>
                  <!-- Der Vorausblick sagt, WANN – sonst wäre eine Arbeit in
                       zwölf Tagen von einer morgen nicht zu unterscheiden. -->
                  <span v-if="item.is_preview" class="shrink-0 text-[10px] font-bold text-amber-600 dark:text-amber-400">{{ t('calendar.day.inDays', { days: item.days_left }) }}</span>
                </button>
              </div>
            </li>
          </ul>

          <!-- Was an keine Stunde passte. Eigene Überschrift, damit es nicht
               wie eine vergessene Stunde aussieht – und nie verschluckt. -->
          <div v-if="block.ungebunden.length" class="mt-3 pt-3 border-t border-dashed border-linie">
            <p class="text-[10px] font-extrabold uppercase text-slate-400 mb-1">{{ t('calendar.day.unbound') }}</p>
            <button v-for="item in block.ungebunden" :key="item.kind + '-' + item.id"
              @click="emit('open-in-project', { link: item.link, projectId: block.projectId })"
              class="w-full flex items-center gap-1.5 text-left text-xs font-semibold text-fliess hover:text-marke py-0.5">
              <AlertTriangle v-if="item.kind === 'milestone'" class="w-3.5 h-3.5 shrink-0 text-amber-500" />
              <Square v-else class="w-3.5 h-3.5 shrink-0 text-slate-400" />
              <span class="truncate">{{ item.title }}</span>
            </button>
          </div>

          <!-- Ein Tipp, nicht ein Formular: Fach, Fälligkeit und Zielspalte
               stehen schon fest, einzutippen bleibt „S. 42 Nr. 3–7". Ohne das
               wird die Ansicht nicht gepflegt und stirbt nach einer Woche. -->
          <div v-if="block.homeworkTarget" class="mt-3">
            <form v-if="eingabe && eingabe.projectId === block.projectId" @submit.prevent="absenden" class="flex items-center gap-2">
              <input ref="eingabefeld" v-model="titel" type="text" :placeholder="t('calendar.day.homeworkPlaceholder')"
                class="flex-1 px-3 py-2 bg-vertieft border border-linie rounded-xl text-sm font-semibold dark:text-white focus:outline-none focus:ring-2 focus:ring-marke" />
              <span v-if="eingabe.subjectName" class="text-xs font-bold text-slate-400 shrink-0">{{ eingabe.subjectName }}</span>
              <button type="submit" :disabled="saving || !titel.trim()" class="px-3 py-2 bg-marke disabled:opacity-50 text-white text-sm font-bold rounded-xl shrink-0">
                <Loader2 v-if="saving" class="w-4 h-4 animate-spin" /><span v-else>{{ t('calendar.day.save') }}</span>
              </button>
              <button type="button" @click="eingabe = null" class="p-2 text-slate-400 shrink-0"><X class="w-4 h-4" /></button>
            </form>
            <button v-else @click="eingabeOeffnen(block)" class="flex items-center gap-1 text-xs font-bold text-marke">
              <Plus class="w-3.5 h-3.5" /> {{ t('calendar.day.addHomework') }}
            </button>
          </div>

          <p v-else class="mt-3 text-xs text-slate-400 flex items-center gap-1">
            <ExternalLink class="w-3 h-3" /> {{ t('calendar.day.noBoard') }}
          </p>
        </section>
      </div>
    </div>
  </div>
</template>
