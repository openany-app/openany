<script setup>
// Hilfe-Seite (/hilfe, Footer-Link): links eine Kachel mit anklickbarem
// Inhaltsverzeichnis (Titel + Untertitel), rechts eine große Kachel mit
// dem langen Handbuch zu allen Modulen und Funktionen. Die Gliederung steht
// in shared/helpOutline.js, die Texte in i18n/locales/{de,en}/help.js.
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  BookOpen, Rocket, NotebookPen, HardDrive, CalendarDays,
  Briefcase, MessageCircle, BookUser, Settings2, LifeBuoy, ChevronRight,
} from 'lucide-vue-next';
import { hilfeGliederung } from '@oberflaeche/shared/helpOutline';

// Seit dem 15.09.2026 im gemeinsamen Paket. Der Sprung zu einer Stelle kommt
// als Eigenschaft (`anker`) herein statt aus der Route: Die Webapp setzt ihn
// aus `#…` in der Adresse, das Programm hat keine.
const props = defineProps({
  anker: { type: String, default: '' },
  // Im Programm: die Abschnitte mit `nur: 'app'` statt derer mit `nur: 'web'`.
  programm: { type: Boolean, default: false },
  // Die Versionsnummer der App. Die Webapp gibt keine mit -- dort steht
  // dann keine Zeile.
  version: { type: String, default: '' },
});

const { t } = useI18n();

// Gliederung (Inhalt) steht in shared/helpOutline.js und wird dort gegen
// beide Sprachen geprüft; hier hängt nur das Symbol je Abschnitt daran.
const ICONS = {
  start: Rocket,
  notes: NotebookPen,
  files: HardDrive,
  calendar: CalendarDays,
  projects: Briefcase,
  messages: MessageCircle,
  contacts: BookUser,
  settings: Settings2,
  contact: LifeBuoy,
};

const STRUCTURE = hilfeGliederung({ programm: props.programm });

const sections = computed(() => STRUCTURE.map((s) => ({
  id: s.id,
  icon: ICONS[s.id],
  title: t(`help.sections.${s.id}.title`),
  subs: s.subs.map((sub) => ({
    id: `${s.id}-${sub.id}`,
    title: t(`help.sections.${s.id}.subs.${sub.id}.title`),
    paragraphs: Array.from({ length: sub.p }, (_, i) => t(`help.sections.${s.id}.subs.${sub.id}.p${i + 1}`)),
  })),
})));

// Welche Abschnitte im Inhaltsverzeichnis aufgeklappt sind. Zuerst keiner,
// und überall gleich -- auf dem Handy steht das Verzeichnis über dem Text
// und sah nie, welcher Abschnitt gerade gelesen wird (Tiffy, 08.10.2026).
const offen = ref(new Set());
const umschalten = (id) => {
  const neu = new Set(offen.value);
  if (neu.has(id)) neu.delete(id); else neu.add(id);
  offen.value = neu;
};

// Aktiven Abschnitt fürs Inhaltsverzeichnis mitführen: die zuletzt über
// die obere Bildschirmkante gewanderte Überschrift gilt als aktiv.
const activeId = ref(STRUCTURE[0].id);
const onScroll = () => {
  const marker = 96; // unterhalb der Kopfzeile
  let current = STRUCTURE[0].id;
  for (const s of STRUCTURE) {
    const el = document.getElementById(s.id);
    if (el && el.getBoundingClientRect().top <= marker) current = s.id;
  }
  activeId.value = current;
};

const jumpTo = (id) => {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
};

// Anker-Links (/hilfe#projects-orte, z. B. aus Leerzuständen oder dem
// Footer): beim Laden und bei Hash-Wechseln zur Stelle scrollen.
const scrollToHash = () => { if (props.anker) jumpTo(props.anker.replace(/^#/, '')); };
watch(() => props.anker, scrollToHash);

onMounted(() => {
  window.addEventListener('scroll', onScroll, { passive: true });
  nextTick(scrollToHash);
});
onBeforeUnmount(() => window.removeEventListener('scroll', onScroll));
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <div class="flex items-center gap-3 mb-2">
      <div class="w-10 h-10 rounded-xl bg-marke-leise flex items-center justify-center text-marke">
        <BookOpen class="w-5 h-5" />
      </div>
      <h1 class="text-2xl font-extrabold text-schrift tracking-tight">{{ t('help.title') }}</h1>
    </div>
    <p class="text-sm text-leise max-w-3xl" :class="version ? 'mb-1' : 'mb-6'">{{ t('help.intro') }}</p>
    <p v-if="version" class="text-xs text-leise mb-6">{{ t('help.version', { version }) }}</p>

    <div class="grid grid-cols-1 lg:grid-cols-[18rem_1fr] gap-6 items-start">

      <!-- Inhaltsverzeichnis (links, klebt beim Scrollen) -->
      <nav class="lg:sticky lg:top-20 karte shadow-sm p-5 max-h-[calc(100vh-7rem)] overflow-y-auto">
        <div class="text-xs font-extrabold uppercase tracking-wider text-slate-400 mb-3">{{ t('help.toc') }}</div>
        <ul class="space-y-1">
          <li v-for="s in sections" :key="s.id">
            <div class="flex items-center gap-1">
              <button @click="jumpTo(s.id)"
                class="flex-1 min-w-0 flex items-center gap-2 px-2.5 py-1.5 rounded-xl text-left text-sm font-bold transition-colors cursor-pointer"
                :class="activeId === s.id
                  ? 'bg-marke-leise text-marke'
                  : 'text-fliess hover:bg-auflage'">
                <component :is="s.icon" class="w-4 h-4 shrink-0" :class="activeId === s.id ? 'text-marke' : 'text-slate-400'" />
                {{ s.title }}
              </button>
              <button type="button" @click="umschalten(s.id)"
                class="shrink-0 p-1.5 rounded-xl text-slate-400 hover:text-marke hover:bg-auflage transition-colors cursor-pointer"
                :aria-expanded="offen.has(s.id)" :aria-label="s.title">
                <ChevronRight class="w-4 h-4 transition-transform" :class="offen.has(s.id) ? 'rotate-90' : ''" />
              </button>
            </div>
            <ul v-if="offen.has(s.id)" class="mt-1 mb-2 ml-4 pl-3 border-l border-linie space-y-0.5">
              <li v-for="sub in s.subs" :key="sub.id">
                <button @click="jumpTo(sub.id)"
                  class="w-full px-2 py-1 rounded-xl text-left text-xs font-semibold text-leise hover:text-marke hover:bg-auflage transition-colors cursor-pointer">
                  {{ sub.title }}
                </button>
              </li>
            </ul>
          </li>
        </ul>
      </nav>

      <!-- Handbuch (rechts) -->
      <article class="karte shadow-sm p-6 sm:p-10">
        <section v-for="(s, idx) in sections" :key="s.id" :id="s.id" class="scroll-mt-20" :class="idx > 0 ? 'mt-12 pt-10 border-t border-linie' : ''">
          <div class="flex items-center gap-3 mb-6">
            <div class="w-9 h-9 rounded-xl bg-marke-leise flex items-center justify-center text-marke">
              <component :is="s.icon" class="w-4.5 h-4.5" />
            </div>
            <h2 class="text-xl font-extrabold text-schrift tracking-tight">{{ s.title }}</h2>
          </div>
          <div v-for="sub in s.subs" :key="sub.id" :id="sub.id" class="scroll-mt-20 mb-7 last:mb-0">
            <h3 class="text-sm font-extrabold text-schrift mb-2">{{ sub.title }}</h3>
            <p v-for="(p, pi) in sub.paragraphs" :key="pi"
              class="text-sm leading-relaxed text-fliess mb-2 last:mb-0">{{ p }}</p>
          </div>
        </section>
      </article>

    </div>
  </div>
</template>
