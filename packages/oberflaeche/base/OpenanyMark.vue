<script setup>
// DAS ZEICHEN VON OPENANY ALS SYMBOL -- Kreis mit Punkt.
//
// Es gibt das Zeichen zweimal, und das ist Absicht:
//
//   In der Kopfleiste (App.vue) ist es ein <div> mit `bg-marke` und einem
//   pulsierenden Punkt. Es lebt: Der Puls hoert auf, wenn das Netz weg ist.
//   Das ist eine Anzeige und kein Sinnbild.
//
//   HIER ist es ein Symbol unter Symbolen -- in einer Kontaktzeile neben
//   Telefonhoerer und Briefumschlag. Es muss sich benehmen wie die anderen:
//   dieselbe Groesse ueber `class`, dieselbe Farbe ueber `currentColor`,
//   keine Bewegung.
//
// FARBLOS, ausdruecklich. Ein markenfarbener Klecks in einer Reihe grauer
// Strichzeichen zoege den Blick auf die Zeile, die am seltensten vorkommt.
// `currentColor` heisst: Das Symbol nimmt die Farbe, die an der Stelle gilt
// -- grau in der Liste, markenfarben, wo etwas anklickbar ist.
//
// AUFGEBAUT WIE EIN lucide-SYMBOL (24er-Raster, `stroke-width: 2`), damit es
// sich austauschen laesst wie eines. Vorher stand hier `AtSign` -- ein
// Klammeraffe fuer einen openany-Namen ist die naheliegende, aber falsche
// Anleihe: `@` heisst E-Mail oder Mastodon, und beides ist es nicht.
//
// Der Punkt ist gefuellt und traegt `stroke="none"`: Sonst waere er bei
// 16 Pixeln ein Ring im Ring statt eines Punktes.
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    width="24" height="24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="9" />
    <circle cx="12" cy="12" r="3.25" fill="currentColor" stroke="none" />
  </svg>
</template>
