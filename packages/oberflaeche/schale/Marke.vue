<script setup>
// Das Zeichen von openany: der Kreis mit dem Punkt.
//
// Die tiefgestellte Wortmarke „openany" daneben ist am 08.09.2026 gefallen;
// das Zeichen trägt allein.
//
// DER PULS IST EINE AUSSAGE, KEIN SCHMUCK: Er heißt „die Verbindung steht".
// Woher das kommt, weiß die Anwendung – die Webapp aus dem Netzzustand, das
// Programm aus dem letzten gelungenen Abgleich.
defineProps({
  verbunden: { type: Boolean, default: true },
  // 'kopf' (Kopfzeile am Schreibtisch, mit Schein) | 'blatt' (Menü-Blatt)
  groesse: { type: String, default: 'kopf' },
});
</script>

<template>
  <div class="rounded-full bg-marke flex items-center justify-center text-white shrink-0"
       :class="[groesse === 'blatt' ? 'w-9 h-9' : 'w-10 h-10', groesse === 'kopf' && verbunden ? 'animate-pulse-glow' : '']">
    <span class="rounded-full"
          :class="[groesse === 'blatt' ? 'w-3 h-3' : 'w-3.5 h-3.5', verbunden ? 'animate-pulse-white' : 'bg-white/70']"></span>
  </div>
</template>
