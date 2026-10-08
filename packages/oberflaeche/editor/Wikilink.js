import { Node, InputRule } from '@tiptap/core';

/**
 * [[Wikilink]] als Inline-Atom-Node (Obsidian-Stil).
 *
 * - Markdown-Quelle bleibt `[[Titel]]` bzw. `[[Titel|Anzeigetext]]`
 *   (Serialisierung + markdown-it-Rule, beides über tiptap-markdown eingebunden).
 * - Angezeigt wird der Anzeigetext (Alias) falls vorhanden, sonst der Titel;
 *   dezent unterstrichen (bewusst unbunt).
 * - Tippt man `[[Titel]]` zu Ende, wird der Text per InputRule zur Node.
 * - Klicks fängt MarkdownEditor über handleClickOn ab und meldet den TITEL
 *   nach außen (Navigation ist Sache des Notizen-Moduls, Alias irrelevant).
 */

// [[Titel]] oder [[Titel|Anzeigetext]] -> { title, alias }. Der Alias ist rein
// visuell; für Navigation/Auflösung zählt nur der Titel (vor dem ersten '|').
function splitWikilink(raw) {
  const idx = raw.indexOf('|');
  if (idx === -1) return { title: raw.trim(), alias: '' };
  return { title: raw.slice(0, idx).trim(), alias: raw.slice(idx + 1).trim() };
}

export default Node.create({
  name: 'wikilink',
  inline: true,
  group: 'inline',
  atom: true,
  selectable: true,

  addAttributes() {
    return { title: { default: '' }, alias: { default: '' } };
  },

  parseHTML() {
    return [{
      tag: 'span[data-wikilink]',
      getAttrs: (el) => ({
        title: el.getAttribute('data-wikilink') || '',
        alias: el.getAttribute('data-wikilink-alias') || '',
      }),
    }];
  },

  renderHTML({ node }) {
    const label = node.attrs.alias || node.attrs.title;
    const attrs = { 'data-wikilink': node.attrs.title, class: 'wikilink' };
    if (node.attrs.alias) attrs['data-wikilink-alias'] = node.attrs.alias;
    return ['span', attrs, label];
  },

  renderText({ node }) {
    return node.attrs.alias
      ? `[[${node.attrs.title}|${node.attrs.alias}]]`
      : `[[${node.attrs.title}]]`;
  },

  addInputRules() {
    return [
      new InputRule({
        find: /\[\[([^[\]\n]+)\]\]$/,
        handler: ({ range, match, chain }) => {
          const { title, alias } = splitWikilink(match[1]);
          if (!title) return;
          chain().insertContentAt(range, [{ type: this.name, attrs: { title, alias } }]).run();
        },
      }),
    ];
  },

  addStorage() {
    return {
      markdown: {
        serialize(state, node) {
          const raw = node.attrs.alias
            ? `${node.attrs.title}|${node.attrs.alias}`
            : node.attrs.title;
          state.write(`[[${raw}]]`);
        },
        parse: {
          setup(markdownit) {
            // Inline-Rule VOR 'link': [[Titel|Anzeige]] -> <span data-wikilink>.
            markdownit.inline.ruler.before('link', 'wikilink', (state, silent) => {
              const src = state.src;
              const pos = state.pos;
              if (src.charCodeAt(pos) !== 0x5B /* [ */ || src.charCodeAt(pos + 1) !== 0x5B) return false;
              const end = src.indexOf(']]', pos + 2);
              if (end === -1) return false;
              const raw = src.slice(pos + 2, end);
              if (!raw.trim() || /[[\]\n]/.test(raw)) return false;
              if (!silent) {
                const token = state.push('wikilink', '', 0);
                token.content = raw.trim();
              }
              state.pos = end + 2;
              return true;
            });
            markdownit.renderer.rules.wikilink = (tokens, idx) => {
              const { title, alias } = splitWikilink(tokens[idx].content);
              const esc = markdownit.utils.escapeHtml;
              return alias
                ? `<span data-wikilink="${esc(title)}" data-wikilink-alias="${esc(alias)}"></span>`
                : `<span data-wikilink="${esc(title)}"></span>`;
            };
          },
        },
      },
    };
  },
});
