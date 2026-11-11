<script setup>
/*
 * „Über openany": Datenschutz und AGB kommen vom Server, die Lizenzen liegen in
 * der App (scripts/lizenzen.mjs). Das Impressum steht am Ende der Hilfe.
 *
 * Ohne Netz zeigt die Seite die zuletzt geholte Fassung eines Textes. Sie ist nur
 * eine Bequemlichkeit, keine Ablage: Fehlt sie, steht dort, dass der Text gerade
 * nicht erreichbar ist.
 */
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { getVersion } from '@tauri-apps/api/app';
import { useMindestversion } from '../mindestversion';
import { ChevronLeft, ChevronRight, FileText, Info, Loader2, Scale, ShieldCheck } from 'lucide-vue-next';
import RechtstextAnzeige from '@oberflaeche/shared/RechtstextAnzeige.vue';
import ZurueckSchliesst from '@oberflaeche/base/ZurueckSchliesst.vue';
import { gemerkterRechtstext, rechtstextHolen } from '../quellen/rechtstext';

const { t, locale } = useI18n();

const EINTRAEGE = [
  { id: 'datenschutz', icon: ShieldCheck },
  { id: 'agb', icon: FileText },
  { id: 'lizenzen', icon: Scale },
];
const nichtDeutsch = computed(() => !String(locale.value).startsWith('de'));

const version = ref('');
getVersion().then((v) => { version.value = v; }).catch(() => {});
const mindestversion = useMindestversion();

const offen = ref(null);
const laedt = ref(false);
const text = ref('');
const veraltet = ref(false);
const fehlt = ref(false);

async function textHolen(art) {
  laedt.value = true;
  veraltet.value = false;
  fehlt.value = false;
  text.value = gemerkterRechtstext(art);
  try {
    const neu = await rechtstextHolen(art);
    if (offen.value !== art) return;
    text.value = neu;
  } catch {
    if (offen.value !== art) return;
    veraltet.value = Boolean(text.value);
    fehlt.value = !text.value;
  } finally {
    if (offen.value === art) laedt.value = false;
  }
}

/* ── Lizenzen ──────────────────────────────────────────────────────────── */

const lizenzen = ref(null);
const lizenzenFehlen = ref(false);
const suche = ref('');
const aufgeklappt = ref(null);

async function lizenzenHolen() {
  if (lizenzen.value) return;
  laedt.value = true;
  try {
    const antwort = await fetch('/lizenzen.json');
    if (!antwort.ok) throw new Error(String(antwort.status));
    lizenzen.value = await antwort.json();
  } catch {
    lizenzenFehlen.value = true;
  } finally {
    laedt.value = false;
  }
}

const treffer = computed(() => {
  const q = suche.value.trim().toLowerCase();
  const alle = lizenzen.value?.pakete ?? [];
  return q ? alle.filter((p) => `${p.name} ${p.lizenz}`.toLowerCase().includes(q)) : alle;
});
const SICHTBAR = 100;
const sichtbar = computed(() => treffer.value.slice(0, SICHTBAR));

watch(offen, (art) => {
  if (art === 'lizenzen') lizenzenHolen();
  else if (art) textHolen(art);
});
const titel = (id) => t(`app.ueber.${id}`);
const schluessel = (p) => `${p.art}:${p.name}:${p.version}`;
</script>

<template>
  <div class="max-w-3xl mx-auto px-4 sm:px-6 py-6 space-y-4">
    <ZurueckSchliesst v-if="offen" @schliessen="offen = null" />

    <template v-if="!offen">
      <header class="karte shadow-sm p-5">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-marke-leise flex items-center justify-center text-marke"><Info class="w-5 h-5" /></div>
          <div>
            <h1 class="text-2xl font-extrabold text-schrift tracking-tight">{{ t('app.ueber.titel') }}</h1>
            <p v-if="version" class="text-xs text-leise">
              {{ t('help.version', { version }) }}
              <span v-if="mindestversion" class="ml-2 font-bold text-warnung" :title="t('help.aktualisierenTitel', { version: mindestversion })">{{ t('help.aktualisieren') }}</span>
            </p>
          </div>
        </div>
      </header>
      <nav class="karte shadow-sm divide-y divide-linie">
        <button v-for="e in EINTRAEGE" :key="e.id" type="button" class="w-full flex items-center gap-3 px-5 py-4 text-left cursor-pointer" @click="offen = e.id">
          <component :is="e.icon" class="w-5 h-5 text-marke shrink-0" />
          <span class="flex-1 text-sm font-bold text-schrift">{{ titel(e.id) }}</span>
          <ChevronRight class="w-4 h-4 text-slate-400" />
        </button>
      </nav>
    </template>

    <template v-else>
      <button type="button" class="flex items-center gap-1 text-sm font-bold text-marke cursor-pointer" @click="offen = null">
        <ChevronLeft class="w-4 h-4" /> {{ t('app.ueber.titel') }}
      </button>

      <section v-if="offen === 'lizenzen'" class="karte shadow-sm p-5 sm:p-8 space-y-4">
        <h1 class="text-2xl font-extrabold text-schrift tracking-tight">{{ titel('lizenzen') }}</h1>
        <div v-if="laedt" class="flex justify-center py-8"><Loader2 class="w-8 h-8 text-marke animate-spin" /></div>
        <p v-else-if="lizenzenFehlen" class="text-sm text-leise">{{ t('app.ueber.lizenzenFehlen') }}</p>
        <template v-else>
          <p class="text-sm text-fliess">{{ t('app.ueber.lizenzenText', { anzahl: lizenzen?.pakete.length ?? 0 }) }}</p>
          <input v-model="suche" type="search" :placeholder="t('app.ueber.suchen')" class="w-full bg-vertieft border border-linie rounded-xl px-4 py-2.5 text-sm outline-none focus:ring-2 focus:ring-marke text-schrift" />
          <ul class="divide-y divide-linie">
            <li v-for="p in sichtbar" :key="schluessel(p)">
              <button type="button" class="w-full py-2.5 flex items-baseline justify-between gap-3 text-left cursor-pointer" @click="aufgeklappt = aufgeklappt === schluessel(p) ? null : schluessel(p)">
                <span class="min-w-0 text-sm font-bold text-schrift break-words">{{ p.name }} <span class="font-medium text-leise">{{ p.version }}</span></span>
                <span class="shrink-0 max-w-[50%] text-right text-xs text-leise">{{ p.lizenz }}</span>
              </button>
              <template v-if="aufgeklappt === schluessel(p)">
                <p v-if="p.urheber" class="mb-2 text-xs text-fliess break-words">{{ p.urheber }}</p>
                <p v-if="p.quelle" class="mb-2 text-xs text-leise break-all">{{ p.quelle }}</p>
                <p v-if="p.standard" class="mb-2 text-xs text-leise">{{ t('app.ueber.standardText') }}</p>
                <p v-if="p.nur" class="mb-2 text-xs text-leise">{{ t('app.ueber.nurPlattform', { plattform: p.nur }) }}</p>
                <pre v-for="i in p.texte" :key="i" class="mb-3 p-3 rounded-xl bg-vertieft text-xs text-fliess whitespace-pre-wrap break-words">{{ lizenzen.texte[i] }}</pre>
              </template>
            </li>
          </ul>
          <p v-if="treffer.length > SICHTBAR" class="text-xs text-leise">{{ t('app.ueber.mehrTreffer', { anzahl: treffer.length - SICHTBAR }) }}</p>
        </template>
      </section>

      <section v-else class="karte shadow-sm p-5 sm:p-8 space-y-4">
        <p v-if="nichtDeutsch" class="rounded-xl bg-vertieft p-3 text-sm font-semibold text-schrift">{{ t('app.ueber.nurDeutsch') }}</p>
        <p v-if="veraltet" class="rounded-xl bg-vertieft p-3 text-sm text-schrift">{{ t('app.ueber.veraltet') }}</p>
        <div v-if="laedt && !text" class="flex justify-center py-8"><Loader2 class="w-8 h-8 text-marke animate-spin" /></div>
        <p v-else-if="fehlt" class="text-sm text-leise">{{ t('app.ueber.nichtErreichbar') }}</p>
        <RechtstextAnzeige v-else :text="text" />
      </section>
    </template>
  </div>
</template>
