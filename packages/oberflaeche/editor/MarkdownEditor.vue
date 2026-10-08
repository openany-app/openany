<script setup>
import { ref, watch, onMounted, onBeforeUnmount, computed, shallowRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { Editor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import { Link, isAllowedUri } from '@tiptap/extension-link';
// Ab Tiptap 3 liegen die Listen- bzw. Tabellen-Knoten je in EINEM Paket;
// @tiptap/extension-task-item & Co. sind dort nur noch Re-Exporte davon.
import { TaskList, TaskItem } from '@tiptap/extension-list';
import { Table, TableRow, TableHeader, TableCell } from '@tiptap/extension-table';
import { Image } from '@tiptap/extension-image';
import { Markdown } from 'tiptap-markdown';
import Wikilink from './Wikilink';
import DeutscheAnfuehrungszeichen from './DeutscheAnfuehrungszeichen';
import LinkModal from './LinkModal.vue';
import BaseButton from '@oberflaeche/base/BaseButton.vue';
import {
  Bold, Italic, Strikethrough, Heading1, Heading2, Heading3,
  List, ListOrdered, ListChecks, Quote, Code, Code2, Link as LinkIcon,
  Minus, Table as TableIcon, Loader, Save, AlertCircle, Pencil,
  Hash, Brackets, Paperclip, BookMarked, X,
} from 'lucide-vue-next';

const { t, locale } = useI18n();

// Wiederverwendbarer WYSIWYG-Markdown-Editor (Tiptap).
// Quelle der Wahrheit ist Markdown (modelValue). Wird auch außerhalb des
// Notizen-Moduls gebraucht (z. B. Kanban-Kartenbeschreibungen).
const props = defineProps({
  modelValue: { type: String, default: '' },
  // Ob es einen Einfuegen-Dialog gibt (Webapp: ja, Programm: noch nicht).
  // Ohne ihn fuehrte die Bueroklammer ins Leere.
  einfuegbar: { type: Boolean, default: true },
  readOnly: { type: Boolean, default: false },
  // Autosave-Status vom Parent: null | 'saving' | 'saved' | 'error'
  saveStatus: { type: String, default: null },
  // Konkrete Fehlermeldung zum Status 'error' (z. B. "Notiz zu groß, max. 1 MB")
  saveError: { type: String, default: '' },
  // Aktuell ungenutzt (keine Placeholder-Extension eingebunden); defineProps()
  // wird gehoisted und kann daher nicht auf die lokale t()-Funktion zugreifen.
  placeholder: { type: String, default: 'Schreibe etwas auf…' },
  // [[Wikilinks]] (Obsidian-Stil) aktivieren – nur im Notizen-Modul.
  wikilinks: { type: Boolean, default: false },
  // Kandidaten fürs [[-Autocomplete: [{ id, title }]
  wikilinkSuggestions: { type: Array, default: () => [] },
  // Kandidaten fürs #-Autocomplete: [{ tag, count }]
  tagSuggestions: { type: Array, default: () => [] },
  // Notiz-Anhänge: relative Pfade (![](skizze.png)) -> Anzeige-URL.
  // null = Feature aus (Editor außerhalb der Notizen).
  assetResolver: { type: Function, default: null },
  // Kandidaten fürs [@-Zitier-Autocomplete: [{ key, author, year, title }]
  citations: { type: Array, default: () => [] },
});
const emit = defineEmits(['update:modelValue', 'retry', 'wikilink', 'insert', 'paste-image', 'asset-open']);

const editor = shallowRef(null);
const showSource = ref(false);
const rawMarkdown = ref(props.modelValue);
// Zuletzt nach außen emittiertes Markdown – verhindert Setz-/Emit-Schleifen.
let lastEmitted = props.modelValue;

// Bilder: im Markdown bleibt der relative Pfad (![](skizze.png)) stehen –
// nur die Anzeige läuft über den assetResolver (Pfad -> authentifizierte
// URL). Externe http(s)-Quellen werden unverändert durchgereicht.
const resolveSrc = (src) => {
  if (!src || /^[a-z][a-z0-9+.-]*:/i.test(src) || src.startsWith('/')) return src;
  return props.assetResolver ? (props.assetResolver(src) ?? src) : src;
};
const NoteImage = Image.extend({
  renderHTML({ HTMLAttributes }) {
    return ['img', { ...HTMLAttributes, src: resolveSrc(HTMLAttributes.src) }];
  },
});

// html:false => Roh-HTML im Markdown wird NICHT als HTML interpretiert
// (kein XSS-Passthrough).
const buildExtensions = () => [
  StarterKit.configure({
    heading: { levels: [1, 2, 3] },
    // Tiptap 3 legt Link und Underline neu ins StarterKit.
    //
    // link: Wir bringen unten eine eigene Variante mit (inclusive:false, eigene
    //   HTML-Attribute). Zwei Erweiterungen gleichen Namens vertraegt Tiptap
    //   nicht -- die aus dem Kit muss also weg.
    //
    // underline: Markdown kennt kein Unterstreichen. Die Auszeichnung liesse
    //   sich setzen (Strg+U), waere beim naechsten Speichern aber still weg,
    //   weil unsere Quelle der Wahrheit die .md-Datei ist. Was wir nicht
    //   speichern koennen, bieten wir gar nicht erst an.
    //
    // trailingNode (auch neu) bleibt AN: nachgemessen, es aendert das
    // serialisierte Markdown an keiner Stelle, verschafft aber einen Absatz
    // hinter einer Tabelle oder einem Codeblock am Dokumentende -- ohne den
    // kommt man dort mit dem Cursor nicht mehr heraus.
    link: false,
    underline: false,
  }),
  // `inclusive: false` ist kein Feinschliff, sondern behebt einen Fehler:
  // Die Link-Erweiterung setzt `inclusive()` auf den Wert von `autolink` –
  // mit autolink:true ist die Markierung also einschliessend, und alles, was
  // man direkt hinter einen Link tippt, landet noch IM Link. Man kam nur
  // heraus, indem man einen Absatz einfuegte.
  //
  // autolink bleibt an: Es haengt an einem eigenen Plugin, das den Text bei
  // jeder Aenderung durchsieht, und nicht an der Einschliesslichkeit der
  // Markierung. Eingefuegte und getippte URLs werden weiterhin erkannt.
  Link.extend({ inclusive: false }).configure({
    openOnClick: false,
    autolink: true,
    HTMLAttributes: { rel: 'noopener noreferrer nofollow', target: '_blank' },
  }),
  TaskList,
  TaskItem.configure({ nested: true }),
  Table.configure({ resizable: false }),
  TableRow,
  TableHeader,
  TableCell,
  // Sprachabhaengig, deshalb als Funktion: beim Umstellen der Sprache wird der
  // Editor nicht neu gebaut, die Regel fragt bei jedem " neu nach.
  DeutscheAnfuehrungszeichen.configure({ istDeutsch: () => locale.value === 'de' }),
  ...(props.wikilinks ? [Wikilink] : []),
  ...(props.assetResolver ? [NoteImage] : []),
  Markdown.configure({ html: false, tightLists: true, linkify: true, transformPastedText: true }),
];

// --- [[-Autocomplete (nur mit wikilinks-Prop) ---
// Zeigt beim Tippen von "[[query" eine Titel-Auswahl an der Cursor-Position.
const rootEl = ref(null);
const wikiSuggest = ref(null); // { items, from, active, top, left }

const closeWikiSuggest = () => { wikiSuggest.value = null; };

const checkWikiSuggest = () => {
  if (!props.wikilinks || props.readOnly || !editor.value) return;
  const { state, view } = editor.value;
  const { $from, empty, from } = state.selection;
  if (!empty) { closeWikiSuggest(); return; }
  // Text vor dem Cursor im aktuellen Block; Nodes als Platzhalter, damit
  // ein "[[" nicht über Node-Grenzen hinweg matcht.
  const textBefore = $from.parent.textBetween(Math.max(0, $from.parentOffset - 120), $from.parentOffset, null, '￼');
  const m = /\[\[([^[\]\n￼]*)$/.exec(textBefore);
  if (!m) { closeWikiSuggest(); return; }
  const query = m[1].toLowerCase();
  const items = props.wikilinkSuggestions
    .filter((s) => s.title && s.title.toLowerCase().includes(query))
    .slice(0, 8);
  if (!items.length) { closeWikiSuggest(); return; }
  const coords = view.coordsAtPos(from);
  const rect = rootEl.value?.getBoundingClientRect();
  wikiSuggest.value = {
    items,
    active: 0,
    from: from - m[1].length - 2, // Start des "[["
    top: rect ? coords.bottom - rect.top + 4 : 0,
    left: rect ? Math.max(8, coords.left - rect.left) : 0,
  };
};

// --- Weblinks öffnen ---
// openOnClick bleibt aus, weil ein Klick im bearbeitbaren Dokument den Cursor
// setzen muss – sonst ließe sich ein Linktext nie mehr korrigieren. Stattdessen:
// im Lesemodus öffnet der einfache Klick, beim Bearbeiten zeigt er eine Blase
// mit der Zieladresse. Die ist zugleich die Antwort für Mobilgeräte (keine
// Zusatztaste) und macht sichtbar, wohin ein Link wirklich führt.
const linkBubble = ref(null); // { href, top, left }
const closeLinkBubble = () => { linkBubble.value = null; };

const openExternal = (href) => {
  // Prüfung über Tiptaps eigene Logik statt handgeschriebener Regex – sie
  // weist u. a. javascript:-Adressen ab.
  if (!isAllowedUri(href)) return;
  window.open(href, '_blank', 'noopener,noreferrer');
};

const openBubbleLink = () => {
  const href = linkBubble.value?.href;
  closeLinkBubble();
  if (href) openExternal(href);
};

// --- [@-Zitier-Autocomplete (nur mit citations) ---
const citeSuggest = ref(null); // { items, from, active, top, left }
const closeCiteSuggest = () => { citeSuggest.value = null; };

const checkCiteSuggest = () => {
  if (!props.citations.length || props.readOnly || !editor.value) { closeCiteSuggest(); return; }
  const { state, view } = editor.value;
  const { $from, empty, from } = state.selection;
  if (!empty) { closeCiteSuggest(); return; }
  const textBefore = $from.parent.textBetween(Math.max(0, $from.parentOffset - 120), $from.parentOffset, null, '￼');
  const m = /\[@([^\s[\]￼]*)$/.exec(textBefore);
  if (!m) { closeCiteSuggest(); return; }
  const query = m[1].toLowerCase();
  const items = props.citations
    .filter((c) => !query
      || c.key.toLowerCase().includes(query)
      || (c.author || '').toLowerCase().includes(query)
      || (c.title || '').toLowerCase().includes(query))
    .slice(0, 8);
  if (!items.length) { closeCiteSuggest(); return; }
  const coords = view.coordsAtPos(from);
  const rect = rootEl.value?.getBoundingClientRect();
  citeSuggest.value = {
    items,
    active: 0,
    from: from - m[1].length - 2, // Start des "[@"
    top: rect ? coords.bottom - rect.top + 4 : 0,
    left: rect ? Math.max(8, coords.left - rect.left) : 0,
  };
};

const pickCitation = (item) => {
  if (!citeSuggest.value) return;
  const to = editor.value.state.selection.from;
  editor.value.chain().focus()
    .insertContentAt({ from: citeSuggest.value.from, to }, `[@${item.key}]`)
    .run();
  closeCiteSuggest();
};

// --- #-Tag-Autocomplete (nur mit tagSuggestions) ---
// Zeigt beim Tippen von "#query" (Zeilenanfang/nach Whitespace) die
// vorhandenen Tags – Wiederverwendungshilfe gegen Fast-Duplikate.
const tagSuggest = ref(null);
const closeTagSuggest = () => { tagSuggest.value = null; };

const checkTagSuggest = () => {
  if (!props.tagSuggestions.length || props.readOnly || !editor.value) { closeTagSuggest(); return; }
  const { state, view } = editor.value;
  const { $from, empty, from } = state.selection;
  if (!empty) { closeTagSuggest(); return; }
  const textBefore = $from.parent.textBetween(Math.max(0, $from.parentOffset - 60), $from.parentOffset, null, '￼');
  // # am Zeilenanfang oder nach Whitespace, gefolgt von Tag-Zeichen (ohne
  // Leerzeichen). '/' erlaubt verschachtelte Tags (#projekt/unterthema).
  // Ein "# " (Überschrift) matcht dadurch nicht.
  const m = /(?:^|\s)#([\p{L}\p{N}_/-]*)$/u.exec(textBefore);
  if (!m) { closeTagSuggest(); return; }
  const query = m[1].toLowerCase();
  const items = props.tagSuggestions
    .filter((tg) => !query || tg.tag.toLowerCase().includes(query))
    .slice(0, 8);
  if (!items.length) { closeTagSuggest(); return; }
  const coords = view.coordsAtPos(from);
  const rect = rootEl.value?.getBoundingClientRect();
  tagSuggest.value = {
    items,
    active: 0,
    from: from - m[1].length - 1, // Start des "#"
    top: rect ? coords.bottom - rect.top + 4 : 0,
    left: rect ? Math.max(8, coords.left - rect.left) : 0,
  };
};

const pickTag = (item) => {
  if (!tagSuggest.value) return;
  const to = editor.value.state.selection.from;
  editor.value.chain().focus()
    .insertContentAt({ from: tagSuggest.value.from, to }, `#${item.tag} `)
    .run();
  closeTagSuggest();
};

const pickWikilink = (item) => {
  if (!wikiSuggest.value) return;
  const to = editor.value.state.selection.from;
  editor.value.chain().focus()
    .insertContentAt({ from: wikiSuggest.value.from, to }, [{ type: 'wikilink', attrs: { title: item.title } }])
    .run();
  closeWikiSuggest();
};

// Tastatursteuerung der Dropdowns (Pfeile/Enter/Tab/Escape).
const handleSuggestKey = (event) => {
  if (event.key === 'Escape' && linkBubble.value) { closeLinkBubble(); return true; }
  const s = wikiSuggest.value ?? citeSuggest.value ?? tagSuggest.value;
  if (!s) return false;
  const pick = wikiSuggest.value ? pickWikilink : citeSuggest.value ? pickCitation : pickTag;
  const close = wikiSuggest.value ? closeWikiSuggest : citeSuggest.value ? closeCiteSuggest : closeTagSuggest;
  if (event.key === 'ArrowDown') { s.active = (s.active + 1) % s.items.length; return true; }
  if (event.key === 'ArrowUp') { s.active = (s.active - 1 + s.items.length) % s.items.length; return true; }
  if (event.key === 'Enter' || event.key === 'Tab') { pick(s.items[s.active]); return true; }
  if (event.key === 'Escape') { close(); return true; }
  return false;
};

const createEditor = () => {
  editor.value = new Editor({
    content: props.modelValue,
    editable: !props.readOnly,
    extensions: buildExtensions(),
    editorProps: {
      attributes: {
        class: 'ProseMirror focus:outline-none',
        'aria-label': t('editor.contentAriaLabel'),
      },
      // Klick auf einen [[Wikilink]] -> Titel nach außen melden; Klick auf
      // einen relativen Link (eingefügtes PDF) -> Anhang öffnen.
      handleClickOn: (view, pos, node, nodePos, event) => {
        closeLinkBubble();
        if (node?.type?.name === 'wikilink') {
          emit('wikilink', node.attrs.title);
          return true;
        }
        if (node?.isText) {
          const link = node.marks?.find((m) => m.type.name === 'link');
          const href = link?.attrs?.href;
          if (href) {
            const isRelative = !/^[a-z][a-z0-9+.-]*:/i.test(href) && !href.startsWith('/');
            // Relativer Pfad = eingebetteter Anhang (nur mit assetResolver).
            if (isRelative) {
              if (!props.assetResolver) return false;
              emit('asset-open', href);
              return true;
            }
            // Weblink: im Lesemodus und bei Cmd/Strg direkt öffnen, sonst die
            // Blase zeigen (der Klick muss beim Bearbeiten den Cursor setzen).
            if (props.readOnly || event?.metaKey || event?.ctrlKey) {
              openExternal(href);
              return true;
            }
            const coords = view.coordsAtPos(pos);
            const rect = rootEl.value?.getBoundingClientRect();
            linkBubble.value = {
              href,
              top: rect ? coords.bottom - rect.top + 4 : 0,
              left: rect ? Math.max(8, coords.left - rect.left) : 0,
            };
            return true;
          }
        }
        return false;
      },
      handleKeyDown: (view, event) => handleSuggestKey(event),
      // Bild aus der Zwischenablage/per Drag einfügen -> Upload beim Parent.
      handlePaste: (view, event) => {
        if (!props.assetResolver || props.readOnly) return false;
        const file = Array.from(event.clipboardData?.files ?? []).find((f) => f.type.startsWith('image/'));
        if (!file) return false;
        emit('paste-image', file);
        return true;
      },
    },
    onUpdate: ({ editor }) => {
      const md = editor.storage.markdown.getMarkdown();
      lastEmitted = md;
      rawMarkdown.value = md;
      emit('update:modelValue', md);
      closeLinkBubble();
      checkWikiSuggest();
      checkCiteSuggest();
      checkTagSuggest();
    },
    onSelectionUpdate: () => { checkWikiSuggest(); checkCiteSuggest(); checkTagSuggest(); },
    onBlur: () => setTimeout(() => { closeWikiSuggest(); closeCiteSuggest(); closeTagSuggest(); }, 200),
  });
};
createEditor();

// Externe Änderung (andere Notiz gewählt) -> Inhalt neu setzen.
watch(() => props.modelValue, (val) => {
  if (val === lastEmitted) return;
  lastEmitted = val;
  rawMarkdown.value = val;
  if (editor.value && !showSource.value) {
    editor.value.commands.setContent(val || '', { emitUpdate: false });
  }
});

watch(() => props.readOnly, (ro) => {
  editor.value?.setEditable(!ro);
});

onBeforeUnmount(() => editor.value?.destroy());

// --- Mobile: Toolbar nur bei offener Tastatur, direkt darüber ---
// Auf dem Handy blendet die Toolbar erst ein, wenn der Editor den Fokus
// hat und die Tastatur offen ist; sie klebt via VisualViewport genau über
// der Tastatur und scrollt in einer Zeile horizontal.
const editorFocused = ref(false);
const kbOpen = ref(false);   // Tastatur offen (Toolbar sichtbar + fixiert)
const kbTop = ref(0);        // Top-Position (Unterkante des sichtbaren Bereichs)
const toolbarEl = ref(null);
const isMobileVp = () => window.matchMedia('(max-width: 767px)').matches;
const updateKeyboard = () => {
  // DAS PROGRAMM AUF ANDROID MELDET DIE TASTATUR SELBST (window.openanyRaender,
  // app/src/raender.js): Seine Webansicht verkleinert visualViewport nicht,
  // die Leiste bliebe sonst fuer immer verborgen. Im Browser gibt es das
  // Objekt nicht, dann gilt der Weg darunter.
  const bruecke = window.openanyRaender;
  if (bruecke?.tastatur) {
    const hoehe = isMobileVp() && editorFocused.value ? bruecke.tastatur() : 0;
    kbOpen.value = hoehe >= 80;
    if (kbOpen.value) {
      const barH = toolbarEl.value?.offsetHeight || 44;
      kbTop.value = Math.max(0, window.innerHeight - hoehe - barH);
    }
    return;
  }
  const vv = window.visualViewport;
  if (!isMobileVp() || !editorFocused.value || !vv) { kbOpen.value = editorFocused.value && !vv; return; }
  // Höhe der Tastatur = Layout-Unterkante minus Unterkante des sichtbaren
  // Fensters. Ist der Abstand klein, ist die Tastatur zu (auch wenn der
  // Editor noch fokussiert ist) → Leiste ausblenden.
  const gap = window.innerHeight - (vv.offsetTop + vv.height);
  if (gap < 80) { kbOpen.value = false; return; }
  kbOpen.value = true;
  // Die Leiste an die Unterkante des sichtbaren Bereichs pinnen – darüber
  // sitzt genau die Tastatur.
  const barH = toolbarEl.value?.offsetHeight || 44;
  kbTop.value = Math.max(0, vv.offsetTop + vv.height - barH);
};
const onFocusIn = () => {
  editorFocused.value = true;
  updateKeyboard();
  // Nach dem Tastatur-Aufklappen erneut messen (Höhe steht dann fest).
  setTimeout(updateKeyboard, 300);
};
const onFocusOut = () => {
  // Kurze Verzögerung: ein Tipp auf einen Toolbar-Button blurrt den Editor
  // nur ganz kurz – so bleibt die Leiste stehen, bis der Fokus wirklich geht.
  setTimeout(() => {
    if (rootEl.value && !rootEl.value.contains(document.activeElement)) {
      editorFocused.value = false;
      kbOpen.value = false;
    }
  }, 250);
};
onMounted(() => {
  rootEl.value?.addEventListener('focusin', onFocusIn);
  rootEl.value?.addEventListener('focusout', onFocusOut);
  window.visualViewport?.addEventListener('resize', updateKeyboard);
  window.addEventListener('openany-raender', updateKeyboard);
  window.visualViewport?.addEventListener('scroll', updateKeyboard);
});
onBeforeUnmount(() => {
  rootEl.value?.removeEventListener('focusin', onFocusIn);
  rootEl.value?.removeEventListener('focusout', onFocusOut);
  window.visualViewport?.removeEventListener('resize', updateKeyboard);
  window.removeEventListener('openany-raender', updateKeyboard);
  window.visualViewport?.removeEventListener('scroll', updateKeyboard);
});

// Umschalten WYSIWYG <-> Rohtext (verlustfrei über Markdown).
const toggleSource = () => {
  if (!showSource.value) {
    rawMarkdown.value = editor.value.storage.markdown.getMarkdown();
    showSource.value = true;
  } else {
    editor.value.commands.setContent(rawMarkdown.value || '', { emitUpdate: false });
    lastEmitted = rawMarkdown.value;
    emit('update:modelValue', rawMarkdown.value);
    showSource.value = false;
  }
};

const onRawInput = () => {
  lastEmitted = rawMarkdown.value;
  emit('update:modelValue', rawMarkdown.value);
};

// --- Toolbar-Helfer ---
const isActive = (name, attrs) => editor.value?.isActive(name, attrs) ?? false;
const chain = () => editor.value.chain().focus();

const setHeading = (level) => chain().toggleHeading({ level }).run();
const toggleBold = () => chain().toggleBold().run();
const toggleItalic = () => chain().toggleItalic().run();
const toggleStrike = () => chain().toggleStrike().run();
const toggleBullet = () => chain().toggleBulletList().run();
const toggleOrdered = () => chain().toggleOrderedList().run();
const toggleTask = () => chain().toggleTaskList().run();
const toggleQuote = () => chain().toggleBlockquote().run();
const toggleCodeBlock = () => chain().toggleCodeBlock().run();
const toggleInlineCode = () => chain().toggleCode().run();
const setHr = () => chain().setHorizontalRule().run();
const insertTable = () => chain().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
const undo = () => chain().undo().run();
const redo = () => chain().redo().run();

// [[ einfügen: öffnet direkt das Autocomplete (checkWikiSuggest via onUpdate).
const startWikilink = () => {
  chain().insertContent('[[').run();
};
// # einfügen: Nutzer tippt den Tag dahinter; ggf. Leerzeichen davor, damit
// das Tag-Muster (nur nach Whitespace/Zeilenanfang) sicher greift.
const startTag = () => {
  const { state } = editor.value;
  const { $from } = state.selection;
  const before = $from.parent.textBetween(Math.max(0, $from.parentOffset - 1), $from.parentOffset, null, '￼');
  chain().insertContent(before && before !== ' ' ? ' #' : '#').run();
};

// --- Link einfügen/bearbeiten ---
//
// Der Vorgänger rief `chain().extendMarkSelection?.()` auf. Diesen Befehl
// gibt es in tiptap nicht – er heisst `extendMarkRange`. Weil der optionale
// Aufruf damit `undefined` ergab, lief das folgende `.setLink()` auf
// `undefined` und warf; der Link landete NIE im Text. Der Fehler blieb
// unbemerkt, weil `?.` so aussieht, als sei der Fall bedacht.
const linkModal = ref(null); // { href, text, editing }

const openLinkModal = () => {
  const { state } = editor.value;
  const { from, to, empty } = state.selection;

  linkModal.value = {
    href: editor.value.getAttributes('link').href || '',
    // Markierten Text als Anzeigetext vorschlagen.
    text: empty ? '' : state.doc.textBetween(from, to, ' '),
    editing: editor.value.isActive('link'),
  };
};

const closeLinkModal = () => { linkModal.value = null; };

const applyLink = ({ href, text }) => {
  const war = linkModal.value;
  closeLinkModal();

  const { empty } = editor.value.state.selection;

  // Cursor steht in einem Link, ohne dass etwas markiert ist: erst die ganze
  // Markierung greifen, damit nicht nur das Zeichen unter dem Cursor
  // geaendert wird.
  if (empty && war?.editing) {
    chain().extendMarkRange('link').setLink({ href }).run();
    return;
  }

  // Nichts markiert und kein bestehender Link: Es gibt keinen Text, auf dem
  // eine Markierung sitzen koennte. Genau hier lief der alte Weg ins Leere –
  // er setzte eine Markierung ohne Text, und sichtbar wurde nichts. Also
  // Text einfuegen, notfalls die Adresse selbst.
  if (empty) {
    chain().insertContent([
      { type: 'text', text: text || href, marks: [{ type: 'link', attrs: { href } }] },
    ]).run();
    return;
  }

  // Etwas markiert: Mit Anzeigetext wird die Markierung ersetzt, ohne bleibt
  // sie stehen und bekommt nur den Link.
  if (text && text !== war?.text) {
    chain().insertContent([
      { type: 'text', text, marks: [{ type: 'link', attrs: { href } }] },
    ]).run();
  } else {
    chain().setLink({ href }).run();
  }
};

const removeLink = () => {
  closeLinkModal();
  chain().extendMarkRange('link').unsetLink().run();
};

const statusText = computed(() => {
  if (props.saveStatus === 'error') return props.saveError || t('editor.saveError');
  return { saving: t('editor.saving'), saved: t('editor.saved') }[props.saveStatus] ?? '';
});

// [@ einfügen: öffnet direkt das Zitier-Autocomplete.
const startCitation = () => {
  chain().insertContent('[@').run();
};

// Vom Parent nach erfolgreichem Upload aufgerufen: Bild-Node bzw.
// Markdown-Link mit dem relativen Pfad einfügen.
const insertImageAsset = (path) => {
  chain().setImage({ src: path }).run();
};
const insertFileAsset = (path, name) => {
  chain().insertContent([
    { type: 'text', text: name || path, marks: [{ type: 'link', attrs: { href: path } }] },
    { type: 'text', text: ' ' },
  ]).run();
};

// Undo/Redo für die Kopfzeile der Notiz-Kachel (Notes.vue).
defineExpose({ undo, redo, insertImageAsset, insertFileAsset });
</script>

<template>
  <div ref="rootEl" class="md-editor relative flex flex-col h-full">
    <!-- Toolbar (nur editierbar). Mobil: nur bei offener Tastatur sichtbar,
         fix direkt über der Tastatur, einzeilig & horizontal scrollbar.
         @mousedown.prevent hält den Editor-Fokus (Tastatur bleibt offen). -->
    <div v-if="!readOnly" ref="toolbarEl" @mousedown.prevent
      :style="kbOpen ? { top: kbTop + 'px', bottom: 'auto' } : null"
      :class="[
        'items-center gap-0.5 px-2 py-1.5 border-b border-linie',
        kbOpen
          ? 'flex fixed left-0 right-0 z-[70] shadow-lg bg-flaeche'
          : 'hidden md:flex md:flex-wrap bg-slate-50/60 dark:bg-slate-950/40'
      ]">
      <!-- NUR die Formatier-Knöpfe scrollen. Der Speicher-Status stand vorher
           mit im Streifen und damit an dessen Ende – auf dem Handy musste man
           erst waagerecht durchscrollen, um zu sehen, ob gespeichert ist.
           Genau die Auskunft, die man ohne Suchen braucht.

           `contents` am Desktop: Der Behälter verschwindet aus dem Layout,
           die Knöpfe fließen und umbrechen wie bisher. -->
      <div :class="kbOpen ? 'flex items-center gap-0.5 flex-nowrap overflow-x-auto flex-1 min-w-0' : 'contents'">

      <!-- Link steht bewusst GANZ VORN und farblich abgesetzt. Auf dem Handy
           laeuft die Leiste waagerecht und wird abgeschnitten; alles hinter
           den ersten paar Knoepfen erreicht man nur durch Scrollen. Der
           haeufigste Handgriff gehoert deshalb an die erste Stelle. -->
      <button type="button" :title="t('editor.link')" :aria-label="t('editor.link')" @click="openLinkModal()"
        class="tb tb-accent" :class="{ 'tb-active': isActive('link') }"><LinkIcon class="w-4 h-4" /></button>

      <span class="tb-sep"></span>

      <button v-for="btn in [
          { fn: () => setHeading(1), icon: Heading1, name: 'heading', attrs: { level: 1 }, label: t('editor.heading1') },
          { fn: () => setHeading(2), icon: Heading2, name: 'heading', attrs: { level: 2 }, label: t('editor.heading2') },
          { fn: () => setHeading(3), icon: Heading3, name: 'heading', attrs: { level: 3 }, label: t('editor.heading3') },
        ]" :key="btn.label" type="button" :title="btn.label" :aria-label="btn.label" @click="btn.fn()"
        class="tb" :class="{ 'tb-active': isActive(btn.name, btn.attrs) }"><component :is="btn.icon" class="w-4 h-4" /></button>

      <span class="tb-sep"></span>

      <button type="button" :title="t('editor.bold')" :aria-label="t('editor.bold')" @click="toggleBold()" class="tb" :class="{ 'tb-active': isActive('bold') }"><Bold class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.italic')" :aria-label="t('editor.italic')" @click="toggleItalic()" class="tb" :class="{ 'tb-active': isActive('italic') }"><Italic class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.strikethrough')" :aria-label="t('editor.strikethrough')" @click="toggleStrike()" class="tb" :class="{ 'tb-active': isActive('strike') }"><Strikethrough class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.inlineCode')" :aria-label="t('editor.inlineCode')" @click="toggleInlineCode()" class="tb" :class="{ 'tb-active': isActive('code') }"><Code class="w-4 h-4" /></button>

      <span class="tb-sep"></span>

      <button type="button" :title="t('editor.list')" :aria-label="t('editor.bulletList')" @click="toggleBullet()" class="tb" :class="{ 'tb-active': isActive('bulletList') }"><List class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.numberedList')" :aria-label="t('editor.orderedList')" @click="toggleOrdered()" class="tb" :class="{ 'tb-active': isActive('orderedList') }"><ListOrdered class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.checklist')" :aria-label="t('editor.checklist')" @click="toggleTask()" class="tb" :class="{ 'tb-active': isActive('taskList') }"><ListChecks class="w-4 h-4" /></button>

      <span class="tb-sep"></span>

      <button type="button" :title="t('editor.quote')" :aria-label="t('editor.blockquote')" @click="toggleQuote()" class="tb" :class="{ 'tb-active': isActive('blockquote') }"><Quote class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.codeBlock')" :aria-label="t('editor.codeBlock')" @click="toggleCodeBlock()" class="tb" :class="{ 'tb-active': isActive('codeBlock') }"><Code2 class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.divider')" :aria-label="t('editor.horizontalLine')" @click="setHr()" class="tb"><Minus class="w-4 h-4" /></button>
      <button type="button" :title="t('editor.table')" :aria-label="t('editor.insertTable')" @click="insertTable()" class="tb"><TableIcon class="w-4 h-4" /></button>

      <template v-if="wikilinks">
        <span class="tb-sep"></span>
        <button type="button" :title="t('editor.wikilink')" :aria-label="t('editor.wikilink')" @click="startWikilink()" class="tb"><Brackets class="w-4 h-4" /></button>
        <button type="button" :title="t('editor.tag')" :aria-label="t('editor.tag')" @click="startTag()" class="tb"><Hash class="w-4 h-4" /></button>
      </template>

      <template v-if="assetResolver">
        <span class="tb-sep"></span>
        <button v-if="einfuegbar" type="button" :title="t('editor.insert')" :aria-label="t('editor.insert')" @click="emit('insert')" class="tb"><Paperclip class="w-4 h-4" /></button>
        <button v-if="citations.length" type="button" :title="t('editor.insertCitation')" :aria-label="t('editor.insertCitation')" @click="startCitation()" class="tb"><BookMarked class="w-4 h-4" /></button>
      </template>

      </div>

      <!-- Status + Quelltext-Schalter: außerhalb des Scroll-Streifens und
           shrink-0, damit sie bei offener Tastatur immer am rechten Rand
           stehen bleiben, egal wie weit die Knöpfe gescrollt sind. -->
      <div class="ml-1 md:ml-auto flex items-center gap-2 shrink-0">
        <!-- Autosave-Status -->
        <span v-if="saveStatus === 'saving'" class="flex items-center gap-1 whitespace-nowrap text-xs font-medium text-marke"><Loader class="w-3 h-3 animate-spin" /> {{ statusText }}</span>
        <span v-else-if="saveStatus === 'saved'" class="flex items-center gap-1 whitespace-nowrap text-xs font-medium text-emerald-500"><Save class="w-3 h-3" /> {{ statusText }}</span>
        <button v-else-if="saveStatus === 'error'" type="button" @click="emit('retry')" class="flex items-center gap-1 whitespace-nowrap text-xs font-bold text-rose-500 hover:underline"><AlertCircle class="w-3 h-3" /> {{ statusText }} · {{ t('editor.retry') }}</button>

        <!-- WYSIWYG / Quelltext (Icon-only, damit die Toolbar die Kachel
             nicht sprengt; Beschriftung via title/aria-label) -->
        <button type="button" @click="toggleSource"
          :title="showSource ? t('editor.switchToEditor') : t('editor.showSource')"
          :aria-label="showSource ? t('editor.switchToEditor') : t('editor.showSource')"
          class="tb shrink-0" :class="{ 'tb-active': showSource }">
          <component :is="showSource ? Pencil : Code" class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Rohtext-Modus -->
    <textarea v-if="showSource && !readOnly" v-model="rawMarkdown" @input="onRawInput"
      spellcheck="false"
      class="flex-1 w-full p-6 bg-transparent border-none text-fliess resize-none focus:outline-none font-mono text-sm leading-relaxed"></textarea>

    <!-- WYSIWYG / Lese-Ansicht -->
    <div v-else class="flex-1 overflow-y-auto md-editor-content" :class="readOnly ? 'is-readonly' : ''">
      <EditorContent :editor="editor" class="h-full" />
    </div>

    <!-- [@-Zitier-Autocomplete-Dropdown -->
    <div v-if="citeSuggest" class="absolute z-50 w-80 max-h-56 overflow-y-auto karte shadow-xl py-1"
      :style="{ top: citeSuggest.top + 'px', left: citeSuggest.left + 'px' }">
      <button v-for="(item, i) in citeSuggest.items" :key="item.key" type="button"
        @mousedown.prevent="pickCitation(item)"
        class="w-full text-left px-3 py-1.5"
        :class="i === citeSuggest.active
          ? 'bg-auflage'
          : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'">
        <span class="block text-sm font-medium truncate text-schrift">{{ item.author || item.key }}<span v-if="item.year" class="text-slate-400"> ({{ item.year }})</span></span>
        <span class="block text-xs text-leise truncate">{{ item.title }}</span>
      </button>
    </div>

    <!-- #-Tag-Autocomplete-Dropdown -->
    <div v-if="tagSuggest" class="absolute z-50 w-56 max-h-56 overflow-y-auto karte shadow-xl py-1"
      :style="{ top: tagSuggest.top + 'px', left: tagSuggest.left + 'px' }">
      <button v-for="(item, i) in tagSuggest.items" :key="item.tag" type="button"
        @mousedown.prevent="pickTag(item)"
        class="w-full text-left px-3 py-1.5 text-sm font-medium flex items-center justify-between gap-2"
        :class="i === tagSuggest.active
          ? 'bg-auflage text-schrift'
          : 'text-fliess hover:bg-slate-50 dark:hover:bg-slate-800/60'">
        <span class="truncate">#{{ item.tag }}</span>
        <span class="text-xs text-slate-400 shrink-0">{{ item.count }}</span>
      </button>
    </div>

    <!-- [[-Autocomplete-Dropdown -->
    <div v-if="wikiSuggest" class="absolute z-50 w-64 max-h-56 overflow-y-auto karte shadow-xl py-1"
      :style="{ top: wikiSuggest.top + 'px', left: wikiSuggest.left + 'px' }">
      <button v-for="(item, i) in wikiSuggest.items" :key="item.id" type="button"
        @mousedown.prevent="pickWikilink(item)"
        class="w-full text-left px-3 py-1.5 text-sm font-medium truncate"
        :class="i === wikiSuggest.active
          ? 'bg-auflage text-schrift'
          : 'text-fliess hover:bg-slate-50 dark:hover:bg-slate-800/60'">
        {{ item.title }}
      </button>
    </div>

    <!-- Weblink angeklickt: Ziel zeigen und öffnen anbieten (auch auf Mobilgeräten) -->
    <div v-if="linkBubble" class="absolute z-50 max-w-[min(22rem,90%)] flex items-center gap-2 px-2.5 py-1.5 karte shadow-xl"
      :style="{ top: linkBubble.top + 'px', left: linkBubble.left + 'px' }">
      <span class="min-w-0 flex-1 truncate text-xs font-mono text-leise" :title="linkBubble.href">{{ linkBubble.href }}</span>
      <BaseButton type="button" @mousedown.prevent="openBubbleLink"
        class="shrink-0" groesse="klein">
        {{ t('editor.openLink') }}
      </BaseButton>
      <button type="button" @mousedown.prevent="closeLinkBubble"
        class="shrink-0 p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded-lg" :aria-label="t('common.close')">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>

    <LinkModal v-if="linkModal" v-bind="linkModal"
      @save="applyLink" @remove="removeLink" @close="closeLinkModal" />
  </div>
</template>

<style scoped>
.tb {
  width: 2rem; height: 2rem; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem; color: #64748b; cursor: pointer; transition: background-color .15s, color .15s;
}
.tb:hover { background: rgba(226, 232, 240, .7); }
:global(.dark) .tb { color: #94a3b8; }
:global(.dark) .tb:hover { background: #1e293b; }
/* Farben ueber die Palette-Variablen (style.css, @theme) statt fester Werte.
   Hier standen Tailwinds STANDARD-Indigos (#4f46e5, #e0e7ff, #4338ca) – die
   App faerbt Indigo aber auf Flieder um (#885c8f). Der Editor lief damit seit
   jeher in einer anderen Farbe als der Rest der Seite, was niemandem auffiel,
   weil man beides selten nebeneinander sieht. Mit var() kann das nicht wieder
   auseinanderlaufen: Wer die Palette aendert, aendert den Editor mit.

   Die fuenf `:global(.dark)`-Regeln, die hier standen, sind seit dem
   08.09.2026 weg: Die Rollen `--color-marke`, `-zart` und `-leise`
   kippen selbst mit der Ansicht, es gibt also nichts mehr nachzuziehen.

   .tb-accent MUSS VOR .tb-active stehen: Bei gleicher Spezifitaet gewinnt die
   spaetere Regel. Stuende der Akzent hinter .tb-active, bliebe der Link-Knopf
   auch dann im Ruhe-Ton, wenn der Cursor wirklich in einem Link steht. */
.tb-accent { color: var(--color-marke); background: var(--color-marke-leise); }
.tb-accent:hover { background: color-mix(in srgb, var(--color-marke) 14%, var(--color-marke-leise)); }
.tb-active { background: color-mix(in srgb, var(--color-marke) 26%, var(--color-marke-leise)); color: var(--color-marke); }
.tb-sep { width: 1px; height: 1.25rem; flex-shrink: 0; background: #e2e8f0; margin: 0 0.25rem; }
:global(.dark) .tb-sep { background: #334155; }
</style>

<!-- Inhaltsstyling (global, unter .md-editor-content genamespaced -> respektiert Dark Mode via .dark am <html>) -->
<style>
.md-editor-content .ProseMirror { padding: 1.5rem; min-height: 100%; outline: none; color: #334155; line-height: 1.7; }
.dark .md-editor-content .ProseMirror { color: #cbd5e1; }
.md-editor-content .ProseMirror > * + * { margin-top: 0.75em; }
.md-editor-content .ProseMirror h1 { font-size: 1.6rem; font-weight: 800; letter-spacing: -0.02em; color: #0f172a; }
.md-editor-content .ProseMirror h2 { font-size: 1.3rem; font-weight: 700; color: #0f172a; }
.md-editor-content .ProseMirror h3 { font-size: 1.1rem; font-weight: 700; color: #0f172a; }
.dark .md-editor-content .ProseMirror :is(h1,h2,h3) { color: #f1f5f9; }
.md-editor-content .ProseMirror ul { list-style: disc; padding-left: 1.5rem; }
.md-editor-content .ProseMirror ol { list-style: decimal; padding-left: 1.5rem; }
.md-editor-content .ProseMirror li > p { margin: 0; }
.md-editor-content .ProseMirror blockquote { border-left: 3px solid var(--color-marke); padding-left: 1rem; color: var(--color-leise); font-style: italic; }
.md-editor-content .ProseMirror code { background: #f1f5f9; color: #be185d; padding: 0.1em 0.35em; border-radius: 0.35rem; font-size: 0.9em; }
.dark .md-editor-content .ProseMirror code { background: #1e293b; color: #f9a8d4; }
.md-editor-content .ProseMirror pre { background: #0f172a; color: #e2e8f0; padding: 1rem; border-radius: 0.75rem; overflow-x: auto; }
.md-editor-content .ProseMirror pre code { background: none; color: inherit; padding: 0; }
/* cursor: pointer – ohne das zeigt der Browser ueber einem Link den
   Text-Cursor, und er sieht schlicht nicht anklickbar aus. Der [[Wikilink]]
   weiter unten hatte es von Anfang an, der gewoehnliche Weblink nicht. */
/* Unterstrichen erst beim Draufzeigen – im Fliesstext waere jede Zeile mit
   Links sonst unruhig. Erkennbar bleibt der Link auch ohne: Farbe UND
   font-weight: 600 zusammen, nie die Farbe allein. Das ist der Grund fuer
   das Fettschreiben; wer Farben schlecht unterscheidet, sieht sonst auf
   einem Handy nichts mehr, weil dort kein Zeigegeraet zum Draufhalten ist.
   text-underline-offset haelt die Linie von den Unterlaengen weg. */
.md-editor-content .ProseMirror a { color: var(--color-marke); text-decoration: none; cursor: pointer; font-weight: 600; }
.md-editor-content .ProseMirror a:hover,
.md-editor-content .ProseMirror a:focus-visible { text-decoration: underline; text-underline-offset: 0.15em; }
.md-editor-content .ProseMirror hr { border: none; border-top: 1px solid #e2e8f0; margin: 1.25em 0; }
.dark .md-editor-content .ProseMirror hr { border-top-color: #334155; }
.md-editor-content .ProseMirror ul[data-type="taskList"] { list-style: none; padding-left: 0.25rem; }
.md-editor-content .ProseMirror ul[data-type="taskList"] li { display: flex; align-items: flex-start; gap: 0.5rem; }
.md-editor-content .ProseMirror ul[data-type="taskList"] li > label { margin-top: 0.15rem; }
.md-editor-content .ProseMirror table { border-collapse: collapse; width: 100%; margin: 0.5em 0; overflow: hidden; }
.md-editor-content .ProseMirror :is(td, th) { border: 1px solid #e2e8f0; padding: 0.4rem 0.6rem; text-align: left; vertical-align: top; }
.md-editor-content .ProseMirror th { background: #f8fafc; font-weight: 700; }
.dark .md-editor-content .ProseMirror :is(td, th) { border-color: #334155; }
.dark .md-editor-content .ProseMirror th { background: #1e293b; }
.md-editor-content.is-readonly .ProseMirror { padding-top: 1.5rem; }
/* [[Wikilinks]]: bewusst unbunt – nur gestrichelt unterstrichen. */
.md-editor-content .ProseMirror .wikilink { text-decoration: underline; text-decoration-style: dashed; text-underline-offset: 3px; cursor: pointer; font-weight: 600; }
/* Eingefügte Bilder: nie breiter als die Notiz, dezent gerundet. */
.md-editor-content .ProseMirror img { max-width: 100%; height: auto; border-radius: 0.5rem; }
</style>
