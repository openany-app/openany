<script setup>
// Nachrichtentext des Projekt-Chats: Text, Web-Adressen und [[Verweise]] auf
// alles im Projekt Verlinkbare – Notizen, Planungs-Behälter samt ihrer
// Einzelpunkte sowie freigegebene Dateien, Ordner, Alben und Bilder.
//
// Gerendert wird über v-for, NIE über v-html – die Maskierung bleibt damit
// Vues Aufgabe, und kein künftiger Zusatz kann versehentlich ein XSS-Loch
// aufreißen. Die Zerlegung selbst steht in shared/chatText.js und ist dort
// unabhängig von Vue getestet.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { parseChatText, linkKey, SHARED_KINDS } from './chatText';

const props = defineProps({
  body: { type: String, default: '' },
  // Karte „Linkziel (klein) => { kind, label, … }" aus der Chat-Antwort.
  // Was hier fehlt, ist in diesem Projekt nicht sichtbar.
  linkTargets: { type: Object, default: () => ({}) },
});

const emit = defineEmits(['open-target']);
const { t } = useI18n();

const segments = computed(() => parseChatText(props.body));

const ziel = (target) => props.linkTargets?.[linkKey(target)] ?? null;

/**
 * Was am Verweis steht. Ein Anzeigetext in der Nachricht gewinnt – die
 * Vorschlagsliste setzt ihn mit ein, damit im Eingabefeld lesbar steht, was
 * man gerade verlinkt hat, und damit die Nachricht als Aufzeichnung dessen
 * heil bleibt, was jemand geschrieben hat. Fehlt er (von Hand getippt), tritt
 * der aktuelle Name aus der Karte ein.
 */
const anzeige = (s) => s.label || ziel(s.target)?.label || s.target;

const oeffne = (target) => {
  const z = ziel(target);
  if (z) emit('open-target', z);
};

// Der Hinweis am toten Verweis hängt an der Sichtbarkeitsgrenze der Art
// (siehe SHARED_KINDS). Ein Titel-Verweis ohne Art meint immer eine Notiz.
const totTitel = (s) => (s.kind === null || SHARED_KINDS.includes(s.kind)
  ? t('projects.chat.noteNotShared')
  : t('projects.chat.targetNotFound'));

// Farben werden bewusst GEERBT (kein text-…): Der Text steht mal auf
// der Markenfarbe (eigene Nachricht, weiß) und mal auf slate-100/800. Eine feste
// Linkfarbe wäre auf einem der beiden Hintergründe unlesbar.
const LINK = 'underline underline-offset-2 font-bold cursor-pointer hover:opacity-75 transition-opacity';
const TOT = 'underline decoration-dotted underline-offset-2 opacity-70 cursor-help';
</script>

<template><span><template v-for="(s, i) in segments" :key="i"><a
  v-if="s.type === 'url'"
  :href="s.href"
  target="_blank"
  rel="noopener noreferrer nofollow"
  :class="LINK"
>{{ s.label }}</a><button
  v-else-if="s.type === 'wikilink' && ziel(s.target)"
  type="button"
  :class="LINK"
  :title="t('projects.chat.openTarget', { title: anzeige(s) })"
  @click="oeffne(s.target)"
>{{ anzeige(s) }}</button><span
  v-else-if="s.type === 'wikilink'"
  :class="TOT"
  :title="totTitel(s)"
>{{ anzeige(s) }}</span><template v-else>{{ s.value }}</template></template></span></template>
