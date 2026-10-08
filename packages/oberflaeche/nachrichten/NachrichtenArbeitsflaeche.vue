<script setup>
/*
 * Die Nachrichten-Arbeitsfläche: Verlauf, Schreibfeld, Adressbuch-Türen.
 *
 * WARUM ES DIESE DATEI GIBT. Bis zum 24.09.2026 hatten Webapp und Programm
 * jeweils ihre eigene Nachrichten-Seite — zwei Abschriften desselben
 * Schreibfelds, derselben Verlaufszeile, desselben Kopfes. Sie liefen schon
 * auseinander: Der Kanal-Umschalter hiess drüben „Weg" und hier auch, aber
 * der eine kannte `aria-checked` und der andere nicht; der eine klappte lange
 * Nachrichten ein, der andere nicht. Genau die Art von Abstand, die niemand
 * meldet, weil nichts fehlschlägt — und die am 15.09.2026 schon einmal die
 * Schale des Programms veralten liess.
 *
 * Also dasselbe Muster wie Notizen, Kalender und Adressbuch: EINE Fläche im
 * Paket, eine Datenquelle je Rahmen.
 *
 * ═══ WAS HIER STEHT UND WAS NICHT ═══
 *
 * Hier: alles, was in beiden Rahmen gleich ist — der Modul-Kopf mit seinen
 * drei Knöpfen, der Verlauf, das Schreibfeld mit der Wahl des Weges.
 *
 * Nicht hier: WOHIN die Adressbuch-Knöpfe führen (die Webapp hat einen
 * Router, das Programm nicht — deshalb `adressbuch`/`neuer-kontakt` als
 * Ereignis), und was nur ein Rahmen kennt. Die Matrix-Anmeldung des
 * Programms und seine Störungsmeldungen stehen im `<slot name="oben">`:
 * Drüben meldet sich openany.de am Konto an, hier das Gerät selbst — das ist
 * kein gemeinsamer Zustand, sondern der Unterschied in der Zusage.
 */
import { ref, computed, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mail, Plus, Send, Trash2, X, MessageSquare, Search, Paperclip, MailOpen, Users, ListOrdered, Loader2, Contact as ContactIcon, UserPlus, Download, Lock, ShieldCheck, ShieldAlert, ShieldQuestion, Ban, Radar, UserCheck } from 'lucide-vue-next';
import ModulePage from '@oberflaeche/base/ModulePage.vue';
import ModuleHeader from '@oberflaeche/base/ModuleHeader.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import { useToast } from '@oberflaeche/composables/useToast';
import { useConfirm } from '@oberflaeche/composables/useConfirm';
import { formatDateTime } from '@oberflaeche/shared/date';
import AnhangWahl from './AnhangWahl.vue';
import { trefferTeile as trefferTeilen } from './treffer';
import AnhangKachel from './AnhangKachel.vue';
import PdfBetrachter from '../speicher/PdfBetrachter.vue';
import BaseModal from '../base/BaseModal.vue';

const props = defineProps({
  /*
   * `liste(seite)` → `{ items, next_page, serverfehler? }`, dazu `senden`,
   * `gelesen` und `loeschen`. Eine Zeile in `items` trägt: `id`, `body`,
   * `transport`, `created_at`, `read_at`, `von_mir` und `peer` (die
   * Gegenseite, schon als Name aufgelöst), dazu `anhang` ({ name, mime,
   * groesse }) oder nichts.
   *
   * Anhänge (seit 28.09.2026), alles optional: `senden(ziel, text, weg,
   * datei)` nimmt eine Datei mit, `anhangGrenze()` sagt die Grenze in Bytes
   * (ohne sie keine Büroklammer), `anhang(zeile)` liefert die Bytes,
   * `anhangSpeichern(zeile)` legt sie ab (sonst lädt der Browser herunter).
   *
   * `von_mir` UND NICHT `meId`: Das Programm hält einen Geräteschlüssel und
   * hat nie eine `/api/user`-Antwort gesehen — es kennt seine eigene Konto-Id
   * gar nicht. Der Server beantwortet die Frage deshalb selbst
   * (MessageResource).
   */
  dataSource: { type: Object, required: true },
  // Ohne verbundenes Matrix-Konto gibt es nichts zu wählen, und eine Wahl
  // mit einer Möglichkeit ist keine.
  matrixVerbunden: { type: Boolean, default: false },
  // Ein Postfach verbunden? Dann gibt es den Weg „E-Mail" (nur im Programm,
  // docs/plan-email-pgp.md, Schritt 2).
  mailVerbunden: { type: Boolean, default: false },
  // Die verbundenen Postfächer (Adressen), das Standard-Postfach zuerst.
  // Ab zweien gibt es beim Schreiben ein Feld „Von", und jede Mail sagt,
  // über welches sie kam.
  mailPostfaecher: { type: Array, default: () => [] },
  // Dasselbe für Matrix: die verbundenen Konten (Kennungen), das
  // Standard-Konto zuerst (nur im Programm).
  matrixKonten: { type: Array, default: () => [] },
  // Ob überhaupt geschrieben werden kann (das Programm wartet erst seinen
  // Zustand ab).
  bereit: { type: Boolean, default: true },
  /*
   * Zerlegt den Nachrichtentext in Stücke, damit Web-Adressen anklickbar
   * werden. ALS EIGENSCHAFT UND NICHT IM PAKET: Der Zerleger der Webapp
   * kennt auch `[[Wikilinks]]` und hängt dafür an `planningTypes` — einer
   * Datei, die das Programm nichts angeht. Vorgabe ist ein Stück Text.
   *
   * Das Ergebnis wird über `v-for` ausgegeben, nie über `v-html`; die
   * Maskierung bleibt Vues Aufgabe.
   */
  zerleger: { type: Function, default: (body) => [{ type: 'text', value: String(body ?? '') }] },
  /*
   * Was unter der Wahl des Weges steht, als `{ matrix, openany }`.
   *
   * ALS EIGENSCHAFT, WEIL DIE ZUSAGE ENTGEGENGESETZT IST: In der Webapp
   * meldet sich openany.de am Matrix-Konto an und kann mitlesen; im Programm
   * ist das Gerät selbst das Matrix-Gerät und der Server sieht keine Zeile.
   * Ein gemeinsamer Satz wäre in genau einem der beiden Rahmen gelogen.
   */
  wegHinweise: { type: Object, default: null },
  /*
   * „Vor Ort" (nur im Programm, 02.10.2026): Direktnachrichten an Geräte in
   * der Nähe, ohne Server. Die Quelle nennt dann `nahZiele()` (wen man
   * anschreiben kann), `blockieren(person)` und `kontaktBestaetigen(ziel)`
   * (6 Ziffern vor Ort, seit 06.10.2026). In der Webapp aus.
   */
  nahVerfuegbar: { type: Boolean, default: false },
  /*
   * Den internen Weg gibt es nur mit einem openany-Server. In der Webapp
   * immer; im Programm nur, wenn es gekoppelt ist (local-first: nur zeigen,
   * was hier geht -- 02.10.2026).
   */
  openanyVerfuegbar: { type: Boolean, default: true },
});

const emit = defineEmits(['adressbuch', 'neuer-kontakt', 'geaendert']);

const { t } = useI18n();
const toast = useToast();
const { confirmDelete, confirmDialog } = useConfirm();

const quelle = props.dataSource;

const liste = ref([]);
const naechsteSeite = ref(null);
const serverfehler = ref(null);

/*
 * FILTER, SUCHE, NACH KONTAKT (Tiffy, 01.10.2026).
 *
 * GEFILTERT WIRD IN DER QUELLE, nicht hier: Die Liste kommt seitenweise, und
 * „Ungelesen" über die erste Seite allein wäre eine halbe Antwort. Die Quelle
 * bekommt `{ wege, ungelesen, anhang, q, mit }`; `mit` ist eine Unterhaltung
 * als Liste `weg:wert` -- so, wie `unterhaltungen()` sie nennt.
 *
 * Eine Quelle ohne `unterhaltungen` bekommt keinen Umschalter.
 *
 * Jede Änderung ersetzt `filter` als Ganzes; ein Beobachter lädt dann neu.
 */
const filter = ref({ wege: [], ungelesen: false, anhang: false, q: '', mit: [] });
const mitName = ref('');
const suchtext = ref('');
const ansicht = ref('verlauf'); // 'verlauf' | 'kontakte'
const unterhaltungen = ref([]);
const kannNachKontakt = computed(() => typeof quelle.unterhaltungen === 'function');
const gefiltert = computed(() => {
  const f = filter.value;
  return Boolean(f.wege.length || f.ungelesen || f.anhang || f.q.trim() || f.mit.length);
});
let suchUhr = null;
watch(suchtext, (q) => {
  clearTimeout(suchUhr);
  suchUhr = setTimeout(() => { filter.value = { ...filter.value, q }; }, 300);
});
const umschalten = (feld) => { filter.value = { ...filter.value, [feld]: !filter.value[feld] }; };
const wegUmschalten = (weg) => {
  const gewaehlt = new Set(filter.value.wege);
  if (gewaehlt.has(weg)) gewaehlt.delete(weg);
  else gewaehlt.add(weg);
  filter.value = { ...filter.value, wege: [...gewaehlt] };
};
// Ein Tipp auf eine Unterhaltung: ihr Verlauf, über alle ihre Wege.
const unterhaltungOeffnen = (u) => {
  mitName.value = u.name;
  ansicht.value = 'verlauf';
  filter.value = { ...filter.value, mit: [...u.mit] };
};
const unterhaltungSchliessen = () => {
  mitName.value = '';
  filter.value = { ...filter.value, mit: [] };
};
const fuerQuelle = () => ({ ...filter.value, wege: [...filter.value.wege], mit: [...filter.value.mit] });

// Der Treffer im Text, hervorgehoben (treffer.js).
const trefferTeile = (text) => trefferTeilen(text, filter.value.q);

const schreibenOffen = ref(false);
const sendet = ref(false);

/*
 * ZWEI EMPFÄNGERFELDER, NICHT EINES, und beide behalten ihren Inhalt beim
 * Umschalten: Ein openany-Konto darf „@tiffy:matrix.org" heissen — aus dem
 * Inhalt wäre also nicht zu erraten, was gemeint ist. Der Server
 * unterscheidet aus demselben Grund zwei Felder (MessageController).
 */
const entwurf = ref({ weg: 'openany', name: '', mxid: '', email: '', von: '', betreff: '', antwortAuf: null, text: '', anhang: null, verschluesseln: false });
/*
 * VON WELCHEM EIGENEN KONTO. Erst ab zweien eine Wahl: Postfächer bei E-Mail,
 * Konten bei Matrix. Eine Antwort geht von dem, bei dem die Nachricht ankam.
 */
const kontenVon = (weg) => ({ email: props.mailPostfaecher, matrix: props.matrixKonten }[weg] ?? []);
const vonListe = computed(() => kontenVon(entwurf.value.weg));
const mehrereKonten = (weg) => kontenVon(weg).length > 1;
watch(() => entwurf.value.weg, (weg) => {
  if (!entwurf.value.antwortAuf) entwurf.value.von = kontenVon(weg)[0] ?? '';
});

/*
 * OPENPGP (docs/plan-email-pgp.md, Schritt 3b) -- nur, wo die Quelle es kann
 * (das Programm). VERSCHLÜSSELT WIRD NUR AUF WUNSCH (Tiffy, 29.09.2026): Der
 * Schalter steht aus. Ob es gehen würde, sagt das Feld trotzdem schon vorher,
 * sobald eine Adresse dasteht -- die Suche geht nur an die eigene Ablage und
 * an den Mailanbieter des Empfängers (WKD).
 */
const pgpMoeglich = computed(() => entwurf.value.weg === 'email' && typeof quelle.pgpStatus === 'function');
const pgp = ref(null); // { eigener, empfaenger } | 'sucht' | null
let pgpUhr = null;
watch(() => [pgpMoeglich.value, entwurf.value.email, entwurf.value.von], ([moeglich, an, von]) => {
  clearTimeout(pgpUhr);
  pgp.value = null;
  const adresse = String(an ?? '').trim();
  if (!moeglich || !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(adresse)) return;
  pgpUhr = setTimeout(async () => {
    pgp.value = 'sucht';
    try {
      const status = await quelle.pgpStatus(von || null, adresse);
      if (entwurf.value.email.trim() === adresse) pgp.value = status;
    } catch {
      pgp.value = null;
    }
  }, 600);
});
const pgpBereit = computed(() => pgp.value && pgp.value !== 'sucht' && pgp.value.eigener && pgp.value.empfaenger);
const fingerabdruckKurz = (f) => (String(f ?? '').slice(-16).match(/.{1,4}/g) ?? []).join(' ');

// Die Wege, die es gerade gibt -- der interne, wo ein Server da ist.
const wege = computed(() => [
  ...(props.openanyVerfuegbar ? ['openany'] : []),
  ...(props.matrixVerbunden ? ['matrix'] : []),
  ...(props.mailVerbunden ? ['email'] : []),
  ...(props.nahVerfuegbar ? ['nah'] : []),
]);

/*
 * VOR ORT: Wen man anschreiben kann, kommt aus der Quelle -- Geräte in der
 * Nähe und Personen, mit denen schon geschrieben wurde. Geladen, sobald der
 * Weg gewählt ist; „unbekannt" heißt: kein gemeinsames Projekt.
 */
const nahZiele = ref([]);
const nahLaedt = ref(false);

/*
 * NUR AN BESTÄTIGTE (06.10.2026). Wer „unbekannt" ist, lässt sich vor Ort
 * nicht anschreiben: Seine Antwort käme hier als Anfrage an oder gar nicht.
 * Statt „Senden" steht dann der Weg dorthin -- per 6 Ziffern bestätigen,
 * solange ein Gerät der Person in der Nähe ist (`quelle.kontaktBestaetigen`).
 */
const nahUnbestaetigt = computed(() => entwurf.value.weg === 'nah'
  && Boolean(nahZiele.value.find((z) => z.ziel === entwurf.value.name)?.unbekannt));
const nahBestaetigt = ref(false);
async function nahBestaetigen() {
  if (typeof quelle.kontaktBestaetigen !== 'function') return;
  nahBestaetigt.value = true;
  try {
    await quelle.kontaktBestaetigen(entwurf.value.name);
    schreibenOffen.value = false;
  } catch (e) {
    toast.error(String(e?.message ?? e));
  } finally {
    nahBestaetigt.value = false;
  }
}

/** Warum eine Nachricht vor Ort nicht ankam -- die Kennung in Worten. */
function nahGrund(kennung) {
  const schluessel = `settings.messages.nahAbgelehnt.${kennung}`;
  const text = t(schluessel);
  return text === schluessel ? t('settings.messages.nahAbgelehnt.nicht-angenommen') : text;
}
async function nahZieleLaden() {
  if (typeof quelle.nahZiele !== 'function') return;
  nahLaedt.value = true;
  try { nahZiele.value = await quelle.nahZiele(); } catch { nahZiele.value = []; } finally { nahLaedt.value = false; }
}

/*
 * ANHÄNGE (docs/plan-email-pgp.md, Schritt 1) -- nur auf Wegen, die sie
 * kennen: heute Matrix. Die Quelle sagt es mit `anhangGrenze()` (Bytes);
 * fehlt die Methode, gibt es keine Büroklammer. Der interne Weg bekommt
 * keine, bis er Anhänge wirklich tragen kann.
 */
const anhaengbar = computed(() => ['matrix', 'email'].includes(entwurf.value.weg) && typeof quelle.anhangGrenze === 'function');
const anhangGrenze = ref(0);
// Je Weg eine eigene Grenze (Matrix 10 MB, E-Mail 15 MB).
watch([anhaengbar, () => entwurf.value.weg], async ([ja, weg]) => {
  anhangGrenze.value = 0;
  if (!ja) return;
  try { anhangGrenze.value = Number(await quelle.anhangGrenze(weg)) || 0; } catch { /* dann prüft der Weg */ }
}, { immediate: true });

/** Was im sichtbaren Empfängerfeld steht. */
const empfaenger = computed({
  get: () => ({ matrix: entwurf.value.mxid, email: entwurf.value.email }[entwurf.value.weg] ?? entwurf.value.name),
  set: (wert) => {
    if (entwurf.value.weg === 'matrix') entwurf.value.mxid = wert;
    else if (entwurf.value.weg === 'email') entwurf.value.email = wert;
    else entwurf.value.name = wert;
  },
});

// Nur die jüngste Antwort zählt: Wer schnell tippt, löst mehrere Anfragen
// aus, und eine langsame alte darf die neue nicht überschreiben.
let ladeNr = 0;
async function laden({ neu = true } = {}) {
  const nr = ++ladeNr;
  if (ansicht.value === 'kontakte' && kannNachKontakt.value) {
    try {
      const antwort = await quelle.unterhaltungen(fuerQuelle());
      if (nr !== ladeNr) return;
      unterhaltungen.value = antwort.items ?? [];
      serverfehler.value = antwort.serverfehler ?? null;
    } catch (e) {
      toast.error(String(e?.response?.data?.message ?? e));
    }
    return;
  }
  const seite = neu ? 1 : naechsteSeite.value;
  if (seite == null) return;
  try {
    const antwort = await quelle.liste(seite, fuerQuelle());
    if (nr !== ladeNr) return;
    const alles = neu ? antwort.items : [...liste.value, ...antwort.items];
    /*
     * NACH DEM ANHÄNGEN NEU SORTIEREN. Das Programm blättert zwei Wege
     * getrennt, und Seite 2 des einen kann neuer sein als Seite 1 des
     * anderen. Ohne das steht eine Nachricht von heute unter einer von
     * gestern, sobald jemand „Mehr laden" drückt. Drüben ist es eine Quelle
     * und die Sortierung folgenlos — schaden kann sie dort nicht.
     */
    alles.sort((a, b) => String(b.created_at).localeCompare(String(a.created_at)));
    liste.value = alles;
    naechsteSeite.value = antwort.next_page ?? null;
    serverfehler.value = antwort.serverfehler ?? null;
  } catch (e) {
    toast.error(String(e?.response?.data?.message ?? e));
  }
}

/*
 * MIT ODER OHNE EMPFÄNGER. „+ Nachricht" ruft es leer; ein Klick auf einen
 * Namen im Adressbuch ruft es MIT — wer von dort kommt, weiss schon, an wen.
 *
 * Der Name wird GESETZT und nicht ergänzt: Ein Formular, das noch einen
 * halben Entwurf von vorhin trägt, schickt ihn sonst an den Falschen.
 */
function oeffneSchreiben(ziel = '', weg = '', antwort = null) {
  // Ohne das Konto eines Weges soll das Blatt nicht mit einer Wahl
  // aufmachen, die ins Leere führt.
  const gewaehlt = wege.value.includes(weg) ? weg
    : (wege.value.includes(entwurf.value.weg) ? entwurf.value.weg : (wege.value[0] ?? 'openany'));
  const name = String(ziel ?? '').trim();

  entwurf.value = {
    weg: gewaehlt,
    name: ['openany', 'nah'].includes(gewaehlt) ? name : '',
    mxid: gewaehlt === 'matrix' ? name : '',
    email: gewaehlt === 'email' ? name : '',
    // Eine Antwort per E-Mail: „Re: …" vorbelegt, und sie gehört in den
    // Faden der Mail, auf die geantwortet wird.
    betreff: antwort?.betreff != null ? `Re: ${String(antwort.betreff).replace(/^(\s*(re|aw|antw)\s*:\s*)+/i, '')}` : '',
    antwortAuf: antwort?.id ?? null,
    // Eine Antwort geht von dem Konto, bei dem die Nachricht ankam.
    von: antwort?.konto ?? kontenVon(gewaehlt)[0] ?? '',
    text: '',
    anhang: null,
    // Eine Antwort auf eine verschlüsselte Mail ist vorbelegt verschlüsselt
    // (Tiffy, 30.09.2026) -- sonst ginge die Antwort auf etwas Vertrauliches
    // aus Versehen im Klartext. Sonst gilt: nur auf Wunsch.
    verschluesseln: antwort?.pgp === 'verschluesselt',
  };
  schreibenOffen.value = true;
}
defineExpose({ oeffneSchreiben, laden });

watch([filter, ansicht], () => laden());

/*
 * DER VERLAUF LÄDT BEIM ÖFFNEN -- hier, nicht im Rahmen.
 *
 * Bis zum 24.09.2026 tat das `MessagesPanel.vue` der Webapp selbst
 * (`onMounted(loadMessages)`). Beim Zusammenlegen zu dieser Fläche ging der
 * Aufruf verloren, in BEIDEN Rahmen: Die Liste blieb leer, bis jemand eine
 * Nachricht schrieb, eine als gelesen markierte oder eine löschte -- erst
 * dann rief die Fläche `laden()`. Nichts schlug fehl, es stand nur nichts
 * da. Liegt der Aufruf hier, kann ihn kein Rahmen mehr vergessen.
 */
onMounted(async () => {
  await laden();
  // Das Programm sieht beim Öffnen gleich nach neuen Mails.
  if (props.mailVerbunden && typeof quelle.abholen === 'function') {
    try { if (await quelle.abholen()) await laden(); } catch { /* der Rahmen zeigt den Fehler */ }
  }
});

/** Auf demselben Weg zurück, auf dem es kam. */
function antworten(n) {
  // E-Mail: an die Adresse, auch wenn `peer` den Namen aus dem Adressbuch
  // zeigt.
  if (n.transport === 'email') oeffneSchreiben(n.antwort_an ?? n.peer ?? '', 'email', n);
  // Matrix: mit der Zeile, damit die Antwort vom selben Konto geht.
  else if (n.transport === 'matrix') oeffneSchreiben(n.peer ?? '', 'matrix', n);
  // Vor Ort: an die Person, nicht an ihren (selbstgewählten) Namen.
  else if (n.transport === 'nah') oeffneSchreiben(n.antwort_an ?? '', 'nah');
  else oeffneSchreiben(n.peer ?? '', n.transport);
}

async function senden() {
  const ziel = String(empfaenger.value ?? '').trim();
  const text = String(entwurf.value.text ?? '').trim();
  // Ein Anhang allein ist auch eine Nachricht.
  const anhang = anhaengbar.value ? entwurf.value.anhang : null;

  if (!ziel || (!text && !anhang)) {
    toast.error(t('settings.messages.fillFields'));
    return;
  }

  sendet.value = true;
  try {
    await quelle.senden(ziel, text, entwurf.value.weg, anhang, {
      betreff: entwurf.value.betreff,
      antwortAuf: entwurf.value.antwortAuf,
      von: entwurf.value.von || null,
      verschluesseln: pgpMoeglich.value && entwurf.value.verschluesseln,
    });
    toast.success(t('settings.messages.sendSuccess'));
    // Der Weg und der Empfänger bleiben stehen — die Nachricht nicht.
    entwurf.value = { ...entwurf.value, text: '', anhang: null, betreff: '', antwortAuf: null, verschluesseln: false };
    schreibenOffen.value = false;
    await laden();
    emit('geaendert');
  } catch (e) {
    toast.error(String(e?.response?.data?.message ?? e));
  } finally {
    sendet.value = false;
  }
}

watch([schreibenOffen, () => entwurf.value.weg], ([offen, weg]) => {
  if (offen && weg === 'nah') nahZieleLaden();
});

async function blockieren(n) {
  if (!(await confirmDialog(t('settings.messages.nahBlockierenText', { name: n.peer ?? '?' }), {
    title: t('settings.messages.nahBlockieren'),
    confirmLabel: t('settings.messages.nahBlockieren'),
  }))) return;
  try {
    await quelle.blockieren(n.antwort_an);
    toast.success(t('settings.messages.nahBlockiert', { name: n.peer ?? '?' }));
  } catch (e) {
    toast.error(String(e));
  }
}

async function gelesen(n) {
  try {
    await quelle.gelesen(n.id, n.transport);
    await laden();
    emit('geaendert');
  } catch (e) {
    toast.error(t('settings.messages.markReadFailed'));
  }
}

/*
 * E-MAIL LÖSCHEN FRAGT ANDERS: nur hier ausblenden, oder auch auf dem
 * Mailserver (in dessen Papierkorb). Tiffy, 29.09.2026 -- die Wahl steht beim
 * Löschen, nicht in den Einstellungen.
 */
const mailLoeschen = ref(null); // Zeile | null
async function mailLoeschenMit(auchServer) {
  const n = mailLoeschen.value;
  mailLoeschen.value = null;
  if (!n) return;
  try {
    await quelle.loeschen(n.id, n.transport, auchServer);
    await laden();
    emit('geaendert');
  } catch (e) {
    toast.error(String(e?.message ?? e) || t('settings.messages.deleteFailed'));
  }
}

async function loeschen(n) {
  if (n.transport === 'email') {
    mailLoeschen.value = n;
    return;
  }
  /*
   * DIE FRAGE HÄNGT AM WEG, weil die Folge daran hängt: Eine interne
   * Nachricht verschwindet nur für die eigene Seite, eine Matrix-Nachricht
   * bleibt im Raum stehen und geht nur auf diesem Gerät weg.
   */
  const frage = n.transport === 'matrix'
    ? t('settings.messages.deleteConfirmMatrix')
    : t('settings.messages.deleteConfirm');

  if (!(await confirmDelete(frage))) return;

  try {
    await quelle.loeschen(n.id, n.transport);
    await laden();
    emit('geaendert');
  } catch (e) {
    toast.error(t('settings.messages.deleteFailed'));
  }
}

/*
 * EINEN ANHANG ANSEHEN. PDFs im PDF-Betrachter (nur lesen), Bilder groß,
 * alles andere wird gespeichert -- in der Webapp als Download, im Programm
 * in Dateien (`anhangSpeichern` der Quelle).
 */
const anhangLaden = (n, i = 0) => () => quelle.anhang(n, i);
// Eine Mail kann mehrere Anhänge tragen, eine Matrix-Nachricht einen.
const anhaengeVon = (n) => (n.anhaenge?.length ? n.anhaenge : (n.anhang ? [n.anhang] : []));
const pdfAnhang = ref(null); // Nachricht | null
const bildAnhang = ref(null); // { url, name } | null

/*
 * VERSCHLÜSSELT GEKOMMEN, UNVERSCHLÜSSELT ABGELEGT? Was die Quelle
 * speichert, geht in den Abgleich. Bei einer PGP-Mail oder über Matrix (Ende
 * zu Ende) fragt die Fläche vorher -- „Aufs Gerät" bleibt ohne Abgleich.
 */
const kamVerschluesselt = (n) => n.pgp === 'verschluesselt' || n.transport === 'matrix';

async function anhangAufsGeraet(n, i = 0) {
  try {
    const ort = await quelle.anhangAufsGeraet(n, i);
    toast.success(t('settings.messages.anhangAufsGeraetOk', { ort }));
  } catch (e) {
    toast.error(String(e?.message ?? e));
  }
}

async function anhangSpeichern(n, i = 0) {
  const info = anhaengeVon(n)[i];
  try {
    if (typeof quelle.anhangSpeichern === 'function') {
      if (kamVerschluesselt(n) && !(await confirmDialog(t('settings.messages.anhangVerschluesseltText'), {
        title: t('settings.messages.anhangVerschluesseltTitel'),
        confirmLabel: t('settings.messages.anhangTrotzdem'),
      }))) return;
      const ort = await quelle.anhangSpeichern(n, i);
      toast.success(ort ? t('settings.messages.anhangGespeichertIn', { ort }) : t('settings.messages.anhangGespeichert'));
      return;
    }
    const daten = await quelle.anhang(n, i);
    const blob = daten instanceof Blob ? daten : new Blob([daten], { type: info.mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = info.name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 60_000);
  } catch (e) {
    toast.error(String(e?.response?.data?.message ?? e?.message ?? e));
  }
}

function anhangOeffnen(n, vorschau, i = 0) {
  const info = anhaengeVon(n)[i] ?? {};
  const mime = info.mime ?? '';
  if (mime.includes('pdf')) pdfAnhang.value = { n, i, name: info.name };
  else if (mime.startsWith('image/') && vorschau) bildAnhang.value = { url: vorschau, name: info.name };
  else anhangSpeichern(n, i);
}

// Lange Nachrichten einklappen, damit ein einzelner Roman die Liste nicht
// sprengt.
const CLAMP_ZEICHEN = 500;
const CLAMP_ZEILEN = 6;
const ausgeklappt = ref(new Set());
const brauchtKlapp = (n) => {
  const body = String(n.body ?? '');
  return body.length > CLAMP_ZEICHEN || body.split('\n').length > CLAMP_ZEILEN;
};
const istAusgeklappt = (n) => ausgeklappt.value.has(n.id);
const klappen = (n) => {
  const naechste = new Set(ausgeklappt.value);
  if (naechste.has(n.id)) naechste.delete(n.id);
  else naechste.add(n.id);
  ausgeklappt.value = naechste;
};
</script>

<template>
  <ModulePage>
    <ModuleHeader :icon="Mail">
      <!-- EIN Wurzel-Element im Schlitz, und kein Titel: Welches Modul offen
           ist, zeigt die Navigation (siehe ModuleHeader).

           Drei Bedienelemente auf einer Linie. Die beiden Adressbuch-Wege
           tragen ihre Beschriftung erst ab `sm`: Auf dem Telefon wäre die
           Zeile sonst breiter als der Bildschirm, und das Symbol allein
           trägt dort ohnehin. -->
      <div class="flex items-center gap-2 sm:gap-3 shrink-0">
        <BaseButton variant="secondary" groesse="kopf" class="shrink-0"
                    :title="t('shell.modules.contacts.label')"
                    @click="emit('adressbuch')">
          <ContactIcon class="w-4 h-4" />
          <span class="hidden sm:inline">{{ t('shell.modules.contacts.label') }}</span>
        </BaseButton>

        <!-- EIGENES SYMBOL statt eines zweiten Plus: Auf dem Telefon fallen
             die Beschriftungen weg, und dann stünden hier zwei Pluszeichen
             nebeneinander — eines beschriftet, eines nackt. -->
        <BaseButton variant="secondary" groesse="kopf" class="shrink-0"
                    :title="t('contacts.add')"
                    @click="emit('neuer-kontakt')">
          <UserPlus class="w-4 h-4" />
          <span class="hidden sm:inline">{{ t('contacts.add') }}</span>
        </BaseButton>

        <BaseButton groesse="kopf" class="shrink-0" :disabled="!bereit" @click="oeffneSchreiben()">
          <Plus class="w-4 h-4" />
          <span>{{ t('settings.messages.newMessage') }}</span>
        </BaseButton>
      </div>
    </ModuleHeader>

    <!-- Was nur ein Rahmen kennt: die Matrix-Anmeldung des Programms, seine
         Störungsmeldungen. -->
    <slot name="oben" />

    <!-- Der Verlauf steht auch ohne Netz. Dass ein Teil fehlt, muss dann
         dastehen — sonst sieht eine halbe Liste aus wie eine ganze. -->
    <p v-if="serverfehler" class="mb-4 text-sm text-leise flex items-start gap-2">
      <span class="min-w-0 break-words">{{ serverfehler }}</span>
    </p>

    <div class="bg-flaeche border border-linie p-4 sm:p-6 rounded-xl shadow-sm">
      <!-- Suche, Filter, Ansicht (Tiffy, 01.10.2026). -->
      <div class="mb-4 space-y-3">
        <div class="flex flex-wrap items-center gap-2">
          <label class="relative flex-1 min-w-[12rem]">
            <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-leise pointer-events-none" />
            <input v-model="suchtext" type="search" autocorrect="off" autocapitalize="off"
                   :placeholder="t('settings.messages.filterSearch')" :aria-label="t('settings.messages.filterSearch')"
                   class="w-full pl-9 pr-3 py-2 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm" />
          </label>
          <div v-if="kannNachKontakt" class="flex gap-1 p-1 rounded-xl border border-linie" role="radiogroup">
            <button v-for="[wert, Bild, text] in [['verlauf', ListOrdered, 'viewTimeline'], ['kontakte', Users, 'viewByContact']]" :key="wert"
                    type="button" role="radio" :aria-checked="ansicht === wert" @click="ansicht = wert"
                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold transition-colors cursor-pointer"
                    :class="ansicht === wert ? 'bg-marke-leise text-marke' : 'text-fliess hover:bg-auflage'">
              <component :is="Bild" class="w-3.5 h-3.5" /> {{ t(`settings.messages.${text}`) }}
            </button>
          </div>
        </div>
        <div class="flex flex-wrap gap-2" role="group" :aria-label="t('settings.messages.filterLabel')">
          <button type="button" :aria-pressed="filter.ungelesen" @click="umschalten('ungelesen')"
                  class="flex items-center gap-1.5 px-3 py-1 rounded-full border text-xs font-bold transition-colors cursor-pointer"
                  :class="filter.ungelesen ? 'border-marke bg-marke-leise text-marke' : 'border-linie text-fliess hover:bg-auflage'">
            <MailOpen class="w-3.5 h-3.5" /> {{ t('settings.messages.filterUnread') }}
          </button>
          <button type="button" :aria-pressed="filter.anhang" @click="umschalten('anhang')"
                  class="flex items-center gap-1.5 px-3 py-1 rounded-full border text-xs font-bold transition-colors cursor-pointer"
                  :class="filter.anhang ? 'border-marke bg-marke-leise text-marke' : 'border-linie text-fliess hover:bg-auflage'">
            <Paperclip class="w-3.5 h-3.5" /> {{ t('settings.messages.filterAttachment') }}
          </button>
          <template v-if="wege.length > 1">
            <button v-for="weg in wege" :key="weg" type="button" :aria-pressed="filter.wege.includes(weg)" @click="wegUmschalten(weg)"
                    class="px-3 py-1 rounded-full border text-xs font-bold transition-colors cursor-pointer"
                    :class="filter.wege.includes(weg) ? 'border-marke bg-marke-leise text-marke' : 'border-linie text-fliess hover:bg-auflage'">
              {{ t(`settings.messages.channel_${weg}`) }}
            </button>
          </template>
        </div>
        <div v-if="mitName && ansicht === 'verlauf'" class="flex items-center gap-2">
          <span class="flex items-center gap-1 pl-3 pr-1 py-1 rounded-full bg-marke-leise text-marke text-xs font-bold min-w-0">
            <span class="truncate">{{ t('settings.messages.withContact', { name: mitName }) }}</span>
            <button type="button" class="p-0.5 rounded-full hover:bg-marke/20 cursor-pointer shrink-0"
                    :title="t('settings.messages.clearWith')" :aria-label="t('settings.messages.clearWith')"
                    @click="unterhaltungSchliessen">
              <X class="w-3.5 h-3.5" />
            </button>
          </span>
        </div>
      </div>

      <!-- Nach Kontakt: je Gegenüber eine Zeile. -->
      <template v-if="ansicht === 'kontakte' && kannNachKontakt">
        <div v-if="unterhaltungen.length === 0" class="flex flex-col items-center justify-center py-12 text-leise space-y-2">
          <Users class="w-8 h-8 opacity-40" />
          <p class="text-sm font-medium">{{ gefiltert ? t('settings.messages.noMatches') : t('settings.messages.noMessages') }}</p>
        </div>
        <ul v-else class="divide-y divide-linie rounded-xl border border-linie">
          <li v-for="u in unterhaltungen" :key="u.mit.join('|')">
            <button type="button" class="w-full flex items-start gap-3 p-3 text-left cursor-pointer hover:bg-auflage"
                    @click="unterhaltungOeffnen(u)">
              <span class="flex-1 min-w-0 space-y-0.5">
                <span class="flex items-center gap-2">
                  <span class="text-sm text-schrift truncate" :class="u.ungelesen ? 'font-extrabold' : 'font-bold'">
                    <template v-for="(h, k) in trefferTeile(u.name)" :key="k"><mark v-if="h.treffer" class="treffer">{{ h.t }}</mark><template v-else>{{ h.t }}</template></template>
                  </span>
                  <span v-for="weg in [...new Set(u.mit.map((m) => m.split(':')[0]))]" :key="weg"
                        class="px-1.5 py-0.5 rounded-full text-[10px] font-extrabold bg-marke-leise text-marke shrink-0">
                    {{ t(`settings.messages.channel_${weg}`) }}
                  </span>
                </span>
                <span class="block text-xs text-leise truncate">
                  {{ u.zuletzt.von_mir ? t('settings.messages.youPrefix') : '' }}{{ u.zuletzt.betreff || u.zuletzt.body || (u.zuletzt.anhang?.name ?? u.zuletzt.anhaenge?.[0]?.name ?? '') }}
                </span>
              </span>
              <span class="flex flex-col items-end gap-1 shrink-0">
                <span class="text-xs text-leise font-medium">{{ formatDateTime(u.zuletzt.created_at) }}</span>
                <span v-if="u.ungelesen" class="px-2 py-0.5 rounded-full bg-marke text-white text-[10px] font-extrabold"
                      :title="t('settings.messages.unreadCount', { n: u.ungelesen })">{{ u.ungelesen }}</span>
              </span>
            </button>
          </li>
        </ul>
      </template>

      <div v-else-if="liste.length === 0" class="flex flex-col items-center justify-center py-12 text-leise space-y-2">
        <MessageSquare class="w-8 h-8 opacity-40" />
        <p class="text-sm font-medium">{{ gefiltert ? t('settings.messages.noMatches') : t('settings.messages.noMessages') }}</p>
      </div>

      <div v-else class="space-y-3">
        <div v-for="n in liste" :key="`${n.transport}:${n.id}`"
             class="p-4 rounded-xl border transition-all space-y-2"
             :class="n.von_mir
               ? 'bg-slate-50/50 dark:bg-slate-950/30 border-slate-100 dark:border-slate-800/60'
               : (n.read_at ? 'bg-marke-leise/20 border-slate-100 dark:border-slate-800/60' : 'bg-marke-leise/60 border-marke')">
          <div class="flex items-center justify-between gap-2 text-xs">
            <div class="flex items-center gap-2 min-w-0">
              <span class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0"
                    :class="n.von_mir ? 'bg-slate-200 dark:bg-slate-800 text-fliess' : 'bg-marke-leise text-marke'">
                {{ n.von_mir ? t('settings.messages.sent') : t('settings.messages.received') }}
              </span>

              <!-- HERKUNFT, nur wenn sie überrascht. Der interne Weg bekommt
                   keine Marke: Er ist der Normalfall, und eine Marke an jeder
                   Zeile sagt nichts mehr. -->
              <span v-if="n.transport === 'matrix'"
                    class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-marke-leise text-marke">
                {{ t('settings.messages.viaMatrix') }}
              </span>
              <span v-else-if="n.transport === 'email'"
                    class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-marke-leise text-marke">
                {{ t('settings.messages.viaEmail') }}
              </span>
              <span v-else-if="n.transport === 'nah'"
                    class="flex items-center gap-1 px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-marke-leise text-marke">
                <Radar class="w-3 h-3" /> {{ t('settings.messages.channel_nah') }}
              </span>
              <!-- Vor Ort: kein gemeinsames Projekt -- der Name ist
                   selbstgewählt und ungeprüft. -->
              <span v-if="n.unbekannt"
                    class="px-2 py-0.5 rounded-full font-extrabold text-[10px] shrink-0 bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300">
                {{ t('settings.messages.nahUnbekannt') }}
              </span>
              <span v-if="n.wartet" class="text-leise font-medium shrink-0">
                {{ n.transport === 'nah' ? t('settings.messages.nahWartet') : t('settings.messages.wartetNetz') }}
              </span>
              <!-- Der Server hat eine wartende Nachricht abgelehnt: Sie geht
                   nicht mehr hinaus; der Grund steht im Titel. -->
              <span v-if="n.fehler" :title="n.transport === 'nah' ? nahGrund(n.fehler) : n.fehler" class="text-rose-600 dark:text-rose-400 font-bold shrink-0">
                {{ t('settings.messages.nichtZugestellt') }}
              </span>

              <span class="font-bold text-fliess break-all">
                {{ n.von_mir
                  ? t('settings.messages.toLabel', { name: n.peer ?? '?' })
                  : t('settings.messages.fromLabel', { name: n.peer ?? '?' }) }}
              </span>
              <!-- OpenPGP: verschlüsselt, und was die Signatur sagt. -->
              <span v-if="n.pgp === 'verschluesselt'" class="flex items-center gap-1 text-marke font-bold shrink-0">
                <Lock class="w-3.5 h-3.5" /> {{ t('settings.messages.pgpEncrypted') }}
              </span>
              <span v-if="n.signatur === 'gueltig'" class="flex items-center gap-1 text-marke font-bold shrink-0">
                <ShieldCheck class="w-3.5 h-3.5" /> {{ t('settings.messages.pgpSigValid') }}
              </span>
              <span v-else-if="n.signatur === 'ungueltig'" class="flex items-center gap-1 text-rose-600 dark:text-rose-400 font-bold shrink-0">
                <ShieldAlert class="w-3.5 h-3.5" /> {{ t('settings.messages.pgpSigInvalid') }}
              </span>
              <span v-else-if="n.signatur === 'unbekannt'" class="flex items-center gap-1 text-leise font-bold shrink-0">
                <ShieldQuestion class="w-3.5 h-3.5" /> {{ t('settings.messages.pgpSigUnknown') }}
              </span>
              <span v-if="n.konto && mehrereKonten(n.transport)"
                    class="text-leise font-medium break-all">
                {{ t('settings.messages.viaMailbox', { adresse: n.konto }) }}
              </span>
            </div>
            <span class="text-leise font-medium shrink-0">{{ formatDateTime(n.created_at) }}</span>
          </div>

          <p v-if="n.transport === 'email' && n.betreff" class="text-sm font-extrabold text-schrift break-words"><template v-for="(h, k) in trefferTeile(n.betreff)" :key="k"><mark v-if="h.treffer" class="treffer">{{ h.t }}</mark><template v-else>{{ h.t }}</template></template></p>

          <div v-if="anhaengeVon(n).length" class="flex flex-wrap gap-2">
            <AnhangKachel v-for="(a, i) in anhaengeVon(n)" :key="i" :anhang="a" :laden="anhangLaden(n, i)"
                          :aufs-geraet="typeof quelle.anhangAufsGeraet === 'function'"
                          @oeffnen="(vorschau) => anhangOeffnen(n, vorschau, i)" @speichern="anhangSpeichern(n, i)"
                          @aufs-geraet="anhangAufsGeraet(n, i)" />
          </div>

          <p v-if="n.pgp === 'unlesbar'" class="text-sm text-leise italic flex items-start gap-2">
            <Lock class="w-4 h-4 shrink-0 mt-0.5" /> {{ t('settings.messages.pgpUnreadable') }}
          </p>
          <p v-if="n.body" class="text-sm text-schrift font-medium whitespace-pre-line leading-relaxed break-words"
             :class="brauchtKlapp(n) && !istAusgeklappt(n) ? 'line-clamp-6' : ''"
          ><template v-for="(s, i) in zerleger(n.body)" :key="i"><a
            v-if="s.type === 'url'"
            :href="s.href"
            target="_blank"
            rel="noopener noreferrer"
            class="text-marke font-bold underline break-all"
          >{{ s.label }}</a><template v-else><template v-for="(h, k) in trefferTeile(s.value)" :key="k"><mark v-if="h.treffer" class="treffer">{{ h.t }}</mark><template v-else>{{ h.t }}</template></template></template></template></p>

          <button v-if="brauchtKlapp(n)" type="button" @click="klappen(n)"
                  class="text-xs font-bold text-marke hover:underline cursor-pointer">
            {{ istAusgeklappt(n) ? t('settings.messages.showLess') : t('settings.messages.readMore') }}
          </button>

          <div class="flex justify-end pt-1 gap-3">
            <button v-if="!n.von_mir && !n.read_at" type="button" @click="gelesen(n)"
                    class="text-xs font-bold text-marke hover:underline cursor-pointer">
              {{ t('settings.messages.markAsRead') }}
            </button>
            <button v-if="n.peer" type="button" @click="antworten(n)"
                    class="text-xs font-bold text-marke hover:underline cursor-pointer">
              {{ t('settings.messages.reply') }}
            </button>
            <button v-if="n.transport === 'nah' && !n.von_mir && typeof quelle.blockieren === 'function'" type="button" @click="blockieren(n)"
                    class="text-xs font-bold text-rose-600 dark:text-rose-400 hover:underline flex items-center gap-1 cursor-pointer">
              <Ban class="w-3.5 h-3.5" />
              <span>{{ t('settings.messages.nahBlockieren') }}</span>
            </button>
            <button type="button" @click="loeschen(n)"
                    class="text-xs font-bold text-rose-600 dark:text-rose-400 hover:underline flex items-center gap-1 cursor-pointer">
              <Trash2 class="w-3.5 h-3.5" />
              <span>{{ t('common.delete') }}</span>
            </button>
          </div>
        </div>

        <div v-if="naechsteSeite" class="text-center pt-1">
          <button type="button" @click="laden({ neu: false })"
                  class="px-4 py-1.5 text-xs font-bold text-marke hover:bg-marke-leise rounded-xl cursor-pointer">
            {{ t('settings.messages.loadMore') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Schreiben. `ueber-tastatur` hebt das Blatt über die Bildschirmtastatur;
         am Rechner ist `--rand-tastatur` schlicht 0. -->
    <div v-if="schreibenOffen"
         class="fixed inset-0 z-50 flex items-end sm:items-center justify-center p-0 sm:p-4 ueber-tastatur">
      <div class="absolute inset-0 bg-slate-900/50 backdrop-blur-sm" @click="schreibenOffen = false"></div>
      <div role="dialog" aria-modal="true" aria-labelledby="nachricht-titel"
           class="relative w-full sm:max-w-lg bg-flaeche border border-linie rounded-t-xl sm:rounded-xl shadow-2xl p-6 space-y-4 animate-modal-in mb-16 sm:mb-0">
        <div class="flex items-center justify-between gap-4">
          <h2 id="nachricht-titel" class="text-lg font-extrabold text-schrift">{{ t('settings.messages.writeMessage') }}</h2>
          <button type="button" @click="schreibenOffen = false" :aria-label="t('common.close')"
                  class="p-2 -mr-2 text-slate-400 hover:bg-auflage rounded-xl transition-colors cursor-pointer">
            <X class="w-5 h-5" />
          </button>
        </div>

        <form class="space-y-4" @submit.prevent="senden">
          <!-- DER UMSCHALTER STEHT VOR DEM EMPFÄNGER, weil er entscheidet,
               was dort hineingehört. -->
          <div v-if="wege.length > 1" class="space-y-1.5">
            <span class="block text-sm font-bold text-fliess">{{ t('settings.messages.channel') }}</span>
            <div class="flex gap-2" role="radiogroup" :aria-label="t('settings.messages.channel')">
              <button v-for="kanal in wege" :key="kanal"
                      type="button" role="radio" :aria-checked="entwurf.weg === kanal"
                      @click="entwurf.weg = kanal"
                      class="px-4 py-2 rounded-xl border text-sm font-bold transition-colors cursor-pointer"
                      :class="entwurf.weg === kanal
                        ? 'border-marke bg-marke-leise text-marke'
                        : 'border-linie text-fliess hover:bg-auflage'">
                {{ t(`settings.messages.channel_${kanal}`) }}
              </button>
            </div>
            <p v-if="wegHinweise?.[entwurf.weg]" class="text-xs text-leise">
              {{ wegHinweise[entwurf.weg] }}
            </p>
          </div>

          <!-- Vor Ort: eine Wahl statt eines Namens -- wen es gibt, weiß nur
               das Gerät. -->
          <label v-if="entwurf.weg === 'nah'" class="block space-y-1.5">
            <span class="block text-sm font-bold text-fliess">{{ t('settings.messages.recipientNah') }}</span>
            <select v-model="entwurf.name" required
                    class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm">
              <option value="" disabled>{{ nahLaedt ? t('common.loading') : t('settings.messages.recipientNahWaehlen') }}</option>
              <option v-for="z in nahZiele" :key="z.ziel" :value="z.ziel">
                {{ z.name }}{{ z.da ? ` · ${t('settings.messages.nahDa')}` : '' }}{{ z.unbekannt ? ` · ${t('settings.messages.nahUnbekannt')}` : '' }}
              </option>
            </select>
            <p v-if="!nahLaedt && !nahZiele.length" class="text-xs text-leise">{{ t('settings.messages.nahNiemand') }}</p>
            <p v-if="nahUnbestaetigt" class="text-xs text-warnung">{{ t('settings.messages.nahErstBestaetigen') }}</p>
            <button type="button" class="text-xs font-bold text-marke hover:underline cursor-pointer" @click="nahZieleLaden">
              {{ t('settings.messages.nahSuchen') }}
            </button>
          </label>
          <label v-else class="block space-y-1.5">
            <span class="block text-sm font-bold text-fliess">
              {{ entwurf.weg === 'matrix' ? t('settings.messages.recipientMatrix')
                : entwurf.weg === 'email' ? t('settings.messages.recipientEmail') : t('settings.messages.recipient') }}
            </span>
            <input v-model="empfaenger" required autocorrect="off" autocapitalize="off"
                   :type="entwurf.weg === 'email' ? 'email' : 'text'"
                   :placeholder="entwurf.weg === 'matrix'
                     ? t('settings.messages.recipientMatrixPlaceholder')
                     : entwurf.weg === 'email' ? t('settings.messages.recipientEmailPlaceholder')
                       : t('settings.messages.recipientPlaceholder')"
                   class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm" />
          </label>

          <!-- Von welchem eigenen Konto -- erst ab zweien eine Wahl. Eine
               Antwort geht von dem, bei dem die Nachricht ankam. -->
          <label v-if="vonListe.length > 1" class="block space-y-1.5">
            <span class="block text-sm font-bold text-fliess">{{ t('settings.messages.fromMailbox') }}</span>
            <select v-model="entwurf.von" :disabled="Boolean(entwurf.antwortAuf)"
                    class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm disabled:opacity-70">
              <option v-for="a in vonListe" :key="a" :value="a">{{ a }}</option>
            </select>
          </label>

          <!-- Der Betreff: nur bei E-Mail, als eigenes Feld (Tiffy, 29.09.2026). -->
          <label v-if="entwurf.weg === 'email'" class="block space-y-1.5">
            <span class="block text-sm font-bold text-fliess">{{ t('settings.messages.subject') }}</span>
            <input v-model="entwurf.betreff" type="text"
                   :placeholder="t('settings.messages.subjectPlaceholder')"
                   class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm" />
          </label>

          <label class="block space-y-1.5">
            <span class="block text-sm font-bold text-fliess">{{ t('settings.messages.message') }}</span>
            <textarea v-model="entwurf.text" :required="!(anhaengbar && entwurf.anhang)" rows="4"
                      :placeholder="t('settings.messages.messagePlaceholder')"
                      class="w-full px-4 py-3 bg-vertieft border border-linie rounded-xl focus:ring-2 focus:ring-marke text-schrift font-medium text-sm resize-none"></textarea>
          </label>

          <AnhangWahl v-if="anhaengbar" v-model="entwurf.anhang" :grenze="anhangGrenze" :disabled="sendet" />

          <!-- Verschlüsseln: ein Schalter, darunter, ob es geht. -->
          <div v-if="pgpMoeglich" class="space-y-1.5">
            <label class="flex items-center gap-2 text-sm font-bold text-fliess cursor-pointer">
              <input v-model="entwurf.verschluesseln" type="checkbox" />
              <Lock class="w-4 h-4 text-marke" /> {{ t('settings.messages.encrypt') }}
            </label>
            <p v-if="pgp === 'sucht'" class="text-xs text-leise flex items-center gap-1.5">
              <Loader2 class="w-3.5 h-3.5 animate-spin" /> {{ t('settings.messages.encryptChecking') }}
            </p>
            <template v-else-if="pgp">
              <p v-if="!pgp.eigener" class="text-xs" :class="entwurf.verschluesseln ? 'text-warnung' : 'text-leise'">
                {{ t('settings.messages.encryptOwnMissing') }}
              </p>
              <p v-else-if="!pgp.empfaenger" class="text-xs" :class="entwurf.verschluesseln ? 'text-warnung' : 'text-leise'">
                {{ t('settings.messages.encryptRecipientMissing', { adresse: entwurf.email.trim() }) }}
              </p>
              <p v-else class="text-xs text-leise">
                {{ t('settings.messages.encryptReady', { fingerabdruck: fingerabdruckKurz(pgp.empfaenger) }) }}
              </p>
            </template>
          </div>

          <BaseButton v-if="nahUnbestaetigt && typeof quelle.kontaktBestaetigen === 'function'" type="button"
                      :disabled="nahBestaetigt" class="w-full" groesse="normal" @click="nahBestaetigen">
            <Loader2 v-if="nahBestaetigt" class="w-4 h-4 animate-spin" />
            <UserCheck v-else class="w-4 h-4" />
            <span>{{ t('settings.messages.nahBestaetigen') }}</span>
          </BaseButton>
          <BaseButton v-else type="submit" :disabled="sendet || nahUnbestaetigt || (pgpMoeglich && entwurf.verschluesseln && !pgpBereit)" class="w-full" groesse="normal">
            <Loader2 v-if="sendet" class="w-4 h-4 animate-spin" />
            <Send v-else class="w-4 h-4" />
            <span>{{ t('settings.messages.send') }}</span>
          </BaseButton>
        </form>
      </div>
    </div>
    <!-- E-Mail löschen: nur hier, oder auch auf dem Mailserver. -->
    <BaseModal v-if="mailLoeschen" :title="t('settings.messages.deleteEmailTitle')" @close="mailLoeschen = null">
      <p class="text-sm text-fliess">{{ t('settings.messages.deleteEmailText') }}</p>
      <template #footer>
        <BaseButton variant="secondary" @click="mailLoeschen = null">{{ t('common.cancel') }}</BaseButton>
        <BaseButton variant="secondary" @click="mailLoeschenMit(false)">{{ t('settings.messages.deleteNurHier') }}</BaseButton>
        <BaseButton variant="danger" @click="mailLoeschenMit(true)">{{ t('settings.messages.deleteAuchServer') }}</BaseButton>
      </template>
    </BaseModal>

    <!-- Anhänge ansehen: PDF im Betrachter (nur lesen), Bilder groß. -->
    <PdfBetrachter v-if="pdfAnhang" :key="`${pdfAnhang.n.id}:${pdfAnhang.i}`" :name="pdfAnhang.name" :laden="anhangLaden(pdfAnhang.n, pdfAnhang.i)" @close="pdfAnhang = null">
      <template #aktionen>
        <button type="button" class="p-2 rounded-xl text-fliess hover:bg-auflage cursor-pointer"
                :title="t('settings.messages.anhangSpeichern')" :aria-label="t('settings.messages.anhangSpeichern')"
                @click="anhangSpeichern(pdfAnhang.n, pdfAnhang.i)">
          <Download class="w-5 h-5" />
        </button>
      </template>
    </PdfBetrachter>
    <div v-if="bildAnhang" class="fixed inset-0 z-[95] bg-slate-900/90 flex flex-col" role="dialog" aria-modal="true" :aria-label="bildAnhang.name"
         @click.self="bildAnhang = null">
      <div class="flex items-center justify-between gap-2 px-4 py-3 text-white" :style="{ paddingTop: 'calc(0.75rem + var(--rand-oben))' }">
        <span class="font-bold truncate">{{ bildAnhang.name }}</span>
        <button type="button" class="p-2 rounded-xl hover:bg-white/10 cursor-pointer" :aria-label="t('common.close')" @click="bildAnhang = null">
          <X class="w-5 h-5" />
        </button>
      </div>
      <div class="flex-1 min-h-0 flex items-center justify-center p-4" @click.self="bildAnhang = null">
        <img :src="bildAnhang.url" :alt="bildAnhang.name" class="max-w-full max-h-full object-contain" />
      </div>
    </div>
  </ModulePage>
</template>

<style scoped>
/* Ein Suchtreffer im Text: dezent in der Markenfarbe, in hell und dunkel lesbar. */
.treffer {
  background: color-mix(in srgb, var(--color-marke, #0d9488) 25%, transparent);
  color: inherit;
  border-radius: 0.2em;
  padding: 0 0.05em;
}
</style>
