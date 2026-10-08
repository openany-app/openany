// Vorgegebene Kalenderfarben (Auswahl beim Anlegen und im Farbwähler).
export const PRESET_COLORS = [
  { value: '#ef4444', label: 'Rot' },
  { value: '#f97316', label: 'Orange' },
  { value: '#d97706', label: 'Bernstein' },
  { value: '#eab308', label: 'Gelb' },
  { value: '#84cc16', label: 'Limette' },
  { value: '#22c55e', label: 'Grün' },
  { value: '#059669', label: 'Smaragd' },
  { value: '#14b8a6', label: 'Türkis' },
  { value: '#06b6d4', label: 'Cyan' },
  { value: '#0ea5e9', label: 'Himmelblau' },
  { value: '#3b82f6', label: 'Blau' },
  { value: '#4f46e5', label: 'Indigo' },
  { value: '#8b5cf6', label: 'Violett' },
  { value: '#a855f7', label: 'Lila' },
  { value: '#d946ef', label: 'Fuchsia' },
  { value: '#ec4899', label: 'Pink' },
  { value: '#f43f5e', label: 'Rose' },
  { value: '#64748b', label: 'Schiefer' },
];

/**
 * Vorbelegung für neue Kalender und Rückfall für Kalender ohne eigene Farbe.
 *
 * Bewusst ein Eintrag AUS der Liste statt eines eigenen Wertes: Vorher standen
 * zwei verschiedene Indigo-Töne fest im Quelltext – '#4f46e5' im Anlege-Dialog
 * und '#6366f1' auf der Startseite. Derselbe Kalender sah damit je nach Ansicht
 * unterschiedlich aus, und der Startseiten-Ton war in keiner Auswahl zu finden.
 */
export const DEFAULT_CALENDAR_COLOR = PRESET_COLORS.find((c) => c.label === 'Indigo').value;
