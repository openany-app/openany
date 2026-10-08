/*
 * Die Wege, auf denen ein Kontakt erreichbar ist.
 *
 * WARUM HIER UND NICHT IN DER ANSICHT: Dieselbe Antwort braucht die Webapp
 * und `openany-app`, und dieselbe steht noch einmal in PHP
 * (`App\Models\ContactChannel::href()`), weil der Server sie in seinen
 * Vorlagen ebenfalls kennt. Drei Fassungen wären zwei zu viel; zwei sind das
 * Minimum, das zwei Sprachen kosten – und deshalb steht die hiesige an genau
 * einer Stelle und ist geprüft.
 *
 * WAS EINE ART IST: eine Auskunft darüber, was ein Klick tun kann. `tel:` und
 * `mailto:` versteht jedes Betriebssystem, eine Postanschrift versteht keines
 * – und das ist der einzige Unterschied, der die Oberfläche interessiert.
 */

/**
 * Muss zu `ContactChannel::ARTEN` passen; der Server weist Fremdes ab.
 *
 * DIE REIHENFOLGE IST DIE DES AUSWAHLFELDS, nicht das Alphabet. Vorn die
 * Kennungen zum Funken – sie sind der Grund, aus dem es dieses Adressbuch
 * überhaupt gibt. Telefon und E-Mail hinten, obwohl sie am häufigsten sind:
 * Sie haben oben ein offenes Feld, und wer trotzdem hier hinuntergeht, sucht
 * die zweite Nummer.
 */
export const WEG_ARTEN = [
  'matrix',
  'meshtastic',
  'openany',
  'address',
  'company',
  // Der Geburtstag steht hier und nicht unter „Erweitert", weil an ihm
  // später der Kalender hängt.
  'birthday',
  'other',
  'phone',
  'email',
];

/** Muss zu `ContactChannel::VCARD_PRAEFIX` passen. */
export const VCARD_PRAEFIX = 'vcard:';

/**
 * Die vCard-Felder, die unter „Erweitert" zur Auswahl stehen.
 *
 * EINE AUSWAHL, KEINE GRENZE. Was ein fremdes Adressbuch mitbringt, kommt
 * auch dann an, wenn es hier nicht steht — es bekommt seinen eigenen Namen
 * und geht genau so zurück. Diese Liste sagt nur, was jemand von Hand
 * anlegen kann, ohne den Feldnamen auswendig zu wissen.
 *
 * Ausgewählt nach einer Frage: Trägt das Feld eine Auskunft über den
 * MENSCHEN, und passt sie in eine Zeile? Deshalb kein `PHOTO` (Base64, viele
 * Kilobyte) und kein `PRODID` (sagt etwas über das Programm, das die Datei
 * geschrieben hat).
 */
export const VCARD_FELDER = [
  'nickname',
  'title',
  'role',
  'url',
  'impp',
  'anniversary',
  'categories',
  'related',
  'lang',
  'tz',
  'geo',
  'caluri',
  'fburl',
].map((feld) => VCARD_PRAEFIX + feld);

/**
 * Wie ein durchgereichtes Feld heißt, wenn niemand einen Namen dafür hat.
 *
 * `vcard:x-abrelatednames` → `X-ABRELATEDNAMES`. Roh und in Großbuchstaben,
 * so wie es in der Datei steht: Ein erfundener deutscher Name wäre eine
 * Übersetzung, die niemand nachschlagen kann.
 */
export function vcardName(art) {
  return String(art ?? '').slice(VCARD_PRAEFIX.length).toUpperCase();
}

/** Ist das ein durchgereichtes Feld? */
export function istDurchgereicht(art) {
  return String(art ?? '').startsWith(VCARD_PRAEFIX);
}

/**
 * Die zwei, die im Formular ein eigenes, offenes Feld bekommen.
 *
 * WARUM ÜBERHAUPT: Ohne sie fängt jeder neue Kontakt mit einem Klick auf
 * „Weg" und einem Griff ins Auswahlfeld an – für die zwei Angaben, die fast
 * jeder hat. Bis zum 07.09.2026 war es genau andersherum: Matrix und
 * Meshtastic hatten eigene Felder, Nummer und E-Mail keines.
 *
 * WARUM SIE TROTZDEM IM AUSWAHLFELD BLEIBEN: Es ist kein zweiter Mechanismus,
 * sondern eine Abkürzung. Wer eine zweite Nummer hat, trägt sie unten ein –
 * dieselbe Art, dieselbe Tabelle, dieselbe Zeile.
 *
 * WARUM DER openany-NAME NICHT DABEI IST, obwohl er in der Liste gleich
 * dahinter steht: Er ist die seltenere Angabe – die meisten Kontakte in einem
 * Adressbuch haben gar kein Konto hier. Ein drittes offenes Feld, das
 * meistens leer bleibt, macht das Formular länger, ohne es schneller zu
 * machen.
 */
export const OFFENE_FELDER = ['phone', 'email'];

/**
 * Wohin ein Klick führt – oder `null`, wenn nirgendwohin.
 *
 * `null` und nicht `'#'`: Ein Link, der nichts tut, ist schlimmer als kein
 * Link. Er sieht klickbar aus, und wer ihn antippt, bekommt nichts und weiß
 * nicht, ob es an ihm lag.
 */
/**
 * Der Name, an den dieser Weg eine Nachricht führen kann – oder `null`.
 *
 * DIE ENTSCHEIDUNG STEHT HIER, DIE ROUTE NICHT (10.09.2026).
 *
 * Ein openany-Name ist der einzige Weg, der nicht hinaus führt, sondern
 * hierher: Dahinter steht jemand mit einem Konto auf diesem Server. `wegZiel`
 * kann das nicht beantworten – es liefert, was ein BETRIEBSSYSTEM versteht
 * (`tel:`, `mailto:`), und dafür gibt es kein Protokoll.
 *
 * Warum die Frage trotzdem in diese Datei gehört und nicht in die Ansicht:
 * „Ist das ein Weg zu einem Konto, und wie heisst es?" ist dieselbe Frage in
 * der Webapp und in `openany-app`. Nur die ANTWORT sieht dort anders aus –
 * hier eine Route, dort ein Bildschirm. Was geteilt wird, ist die Frage.
 *
 * Deshalb gibt diese Funktion einen NAMEN zurück und keine Adresse. Eine
 * Route wäre eine Auskunft dieser Webapp und drüben falsch.
 */
export function nachrichtenName(weg) {
  const ziel = nachrichtenWeg(weg);
  return ziel?.kanal === 'openany' ? ziel.kennung : null;
}

/**
 * Der Weg, auf dem dieser Eintrag eine Nachricht tragen kann – oder `null`.
 *
 * DIE FRAGE HAT SEIT MATRIX ZWEI ANTWORTEN (12.09.2026).
 *
 * `nachrichtenName` oben konnte nur eine geben: einen openany-Namen. Solange
 * das der einzige Weg nach innen war, war das vollstaendig. Eine
 * Matrix-Kennung fuehrt aber ebenfalls in die Nachrichten – nur auf einem
 * anderen Weg, und der Verlauf muss wissen, auf welchem.
 *
 * WARUM EIN OBJEKT UND WEITER KEINE ROUTE: Die Begruendung darueber gilt
 * unveraendert. Was geteilt wird, ist die FRAGE („kann dieser Eintrag eine
 * Nachricht tragen, und an wen?"), nicht die Antwort – die ist hier eine
 * Route und in `openany-app` ein Bildschirm. Das Objekt nennt deshalb Kanal
 * und Kennung und ueberlaesst beiden Seiten, was sie daraus bauen.
 *
 * WARUM `nachrichtenName` BLEIBT und nicht ersetzt wird: Der Adressbuch-Code
 * fragt an mehreren Stellen genau das Alte – „ist das jemand von hier?" –,
 * und diese Frage ist weiterhin sinnvoll. Sie ist jetzt nur ein Sonderfall
 * dieser Funktion statt ihr eigener Mechanismus.
 */
export function nachrichtenWeg(weg) {
  const wert = String(weg?.value ?? '').trim();
  if (!wert) return null;

  switch (weg.kind) {
    case 'openany': return { kanal: 'openany', kennung: wert };
    // Ungeprueft: Ob hinter der Kennung wirklich jemand steht, weiss erst
    // der Homeserver. Eine Regex hier wuerde nur so tun, als wuesste sie es –
    // und an einer Kennung scheitern, die es laengst gibt.
    case 'matrix': return { kanal: 'matrix', kennung: wert };
    default: return null;
  }
}

export function wegZiel(weg) {
  const wert = String(weg?.value ?? '').trim();
  if (!wert) return null;

  switch (weg.kind) {
    // Leerzeichen, Klammern und Bindestriche gehören zur Lesbarkeit, nicht
    // zur Nummer – manche Wähler stolpern darüber. Das führende Plus bleibt.
    case 'phone': return `tel:${wert.replace(/[^\d+]/g, '')}`;
    case 'email': return `mailto:${wert}`;
    default: return null;
  }
}
