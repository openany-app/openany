<script setup>
// Das Adressbuch: wer, und wie erreichbar.
//
// Bewusst schlicht — eine Liste, ein Dialog, sonst nichts. Ein Kontakt hält
// einen Namen fest und beliebig viele WEGE (Nummer, E-Mail, openany-Name,
// Anschrift, Betrieb, Matrix, Meshtastic, Sonstiges), dazu optional einen
// Verweis auf ein openany-Konto. Er ist KEINE Berechtigung: Wer ein Projekt
// teilen darf, entscheidet weiter das Projekt.
//
// Was hier NICHT steht, gehört auch nicht her: die im lokalen Netz gefundene
// Adresse. Sie gilt nur hier und jetzt und wanderte über den Abgleich als
// Müll auf die anderen Geräte. Erreichbarkeit ist Laufzeitzustand.
//
// SEIT DEM 15.09.2026 IM GEMEINSAMEN PAKET, mit einer Datenquelle statt der
// API – dasselbe Muster wie Notizen und Kalender. Was vom Rahmen abhängt,
// kommt als Eigenschaft herein: ob es Nachrichten gibt, ob Matrix verbunden
// ist, ob das Formular gleich offen sein soll. Die Webapp setzt das aus
// Route und Sitzung (views/Contacts.vue), das Programm aus seinem Zustand.
import { ref, computed, onMounted, nextTick, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import BaseModal from '@oberflaeche/base/BaseModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { Contact as ContactIcon, Plus, Pencil, Trash2, Loader2, X, ChevronRight, Search, Camera, Phone, Mail, MapPin, Building2, Radio, MessageSquare, Cake, Hash } from 'lucide-vue-next';
import OpenanyMark from '@oberflaeche/base/OpenanyMark.vue';
import { WEG_ARTEN, OFFENE_FELDER, VCARD_FELDER, wegZiel, nachrichtenWeg, vcardName, istDurchgereicht } from '@oberflaeche/shared/kontaktwege';
import { passt } from '@oberflaeche/shared/textsuche';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';

const props = defineProps({
  // getContacts, createContact, updateContact, deleteContact,
  // uploadContactPhoto, deleteContactPhoto, contactPhotoUrl(id, hash).
  // Ein Kontaktfoto darf statt über contactPhotoUrl auch eine fertige
  // `photo.url` mitbringen (das Programm liefert Bilder aus der Datei).
  dataSource: { type: Object, required: true },
  // Gibt es Nachrichten, zu denen ein Weg führen kann? Ohne sie fehlt der
  // Knopf dorthin, und ein openany-Name ist schlichter Text.
  nachrichten: { type: Boolean, default: false },
  // Nur mit verbundenem Konto führt eine Matrix-Kennung irgendwohin.
  matrixVerbunden: { type: Boolean, default: false },
  // Beim Öffnen gleich das Formular (`?neu`) bzw. die Suche (`?suche=`).
  startNeu: { type: Boolean, default: false },
  startSuche: { type: String, default: null },
});
// `schreiben` trägt `{ kanal, kennung }` — die Absicht, dieser Gegenstelle
// zu schreiben. Nur Rahmen ohne Router hören darauf (siehe `alsRoute`).
const emit = defineEmits(['nachrichten', 'start-verbraucht', 'schreiben']);
const api = props.dataSource;
const Link = inject('oberflaeche:link', null);

/** Die Adresse des Fotos – fertig aus der Quelle oder über contactPhotoUrl. */
const fotoUrl = (kontakt) => kontakt.photo?.url ?? api.contactPhotoUrl?.(kontakt.id, kontakt.photo?.hash) ?? null;

const { t } = useI18n();
const toast = useToast();
const { confirmDelete } = useConfirm();

const kontakte = ref([]);

/*
 * Gesucht wird HIER und nicht auf dem Server.
 *
 * Die Liste ist ohnehin vollständig geladen – ein Endpunkt je Tastendruck
 * wäre eine Netzwerkrunde für eine Antwort, die schon im Speicher liegt, und
 * auf einer Mobilfunkleitung eine spürbare. Wenn ein Adressbuch einmal so
 * groß wird, dass das Laden selbst zu lange dauert, ist die Suche nicht die
 * Stelle, an der man anfängt.
 *
 * GESUCHT WIRD IM NAMEN. Nicht in den Wegen: Wer „017" tippt, bekäme sonst
 * jeden zweiten Kontakt, und die Suche wäre in dem Moment nutzlos, in dem
 * man sie braucht. Die Frage „wer ruft da an" ist eine andere und braucht
 * eine andere Antwort.
 */
const suche = ref('');

const gefiltert = computed(() => kontakte.value.filter((k) => passt(k.display_name, suche.value)));
const laedt = ref(true);
const speichert = ref(false);
const dialogOffen = ref(false);
const bearbeitet = ref(null);
/*
 * Das Formular hat zwei Teile, und beide schreiben in DIESELBE Tabelle.
 *
 * `offen` sind die zwei Angaben, die fast jeder hat – Nummer und E-Mail. Sie
 * stehen als eigene Felder da, damit ein neuer Kontakt nicht mit einem Klick
 * auf „Weg" und einem Griff ins Auswahlfeld anfängt.
 *
 * `channels` ist alles Übrige, samt zweiter und dritter Nummer. Die zwei
 * Arten oben stehen deshalb AUCH im Auswahlfeld: Es ist eine Abkürzung, kein
 * zweiter Mechanismus.
 *
 * Beim Laden wird je Art der ERSTE Weg nach oben geholt, der Rest bleibt
 * unten stehen; beim Speichern läuft es zusammen, oben zuerst.
 */
const formular = ref({ display_name: '', offen: {}, channels: [] });

/*
 * Das Bild wird ERST BEIM SPEICHERN geschickt.
 *
 * Sofort hochzuladen ginge nur bei einem Kontakt, den es schon gibt — ein
 * neuer hat noch keine Id. Zwei Wege für dieselbe Handlung wären zwei Wege,
 * die auseinanderlaufen; und ein Bild, das schon oben ist, während der
 * Mensch auf „Abbrechen" drückt, ist eine Überraschung.
 */
const bildfeld = ref(null);
const bildDatei = ref(null);
const bildWeg = ref(false);
const bildVorschau = ref(null);

/** Die Anfangsbuchstaben, wenn es kein Bild gibt. */
const initialen = (name) => String(name ?? '')
  .split(/\s+/).filter(Boolean).slice(0, 2)
  .map((teil) => teil[0].toUpperCase()).join('');

const bildGewaehlt = (ereignis) => {
  const datei = ereignis.target.files?.[0];
  if (!datei) return;

  bildDatei.value = datei;
  bildWeg.value = false;
  // Vorschau aus dem Browser und nicht vom Server: Sie soll da sein, bevor
  // irgendetwas hochgeladen ist.
  bildVorschau.value = URL.createObjectURL(datei);
  ereignis.target.value = '';
};

const bildEntfernen = () => {
  bildDatei.value = null;
  bildVorschau.value = null;
  bildWeg.value = true;
};

// Je Art ein Sinnbild – dieselbe Zuordnung in der Liste wie im Formular, damit
// eine Zeile nach dem Speichern nicht plötzlich anders aussieht.
const SINNBILD = {
  phone: Phone,
  email: Mail,
  // DAS ZEICHEN, NICHT DER KLAMMERAFFE (10.09.2026). `AtSign` war die
  // naheliegende und falsche Anleihe: `@` heisst E-Mail oder Mastodon, und
  // ein openany-Name ist weder das eine noch das andere.
  openany: OpenanyMark,
  address: MapPin,
  company: Building2,
  birthday: Cake,
  matrix: MessageSquare,
  meshtastic: Radio,
  other: Hash,
};

/*
 * Wie eine Art heißt.
 *
 * Für die eigenen Arten gibt es eine Übersetzung. Für ein durchgereichtes
 * Feld gibt es keine, und es soll auch keine erfundene geben: Es zeigt
 * seinen vCard-Namen, so wie er in der Datei steht. Wer wissen will, was
 * `X-ABRELATEDNAMES` ist, kann danach suchen — bei einem ausgedachten
 * deutschen Wort könnte er das nicht.
 */
const artName = (art) => (istDurchgereicht(art) ? vcardName(art) : t('contacts.kinds.' + art));

/*
 * Eine neue Zeile faengt mit `phone` an und nicht mit dem ERSTEN Eintrag des
 * Auswahlfelds: Dort steht seit dem 07.09.2026 die Matrix-Kennung vorn, weil
 * sie schwerer zu finden waere als eine Telefonnummer. Was jemand hier unten
 * am ehesten eintraegt, ist trotzdem die zweite Nummer.
 */
const wegHinzufuegen = () => {
  formular.value.channels.push({ kind: 'phone', label: '', value: '' });
};

const wegEntfernen = (i) => {
  formular.value.channels.splice(i, 1);
};

/*
 * Die Liste zeigt einen Namen und eine Nummer – mehr nicht.
 *
 * WARUM NICHT ALLES: Wer ein Adressbuch aufschlägt, sucht einen NAMEN. Standen
 * hinter jedem Namen alle Wege nebeneinander, wäre die Liste bei drei
 * Kontakten schön und bei dreißig unlesbar – und der Name, das einzige, wonach
 * man sucht, ginge in seiner eigenen Zeile unter.
 *
 * Aufgeklappt steht dann jeder Weg in einer EIGENEN Zeile und nicht
 * hintereinander: Nebeneinander ist eine Nummer von einer Kennung nur am
 * Sinnbild zu unterscheiden, und beim Umbrechen zerreißt es die Paare.
 */
const aufgeklappt = ref([]);

const istOffen = (id) => aufgeklappt.value.includes(id);

const umklappen = (id) => {
  const i = aufgeklappt.value.indexOf(id);

  // Mehrere zugleich offen: Wer zwei Nummern vergleichen will, soll nicht
  // hin- und herklicken müssen.
  if (i === -1) aufgeklappt.value.push(id);
  else aufgeklappt.value.splice(i, 1);
};

/*
 * Die erste Telefonnummer – oder nichts.
 *
 * KEIN RÜCKFALL auf den ersten Weg irgendeiner Art: Stünde da mal eine
 * Nummer und mal eine Matrix-Kennung, hieße dieselbe Stelle zweierlei, und
 * die Liste wäre in genau dem Moment nicht mehr überfliegbar, in dem sie es
 * sein soll. Ein Name allein ist eine ehrliche Zeile.
 */
const ersteNummer = (kontakt) => (kontakt.channels ?? []).find((w) => w.kind === 'phone')?.value ?? '';

// Je Art den ersten Weg nach oben, alles andere bleibt unten. `splice` und
// nicht `filter`, damit die Reihenfolge der übrigen erhalten bleibt.
const aufteilen = (wege) => {
  const rest = wege.map((w) => ({ kind: w.kind, label: w.label ?? '', value: w.value }));
  const offen = {};

  OFFENE_FELDER.forEach((art) => {
    const i = rest.findIndex((w) => w.kind === art);
    offen[art] = i === -1 ? '' : rest.splice(i, 1)[0].value;
  });

  return { offen, channels: rest };
};

const zusammenfuehren = () => [
  ...OFFENE_FELDER.map((art) => ({ kind: art, label: null, value: formular.value.offen[art] })),
  ...formular.value.channels,
];

// Der Weg zurück aus der Sackgasse — siehe die Begründung an der Kopfleiste.
const zuNachrichten = () => emit('nachrichten');

/*
 * Vom Namen zur Route.
 *
 * WELCHER Weg zu einem Konto führt und wie es heisst, beantwortet
 * `nachrichtenWeg` in `shared/kontaktwege` — dieselbe Frage stellt sich
 * `openany-app`, nur mit anderer Antwort. WOHIN das in dieser Webapp führt,
 * steht hier: eine Route kennt drüben niemand.
 *
 * `?an=` ist derselbe Griff wie `?neu=` in der Gegenrichtung — Nachrichten
 * öffnet damit hier das Kontaktformular, das Adressbuch dort das
 * Schreibfeld. Eine Absicht, ein Fragezeichen.
 */
const nachrichtenZiel = (weg) => {
  // Ohne Nachrichten führt nichts dorthin.
  if (!props.nachrichten) return null;
  const ziel = nachrichtenWeg(weg);
  if (!ziel) return null;

  // EINE MATRIX-KENNUNG FÜHRT NUR MIT VERBUNDENEM KONTO IRGENDWOHIN. Ohne
  // eines stünde hier ein Link, der den Verfassen-Dialog öffnet, in dem es
  // den Matrix-Weg gar nicht gibt — ein toter Link, der klickbar aussieht.
  // Genau die Sorte, vor der der Kommentar in `wegZiel` warnt.
  if (ziel.kanal === 'matrix' && !props.matrixVerbunden) return null;

  return ziel;
};

/*
 * VON DER ABSICHT ZUM WEG DORTHIN — und das ist zweierlei.
 *
 * `nachrichtenZiel` beantwortet nur, OB und an WEN geschrieben werden kann.
 * WIE man dorthin kommt, weiss der Rahmen: Die Webapp hat einen Router und
 * bekommt eine Route, das Programm hat keinen und bekommt ein Ereignis.
 * Dieselbe Aufteilung wie in `LeisteUnten`, `KopfZeile` und `MenuEintrag` —
 * „ein `Link`, wenn einer da ist, sonst ein Knopf".
 *
 * Bis zum 24.09.2026 verlangte `nachrichtenZiel` selbst eine
 * Link-Komponente. Damit war im Programm JEDE Kennung schlichter Text, ohne
 * dass etwas fehlschlug: Es gab dort einfach nie einen Weg von einem Kontakt
 * zu einer Nachricht.
 */
const alsRoute = (ziel) => ({
  path: '/messages',
  query: ziel.kanal === 'matrix'
    ? { an: ziel.kennung, kanal: 'matrix' }
    : { an: ziel.kennung },
});

// Wie eine Zeile in der Liste aussieht: Sinnbild, Beschriftung, Wert – und ob
// ein Klick etwas tun kann.
const alsZeile = (weg) => ({
  ...weg,
  icon: SINNBILD[weg.kind] ?? Hash,
  name: artName(weg.kind),
  // Ein Sinnbild gibt es nur für die eigenen Arten; ein durchgereichtes Feld
  // bekäme sonst ein Rauten-Zeichen und sonst nichts.
  zeigtArt: istDurchgereicht(weg.kind),
  ziel: wegZiel(weg),
  nachricht: nachrichtenZiel(weg),
});

const laden = async () => {
  laedt.value = true;
  try {
    const { data } = await api.getContacts();
    kontakte.value = data;
  } catch {
    toast.error(t('contacts.loadFailed'));
  } finally {
    laedt.value = false;
  }
};

const oeffnen = (kontakt = null) => {
  bearbeitet.value = kontakt;
  bildDatei.value = null;
  bildWeg.value = false;
  bildVorschau.value = kontakt?.photo ? fotoUrl(kontakt) : null;

  formular.value = {
    display_name: kontakt?.display_name ?? '',
    // `aufteilen` legt flache Kopien an – sonst schriebe das Formular in die
    // geladene Liste, und „Abbrechen" ließe die Änderung sichtbar stehen.
    ...aufteilen(kontakt?.channels ?? []),
  };
  dialogOffen.value = true;
};

const speichern = async () => {
  const name = formular.value.display_name.trim();
  if (!name) {
    toast.error(t('contacts.nameRequired'));
    return;
  }

  // Leere Felder gehen als null hinaus und nicht als '' – sonst stünde in der
  // Spalte eine leere Zeichenkette, und „keine Kennung" wäre zwei
  // verschiedene Zustände.
  const nutzlast = {
    display_name: name,
    // Leere Zeilen gehen mit hinaus und werden serverseitig ausgesiebt
    // (Contact::wegeSetzen). Sie hier zu filtern wäre eine zweite Regel für
    // dieselbe Sache – und die beiden liefen irgendwann auseinander.
    channels: zusammenfuehren(),
  };

  speichert.value = true;
  try {
    const { data } = bearbeitet.value
      ? await api.updateContact(bearbeitet.value.id, nutzlast)
      : await api.createContact(nutzlast);

    // Das Bild NACH dem Kontakt: Ein neuer hat vorher keine Id. Scheitert
    // nur das Bild, bleibt der Kontakt trotzdem gespeichert — die Meldung
    // sagt dann genau das, statt „Speichern fehlgeschlagen" über beides.
    try {
      if (bildDatei.value) await api.uploadContactPhoto(data.id, bildDatei.value);
      else if (bildWeg.value) await api.deleteContactPhoto(data.id);
    } catch {
      toast.error(t('contacts.photoFailed'));
    }

    toast.success(bearbeitet.value ? t('contacts.saved') : t('contacts.created'));
    dialogOffen.value = false;
    await laden();
  } catch {
    toast.error(t('contacts.saveFailed'));
  } finally {
    speichert.value = false;
  }
};

const loeschen = async (kontakt) => {
  if (!await confirmDelete(t('contacts.deleteConfirm', { name: kontakt.display_name }))) return;

  try {
    await api.deleteContact(kontakt.id);
    toast.success(t('contacts.deleted'));
    await laden();
  } catch {
    toast.error(t('contacts.deleteFailed'));
  }
};

// „+ Kontakt" IN DEN NACHRICHTEN SOLL HIER GLEICH DAS FORMULAR AUFSCHLAGEN –
// und die Startseiten-Kachel bringt einen Suchbegriff mit. Woher das kommt
// (in der Webapp aus `?neu` bzw. `?suche=`), weiß der Rahmen; hier steht nur,
// was damit geschieht. `start-verbraucht` sagt dem Rahmen, dass er die Frage
// wegräumen kann – sonst öffnete der Zurück-Knopf das Formular erneut.
const suchfeld = ref(null);

onMounted(async () => {
  await laden();

  if (props.startNeu) {
    emit('start-verbraucht');
    oeffnen();
  } else if (props.startSuche !== null) {
    suche.value = props.startSuche;
    emit('start-verbraucht');
    // Das Feld steht erst nach dem Laden im DOM (v-else auf `laedt`).
    await nextTick();
    suchfeld.value?.focus();
  }
});
</script>

<template>
  <ModulePage>
    <ModuleHeader :icon="ContactIcon">
      <!-- DER WEG ZURÜCK, ergänzt am 10.09.2026.
           Das Adressbuch ist seit dem Vortag nur noch über die Nachrichten
           erreichbar — aus dem Menü ist es verschwunden. Damit war es eine
           Sackgasse: Wer hier ankam, hatte keinen Weg zurück ausser dem
           Zurück-Knopf des Browsers, und auf dem Telefon in der App gibt es
           den nicht.
           Beschriftung erst ab `sm`, wie drüben in Messages.vue: Auf dem
           Telefon trägt das Sinnbild allein, und die Haupthandlung der Seite
           („+ Kontakt") behält ihr Wort. -->
      <div class="flex items-center gap-2 sm:gap-3">
        <BaseButton
          v-if="nachrichten"
          @click="zuNachrichten"
          variant="secondary" groesse="kopf" class="shrink-0"
          :title="t('shell.header.messages')"
        >
          <Mail class="w-4 h-4" />
          <span class="hidden sm:inline">{{ t('shell.header.messages') }}</span>
        </BaseButton>

        <BaseButton @click="oeffnen()" groesse="kopf" class="shrink-0">
          <Plus class="w-4 h-4" />
          <span>{{ t('contacts.add') }}</span>
        </BaseButton>
      </div>
    </ModuleHeader>

    <div v-if="laedt" class="flex justify-center py-12">
      <Loader2 class="w-6 h-6 animate-spin text-slate-400" />
    </div>

    <div v-else-if="kontakte.length === 0"
      class="karte p-8 text-center">
      <p class="font-bold text-schrift">{{ t('contacts.empty') }}</p>
      <p class="mt-1 text-sm text-leise">{{ t('contacts.emptyHint') }}</p>
    </div>

    <template v-else>
      <!-- Das Suchfeld steht ueber der Liste und nur, wenn es etwas zu
           durchsuchen gibt: Ein Feld ueber einer leeren Liste verspricht
           etwas, das es nicht halten kann. -->
      <div class="relative mb-3">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
        <input ref="suchfeld" v-model="suche" type="search" :placeholder="t('contacts.searchPlaceholder')"
          :aria-label="t('contacts.search')"
          autocapitalize="none" autocorrect="off" spellcheck="false"
          class="w-full pl-9 pr-9 py-2 rounded-xl border border-linie bg-flaeche text-schrift" />
        <button v-if="suche" type="button" @click="suche = ''" :aria-label="t('common.cancel')"
          class="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-slate-400 hover:text-slate-900 dark:hover:text-white rounded-lg cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div v-if="gefiltert.length === 0"
        class="karte p-8 text-center">
        <p class="font-bold text-schrift">{{ t('contacts.noMatch', { suche }) }}</p>
      </div>

      <ul v-else class="karte divide-y divide-linie">
      <li v-for="kontakt in gefiltert" :key="kontakt.id">
        <div class="flex items-center justify-between gap-3 p-3 sm:p-4">
          <!-- Der Name IST der Schalter. Ein eigener Pfeil daneben wäre ein
               zweites, kleineres Ziel für dieselbe Handlung – auf dem Telefon
               das schlechtere von beiden. -->
          <button @click="umklappen(kontakt.id)" :aria-expanded="istOffen(kontakt.id)"
            class="flex items-center gap-2 min-w-0 flex-1 text-left cursor-pointer">
            <ChevronRight class="w-4 h-4 shrink-0 text-slate-400 transition-transform"
              :class="istOffen(kontakt.id) ? 'rotate-90' : ''" />
            <!-- Ein Gesicht in der Liste, sonst die Anfangsbuchstaben. Ein
                 leerer grauer Kreis sähe aus, als lade noch etwas. -->
            <img v-if="kontakt.photo" :src="fotoUrl(kontakt)" alt=""
              class="w-9 h-9 shrink-0 rounded-full object-cover bg-auflage" />
            <span v-else
              class="w-9 h-9 shrink-0 rounded-full bg-marke-leise text-marke flex items-center justify-center text-xs font-extrabold">
              {{ initialen(kontakt.display_name) }}
            </span>
            <span class="min-w-0">
              <span class="block font-bold text-schrift truncate">{{ kontakt.display_name }}</span>
              <span v-if="ersteNummer(kontakt)" class="block text-sm text-leise truncate">
                {{ ersteNummer(kontakt) }}
              </span>
              <span v-else-if="!kontakt.channels?.length" class="block text-sm italic text-leise truncate">
                {{ t('contacts.noIdentifiers') }}
              </span>
            </span>
          </button>
          <div class="flex items-center gap-1 shrink-0">
            <button @click="oeffnen(kontakt)" :aria-label="t('contacts.editTitle')"
              class="p-2 text-slate-500 hover:text-slate-900 dark:hover:text-white hover:bg-auflage rounded-xl cursor-pointer">
              <Pencil class="w-4 h-4" />
            </button>
            <button @click="loeschen(kontakt)" :aria-label="t('common.delete')"
              class="p-2 text-slate-500 hover:text-rose-600 hover:bg-auflage rounded-xl cursor-pointer">
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Aufgeklappt: jeder Weg in einer eigenen Zeile. Klickbar, wo das
             Betriebssystem etwas damit anfangen kann (tel:, mailto:) – sonst
             schlichter Text, denn ein Link, der nichts tut, ist schlimmer als
             keiner. -->
        <div v-if="istOffen(kontakt.id)" class="px-3 sm:px-4 pb-3 sm:pb-4 pl-9 sm:pl-10 space-y-1.5">
          <p v-if="!kontakt.channels?.length" class="text-sm italic text-leise">
            {{ t('contacts.noIdentifiers') }}
          </p>

          <div v-for="(weg, i) in kontakt.channels.map(alsZeile)" :key="i"
            class="flex items-center gap-2 text-sm min-w-0">
            <component :is="weg.icon" class="w-4 h-4 shrink-0 text-slate-400" />
            <!-- Drei Fälle, und die Reihenfolge ist die der Genauigkeit:
                 ein Weg nach innen (openany-Name → Nachrichten), ein Weg
                 nach außen (tel:, mailto:), oder gar keiner. -->
            <component :is="Link" v-if="weg.nachricht && Link" :to="alsRoute(weg.nachricht)"
              :title="t('contacts.writeMessage')"
              class="text-marke hover:underline truncate">{{ weg.value }}</component>
            <!-- Ohne Router: derselbe Weg, nur als Absicht an den Rahmen. -->
            <button v-else-if="weg.nachricht" type="button"
              @click="emit('schreiben', weg.nachricht)"
              :title="t('contacts.writeMessage')"
              class="text-marke hover:underline truncate cursor-pointer text-left">{{ weg.value }}</button>
            <a v-else-if="weg.ziel" :href="weg.ziel"
              class="text-marke hover:underline truncate">{{ weg.value }}</a>
            <span v-else class="text-fliess truncate">{{ weg.value }}</span>
            <!-- Die Art dazu, wo das Sinnbild sie nicht erklärt: Telefon und
                 E-Mail erkennt man am Bild, „ANNIVERSARY" nicht. -->
            <span v-if="weg.zeigtArt" class="text-xs text-slate-400 dark:text-slate-500 shrink-0">{{ weg.name }}</span>
            <span v-if="weg.label" class="text-xs text-slate-400 dark:text-slate-500 shrink-0">{{ weg.label }}</span>
          </div>

          <p v-if="kontakt.linked_user" class="text-xs text-slate-400 dark:text-slate-500 truncate">
            {{ t('contacts.linkedUser') }}: {{ kontakt.linked_user.name }}
          </p>
        </div>
      </li>
      </ul>
    </template>

    <BaseModal v-if="dialogOffen"
      :title="bearbeitet ? t('contacts.editTitle') : t('contacts.newTitle')"
      @close="dialogOffen = false">
      <form class="space-y-4" @submit.prevent="speichern">
        <!-- Das Bild steht neben dem Namen und nicht in einem eigenen
             Abschnitt: Beides zusammen ist, woran man einen Menschen
             erkennt. -->
        <div class="flex items-start gap-3">
          <div class="shrink-0">
            <button type="button" @click="bildfeld.click()" :aria-label="t('contacts.photoChoose')"
              class="relative w-16 h-16 rounded-full overflow-hidden bg-marke-leise flex items-center justify-center cursor-pointer group">
              <img v-if="bildVorschau" :src="bildVorschau" alt="" class="w-full h-full object-cover" />
              <span v-else class="text-marke text-lg font-extrabold">
                {{ initialen(formular.display_name) || '?' }}
              </span>
              <span class="absolute inset-0 bg-slate-900/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                <Camera class="w-5 h-5 text-white" />
              </span>
            </button>
            <button v-if="bildVorschau" type="button" @click="bildEntfernen"
              class="mt-1 w-full text-[11px] text-slate-500 hover:text-rose-600 cursor-pointer">
              {{ t('contacts.photoRemove') }}
            </button>
            <input ref="bildfeld" type="file" accept="image/*" class="hidden" @change="bildGewaehlt" />
          </div>

          <div class="flex-1 min-w-0">
          <label class="block text-sm font-bold text-fliess mb-1">
            {{ t('contacts.displayName') }}
          </label>
          <input v-model="formular.display_name" type="text" required
            :placeholder="t('contacts.displayNamePlaceholder')"
            class="w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift" />
          <p class="mt-1 text-[11px] text-slate-400 dark:text-slate-500">{{ t('contacts.photoHint') }}</p>
          </div>
        </div>
        <!-- Die beiden Angaben, die fast jeder hat, stehen offen da. Bis zum
             07.09.2026 war es andersherum: Matrix und Meshtastic hatten
             eigene Felder, Nummer und E-Mail keines. Beide Arten stehen
             AUCH im Auswahlfeld unten – das hier ist eine Abkürzung, kein
             zweiter Mechanismus. -->
        <div v-for="art in OFFENE_FELDER" :key="art">
          <label class="block text-sm font-bold text-fliess mb-1">
            {{ t('contacts.kinds.' + art) }}
          </label>
          <input v-model="formular.offen[art]" type="text"
            :placeholder="t('contacts.kindPlaceholder.' + art)"
            autocapitalize="none" autocorrect="off" spellcheck="false"
            :inputmode="art === 'phone' ? 'tel' : 'email'"
            class="w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift" />
        </div>
        <!-- Nummern und Adressen: beliebig viele, in der Reihenfolge des
             Menschen. Keine feste Zahl von Feldern, weil jede feste Zahl bei
             jemandem nicht passt. -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="block text-sm font-bold text-fliess">
              {{ t('contacts.moreChannels') }}
            </label>
            <button type="button" @click="wegHinzufuegen"
              class="flex items-center gap-1 px-2 py-1 text-xs font-bold text-marke hover:bg-auflage rounded-lg cursor-pointer">
              <Plus class="w-3.5 h-3.5" />
              {{ t('contacts.channelAdd') }}
            </button>
          </div>

          <p v-if="!formular.channels.length" class="text-xs text-leise">
            {{ t('contacts.channelsEmpty') }}
          </p>

          <div v-for="(weg, i) in formular.channels" :key="i" class="flex items-start gap-2 mb-2">
            <select v-model="weg.kind" :aria-label="t('contacts.channelKind')"
              class="shrink-0 max-w-[9rem] px-2 py-2 rounded-xl border border-linie bg-vertieft text-sm text-schrift">
              <option v-for="art in WEG_ARTEN" :key="art" :value="art">{{ t('contacts.kinds.' + art) }}</option>
              <!-- Alles, was vCard sonst noch kennt. Eine Auswahl, keine
                   Grenze: Ein Feld, das ein fremdes Adressbuch mitbringt,
                   kommt auch dann an, wenn es hier nicht steht — es taucht
                   dann als eigener Eintrag unter seinem vCard-Namen auf. -->
              <optgroup :label="t('contacts.advanced')">
                <option v-for="art in VCARD_FELDER" :key="art" :value="art">{{ vcardName(art) }}</option>
                <!-- Ein durchgereichtes Feld, das nicht in der Auswahl steht,
                     braucht trotzdem einen Eintrag – sonst stünde im
                     Auswahlfeld etwas anderes, als in der Zeile steht, und
                     Speichern änderte die Art. -->
                <option v-if="istDurchgereicht(weg.kind) && !VCARD_FELDER.includes(weg.kind)"
                  :value="weg.kind">{{ vcardName(weg.kind) }}</option>
              </optgroup>
            </select>
            <div class="flex-1 min-w-0 space-y-2">
              <input v-model="weg.value" type="text" :placeholder="t('contacts.channelValue')"
                autocapitalize="none" autocorrect="off" spellcheck="false"
                class="w-full px-3 py-2 rounded-xl border border-linie bg-vertieft text-schrift" />
              <input v-model="weg.label" type="text" :placeholder="t('contacts.channelLabel')"
                class="w-full px-3 py-1.5 rounded-xl border border-linie bg-vertieft text-sm text-leise" />
            </div>
            <button type="button" @click="wegEntfernen(i)" :aria-label="t('contacts.channelRemove')"
              class="p-2 text-slate-400 hover:text-rose-600 hover:bg-auflage rounded-xl cursor-pointer">
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        <p v-if="bearbeitet?.linked_user" class="text-xs text-leise">
          {{ t('contacts.linkedUser') }}: {{ bearbeitet.linked_user.name }} —
          {{ t('contacts.linkedUserHint') }}
        </p>
      </form>
      <template #footer>
        <BaseButton variant="secondary" @click="dialogOffen = false">{{ t('common.cancel') }}</BaseButton>
        <BaseButton :loading="speichert" @click="speichern">{{ t('contacts.save') }}</BaseButton>
      </template>
    </BaseModal>
  </ModulePage>
</template>
