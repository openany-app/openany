/*
 * Excalidraw ist eine React-Komponente (docs/plan-whiteboard.md). Diese Datei
 * hängt sie in ein Feld der Vue-Ansicht und wird -- samt React -- erst
 * geladen, wenn man ein Whiteboard öffnet (`import()` in
 * WhiteboardAnsicht.vue). Vorher muss `window.EXCALIDRAW_ASSET_PATH` stehen,
 * damit die Schriften aus der App kommen (vitePlugin.js).
 */
import { createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { Excalidraw, MainMenu, convertToExcalidrawElements } from '@excalidraw/excalidraw';
import '@excalidraw/excalidraw/index.css';

// Ein eigenes Menü: Das vorgegebene verweist auf GitHub, X und Discord und
// bietet Laden/Speichern von Dateien -- beides gehört nicht hierher.
const menue = () => createElement(
  MainMenu,
  null,
  createElement(MainMenu.DefaultItems.SaveAsImage),
  createElement(MainMenu.DefaultItems.ChangeCanvasBackground),
  createElement(MainMenu.DefaultItems.ClearCanvas),
);

/**
 * @param {HTMLElement} feld
 * @param {object} eigenschaften  Props für <Excalidraw>
 * @returns {{ setzen(neu: object): void, abbauen(): void, bauen(gerippe: object[]): object[] }}
 */
export function anbringen(feld, eigenschaften) {
  const wurzel = createRoot(feld);
  let jetzt = eigenschaften;
  const zeichnen = () => wurzel.render(createElement(Excalidraw, jetzt, menue()));
  zeichnen();
  return {
    setzen(neu) {
      jetzt = { ...jetzt, ...neu };
      zeichnen();
    },
    abbauen() {
      wurzel.unmount();
    },
    // Aus einem Gerippe ({ type, x, y, label, … }) fertige Elemente machen.
    bauen: (gerippe) => convertToExcalidrawElements(gerippe),
  };
}
