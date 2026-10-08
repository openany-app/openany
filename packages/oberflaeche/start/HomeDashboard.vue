<script setup>
// Widget-Startseite: kompakte Kacheln (Kalender, Notizen, Speicher,
// Nachrichten, Adressbuch, Projekte) statt der vollen Modul-Ansichten. Welche Kacheln erscheinen,
// steuert die Modul-Auswahl aus den Einstellungen; alle Daten kommen
// gebündelt von GET /api/dashboard.
import { ref, computed, onMounted, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import { expandRecurringEvents } from '@oberflaeche/shared/expandEvents';
import { formatTime, formatWeekday, getLocalDateString } from '@oberflaeche/shared/date';
import { formatBytes } from '@oberflaeche/shared/format';
import { DEFAULT_CALENDAR_COLOR } from '@oberflaeche/shared/calendarColors';
import { formatDateShort } from '@oberflaeche/shared/date';
import {
  CalendarDays, FileText, HardDrive, Briefcase, ArrowRight, Loader2,
  Folder, Archive, Users, Mail, Contact, UserPlus, Search,
} from 'lucide-vue-next';

const { t } = useI18n();
const props = defineProps({
  // Ids der für die Startseite gewählten Module (Reihenfolge der Registry).
  moduleIds: { type: Array, required: true },
  // getDashboard() – dieselbe Antwort wie /api/dashboard. Seit dem 15.09.2026
  // im gemeinsamen Paket: Das Programm liefert sie aus seiner SQLite, und nur
  // für die Module, die es trägt.
  dataSource: { type: Object, required: true },
});
// Wohin ein Klick führt: mit Router (Webapp) als Link, ohne (Programm) als
// Ereignis mit demselben Ziel – die Anwendung übersetzt es in ihren Ort.
const emit = defineEmits(['oeffnen']);
const Link = inject('oberflaeche:link', null);
const api = props.dataSource;

const loading = ref(true);
const error = ref('');
const data = ref(null);

const load = async () => {
  loading.value = true; error.value = '';
  try {
    data.value = (await api.getDashboard()).data;
  } catch (e) {
    error.value = t('home.dashboard.loadFailed');
  } finally { loading.value = false; }
};
onMounted(load);

const show = (id) => props.moduleIds.includes(id);

// Kalender-Widget: Wochenansicht der laufenden Woche (Mo–So). Serien
// werden aufgefaltet, je Tag die überlappenden Termine (max. 3 + Zähler).
const weekDays = computed(() => {
  if (!data.value) return [];
  const expanded = expandRecurringEvents(data.value.events);
  const monday = new Date();
  monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7));
  monday.setHours(0, 0, 0, 0);
  const todayStr = getLocalDateString(new Date());

  return Array.from({ length: 7 }, (_, i) => {
    const date = new Date(monday);
    date.setDate(monday.getDate() + i);
    const dateString = getLocalDateString(date);
    const events = expanded
      .filter((e) => {
        const start = getLocalDateString(new Date(e.start_date));
        const end = getLocalDateString(new Date(e.end_date));
        return start <= dateString && end >= dateString;
      })
      .sort((a, b) => (b.is_all_day - a.is_all_day) || a.start_date.localeCompare(b.start_date));
    return { date, dateString, events, isToday: dateString === todayStr };
  });
});
const weekHasEvents = computed(() => weekDays.value.some((d) => d.events.length));
const calendarColor = (calendarId) =>
  data.value?.calendars.find((c) => c.id === calendarId)?.color || DEFAULT_CALENDAR_COLOR;

const dayLabel = (d) => formatWeekday(d);
const eventTime = (e) => (e.is_all_day ? '' : formatTime(e.start_date));

// Speicher: Belegungs-Balken (quota null = unbegrenzt).
const storagePercent = computed(() => {
  const st = data.value?.storage;
  if (!st?.quota) return null;
  return Math.min(100, Math.round((st.used / st.quota) * 100));
});
const zoneIcon = (zone) => (zone === 'documents' ? Archive : Folder);
const zoneLink = (zone) => (zone === 'documents' ? '/files#dokumente' : '/files#dateien');

// Nachrichten: Absender ist bei eingegangenen Matrix-Nachrichten kein Konto
// hier – dann steht die fremde Kennung in `peer`.
const senderName = (m) => m.sender?.name || m.peer || t('home.dashboard.unknownSender');

// Adressbuch: Die Kachel zeigt keine Kontakte, sondern führt zur Suche.
const kontaktSuche = ref('');
const kontakteSuchen = () => {
  // Kein Link, sondern ein Formular: Hier meldet die Kachel das Ziel immer,
  // auch in der Webapp, und die Anwendung navigiert.
  emit('oeffnen', { path: '/contacts', query: { suche: kontaktSuche.value.trim() } });
};
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <div v-if="loading" class="flex justify-center py-20"><Loader2 class="w-8 h-8 text-marke animate-spin" /></div>
    <p v-else-if="error" class="py-20 text-center text-sm font-medium text-rose-500">{{ error }}</p>

    <!-- CSS-Spaltenlayout statt Zeilen-Raster: unterschiedlich hohe
         Kacheln schließen lückenlos aneinander an (Masonry-Effekt). -->
    <div v-else class="columns-1 md:columns-2 gap-6">

      <!-- Kalender: nächste Termine -->
      <section v-if="show('calendar')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-marke-leise text-marke flex items-center justify-center"><CalendarDays class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.calendarTitle') }}
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/calendar' : undefined" @click="Link || emit('oeffnen', '/calendar')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="!weekHasEvents" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noEventsWeek') }}</p>
        <div class="divide-y divide-slate-100 dark:divide-slate-800/60">
          <component :is="Link ?? 'button'" v-for="day in weekDays" :key="day.dateString" :to="Link ? '/calendar' : undefined" @click="Link || emit('oeffnen', '/calendar')"
            class="w-full text-left flex items-start gap-3 px-2 py-1.5 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
            :class="day.isToday ? 'bg-marke-leise/60' : ''">
            <span class="w-12 shrink-0 pt-0.5 leading-tight"
              :class="day.isToday ? 'text-marke' : 'text-slate-400'">
              <span class="block text-[10px] font-extrabold uppercase tracking-wide">{{ dayLabel(day.date) }}</span>
              <span class="block text-sm font-extrabold">{{ day.date.getDate() }}.</span>
            </span>
            <span class="flex-1 min-w-0 py-0.5">
              <span v-if="day.events.length === 0" class="block text-xs text-slate-300 dark:text-slate-600 font-medium">–</span>
              <span v-for="e in day.events.slice(0, 3)" :key="e.id" class="flex items-center gap-1.5 min-w-0">
                <span class="w-2 h-2 rounded-full shrink-0" :style="{ backgroundColor: calendarColor(e.calendar_id) }"></span>
                <span class="flex-1 min-w-0 text-xs font-bold text-schrift truncate">{{ e.title }}</span>
                <span v-if="eventTime(e)" class="text-[10px] font-semibold text-slate-400 shrink-0">{{ eventTime(e) }}</span>
              </span>
              <span v-if="day.events.length > 3" class="block text-[10px] font-bold text-slate-400 mt-0.5">{{ t('home.dashboard.moreEvents', { count: day.events.length - 3 }) }}</span>
            </span>
          </component>
        </div>
      </section>

      <!-- Notizen: zuletzt bearbeitet -->
      <section v-if="show('notes')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-auflage text-fliess flex items-center justify-center"><FileText class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.notesTitle') }}
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/notes' : undefined" @click="Link || emit('oeffnen', '/notes')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="data.notes.length === 0" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noNotes') }}</p>
        <ul v-else class="space-y-1">
          <li v-for="n in data.notes" :key="n.id">
            <component :is="Link ?? 'button'" :to="Link ? `/notes?note=${n.id}` : undefined" @click="Link || emit('oeffnen', `/notes?note=${n.id}`)" class="w-full text-left flex items-center gap-3 px-2 py-2 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
              <FileText class="w-4 h-4 text-slate-400 shrink-0" />
              <span class="flex-1 min-w-0 text-sm font-bold text-schrift truncate">{{ n.title || t('home.dashboard.untitledNote') }}</span>
              <span class="text-xs font-semibold text-slate-400 shrink-0">{{ formatDateShort(n.updated_at) }}</span>
            </component>
          </li>
        </ul>
      </section>

      <!-- Speicher: letzte Dateien + Belegung -->
      <section v-if="show('files')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-marke-leise text-marke flex items-center justify-center"><HardDrive class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.storageTitle') }}
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/files' : undefined" @click="Link || emit('oeffnen', '/files')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="data.files.length === 0" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noFiles') }}</p>
        <ul v-else class="space-y-1">
          <li v-for="f in data.files" :key="f.id">
            <component :is="Link ?? 'button'" :to="Link ? zoneLink(f.zone) : undefined" @click="Link || emit('oeffnen', zoneLink(f.zone))" class="w-full text-left flex items-center gap-3 px-2 py-2 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
              <component :is="zoneIcon(f.zone)" class="w-4 h-4 text-slate-400 shrink-0" />
              <span class="flex-1 min-w-0 text-sm font-bold text-schrift truncate">{{ f.name }}</span>
              <span class="text-xs font-semibold text-slate-400 shrink-0">{{ formatBytes(f.size) }}</span>
            </component>
          </li>
        </ul>
        <div class="mt-4 pt-4 border-t border-linie">
          <div class="flex items-center justify-between text-xs font-bold text-leise mb-1.5">
            <span>{{ t('home.dashboard.storageUsed', { used: formatBytes(data.storage.used) }) }}</span>
            <span v-if="storagePercent !== null">{{ storagePercent }} %</span>
            <span v-else>{{ t('home.dashboard.storageUnlimited') }}</span>
          </div>
          <div v-if="storagePercent !== null" class="h-2 rounded-full bg-auflage overflow-hidden">
            <div class="h-full rounded-full bg-marke transition-all" :style="{ width: storagePercent + '%' }"></div>
          </div>
        </div>
      </section>

      <!-- Nachrichten: Posteingang mit Ungelesen-Zähler -->
      <section v-if="show('messages')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-marke-leise text-marke flex items-center justify-center"><Mail class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.messagesTitle') }}
            <span v-if="data.messages.unread > 0" class="px-1.5 py-0.5 text-[10px] font-extrabold bg-rose-500 text-white rounded-full leading-none">{{ t('home.dashboard.unreadCount', { count: data.messages.unread }) }}</span>
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/messages' : undefined" @click="Link || emit('oeffnen', '/messages')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="data.messages.latest.length === 0" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noMessages') }}</p>
        <ul v-else class="space-y-1">
          <li v-for="m in data.messages.latest" :key="m.id">
            <component :is="Link ?? 'button'" :to="Link ? '/messages' : undefined" @click="Link || emit('oeffnen', '/messages')" class="w-full text-left flex items-start gap-3 px-2 py-2 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
              <span class="w-2 h-2 mt-1.5 rounded-full shrink-0" :class="m.read_at ? 'bg-transparent' : 'bg-rose-500'"></span>
              <span class="flex-1 min-w-0">
                <span class="flex items-center gap-2 min-w-0">
                  <span class="flex-1 min-w-0 text-sm truncate" :class="m.read_at ? 'font-bold text-fliess' : 'font-extrabold text-schrift'">{{ senderName(m) }}</span>
                  <span class="text-xs font-semibold text-slate-400 shrink-0">{{ formatDateShort(m.created_at) }}</span>
                </span>
                <span class="block text-xs text-leise truncate">{{ m.body }}</span>
              </span>
            </component>
          </li>
        </ul>
      </section>

      <!-- Adressbuch: Suchfeld, Anzahl, neuer Kontakt -->
      <section v-if="show('contacts')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-auflage text-fliess flex items-center justify-center"><Contact class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.contactsTitle') }}
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/contacts' : undefined" @click="Link || emit('oeffnen', '/contacts')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="data.contacts.total === 0" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noContacts') }}</p>
        <!-- Keine Namen auf der Startseite, nur der Weg zur Suche: Getippt
             wird hier, gesucht drüben (Contacts.vue liest `?suche`). -->
        <form v-else role="search" class="relative" @submit.prevent="kontakteSuchen">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
          <input v-model="kontaktSuche" type="search" :placeholder="t('contacts.searchPlaceholder')"
            :aria-label="t('contacts.search')"
            autocapitalize="none" autocorrect="off" spellcheck="false"
            class="w-full pl-9 pr-10 py-2 rounded-xl border border-linie bg-flaeche text-schrift text-sm" />
          <button type="submit" :aria-label="t('contacts.search')"
            class="absolute right-1.5 top-1/2 -translate-y-1/2 p-1.5 rounded-lg text-marke hover:bg-marke-leise cursor-pointer">
            <ArrowRight class="w-4 h-4" />
          </button>
        </form>
        <div class="mt-4 pt-4 border-t border-linie flex items-center justify-between text-xs font-bold text-leise">
          <span>{{ t('home.dashboard.contactsTotal', { count: data.contacts.total }, data.contacts.total) }}</span>
          <component :is="Link ?? 'button'" :to="Link ? { path: '/contacts', query: { neu: '1' } } : undefined" @click="Link || emit('oeffnen', { path: '/contacts', query: { neu: '1' } })" class="flex items-center gap-1 text-marke hover:underline"><UserPlus class="w-3.5 h-3.5" /> {{ t('home.dashboard.newContact') }}</component>
        </div>
      </section>

      <!-- Projekte: Ungelesenes + Aktivität -->
      <section v-if="show('projects')" class="karte shadow-sm p-6 transition-colors mb-6 break-inside-avoid">
        <div class="flex items-center justify-between mb-4">
          <h2 class="flex items-center gap-2.5 font-extrabold text-schrift">
            <span class="w-9 h-9 rounded-xl bg-marke-leise text-marke flex items-center justify-center"><Briefcase class="w-4.5 h-4.5" /></span>
            {{ t('home.dashboard.projectsTitle') }}
          </h2>
          <component :is="Link ?? 'button'" :to="Link ? '/projects' : undefined" @click="Link || emit('oeffnen', '/projects')" class="flex items-center gap-1 text-xs font-bold text-marke hover:underline">{{ t('home.dashboard.showAll') }} <ArrowRight class="w-3.5 h-3.5" /></component>
        </div>
        <p v-if="data.projects.length === 0" class="py-6 text-center text-sm text-slate-400">{{ t('home.dashboard.noProjects') }}</p>
        <ul v-else class="space-y-1">
          <li v-for="p in data.projects" :key="p.id">
            <component :is="Link ?? 'button'" :to="Link ? `/projects/${p.id}` : undefined" @click="Link || emit('oeffnen', `/projects/${p.id}`)" class="w-full text-left flex items-center gap-3 px-2 py-2 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
              <span class="flex-1 min-w-0 text-sm font-bold text-schrift truncate">{{ p.name }}</span>
              <span v-if="p.unread_messages > 0" class="px-1.5 py-0.5 text-[10px] font-extrabold bg-rose-500 text-white rounded-full leading-none shrink-0">{{ p.unread_messages }}</span>
              <span class="flex items-center gap-1 text-xs font-semibold text-slate-400 shrink-0"><Users class="w-3.5 h-3.5" /> {{ p.members_count }}</span>
            </component>
          </li>
        </ul>
      </section>

    </div>
  </div>
</template>
