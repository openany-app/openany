<script setup>
/*
 * Notizen — die Arbeitsfläche der Webapp, mit der lokalen SQLite dahinter.
 *
 * Bis zum 15.09.2026 stand hier eine eigene, kurze Ansicht (Liste und
 * Textfeld). Jetzt ist es dieselbe Komponente wie drüben: Mappen, Suche,
 * Tags, [[Verweise]], Rückverweise, Markdown-Editor mit Autosave.
 *
 * Von den `erweiterungen` gibt es hier das Einfügen (seit 17.09.2026,
 * eigener Dialog: eine Datei vom Gerät) und den Graphen (seit 30.09.2026,
 * aus dem eigenen Speicher, auch ohne Netz). Teilen und KI brauchen einen
 * Server und fehlen — mit ihren Knöpfen.
 */
import { defineAsyncComponent } from 'vue';
import NotesWorkspace from '@oberflaeche/notizen/NotesWorkspace.vue';
import { lokaleNotizenQuelle } from '../quellen/notizen';
import AnhangEinfuegen from './AnhangEinfuegen.vue';

const quelle = lokaleNotizenQuelle();
const erweiterungen = {
    einfuegen: AnhangEinfuegen,
    graph: defineAsyncComponent(() => import('@oberflaeche/notizen/NotesGraph.vue')),
};
</script>

<template>
  <NotesWorkspace :data-source="quelle" :erweiterungen="erweiterungen" />
</template>
