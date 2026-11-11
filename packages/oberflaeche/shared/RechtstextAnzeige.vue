<script setup>
// Zeigt einen Rechtstext (rechtstext.js). Nur die Form steht im Paket, die
// Texte kommen vom Server.
import { computed } from 'vue';
import { rechtstextBloecke } from './rechtstext';

const props = defineProps({
  text: { type: String, default: '' },
  // Wo die Umgebung den Titel schon zeigt.
  ohneTitel: { type: Boolean, default: false },
});
const bloecke = computed(() => rechtstextBloecke(props.text).filter((b) => !props.ohneTitel || b.art !== 'h1'));
</script>

<template>
  <div class="space-y-3 text-sm text-fliess leading-relaxed">
    <template v-for="(b, i) in bloecke" :key="i">
      <h1 v-if="b.art === 'h1'" class="text-2xl font-extrabold text-schrift tracking-tight">{{ b.zeilen[0].map((t) => t.text).join('') }}</h1>
      <h2 v-else-if="b.art === 'h2'" class="pt-3 text-base font-extrabold text-schrift">{{ b.zeilen[0].map((t) => t.text).join('') }}</h2>
      <ul v-else-if="b.art === 'ul'" class="list-disc pl-5 space-y-1">
        <li v-for="(z, j) in b.zeilen" :key="j"><template v-for="(t, k) in z" :key="k"><strong v-if="t.fett">{{ t.text }}</strong><template v-else>{{ t.text }}</template></template></li>
      </ul>
      <p v-else>
        <template v-for="(z, j) in b.zeilen" :key="j"><br v-if="j" /><template v-for="(t, k) in z" :key="k"><strong v-if="t.fett">{{ t.text }}</strong><template v-else>{{ t.text }}</template></template></template>
      </p>
    </template>
  </div>
</template>
