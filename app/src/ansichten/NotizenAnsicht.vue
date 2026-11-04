<script setup>
/*
 * Notizen — die Arbeitsfläche aus dem gemeinsamen Paket, mit der lokalen
 * SQLite dahinter: Mappen, Suche, Tags, [[Verweise]], Rückverweise,
 * Markdown-Editor mit Autosave.
 *
 * Von den `erweiterungen` gibt es hier das Einfügen (eigener Dialog: eine
 * Datei vom Gerät) und den Graphen (aus dem eigenen Speicher, auch ohne
 * Netz).
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
