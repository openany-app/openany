// Symbol und Farbe je Anzeige-Typ einer Datei. Der Typ kommt vom Server
// (FileNode::displayType()), die Zuordnung brauchen sowohl der persönliche
// Speicher (FileExplorer) als auch die eingebettete Projekt-Ansicht
// (ProjectFileView) – und beide müssen dieselbe Datei gleich darstellen.
import { Folder, FileText, Music, Video, File, Archive, Image as ImageIcon } from 'lucide-vue-next';

const ICONS = {
  pdf: FileText,
  image: ImageIcon,
  audio: Music,
  video: Video,
  text: FileText,
};

// Tailwind kennt nur die Stufen 50, 100–900 und 950. Hier standen einmal
// 450 und 655 – dafür entsteht keine CSS-Regel. Bei den dunklen Varianten
// blieb dadurch stillschweigend die helle Farbe stehen; bei `text` waren
// BEIDE Stufen ungültig, das Symbol erbte also die Textfarbe der Zeile.
const COLORS = {
  folder: 'text-marke',
  pdf: 'text-rose-600 dark:text-rose-400',
  image: 'text-sky-600 dark:text-sky-400',
  audio: 'text-emerald-600 dark:text-emerald-400',
  video: 'text-pink-600 dark:text-pink-400',
  text: 'text-amber-600 dark:text-amber-400',
};

// Ordner heißen im Dokumente-Modul „Akten" und tragen deshalb ein anderes
// Symbol; alle übrigen Typen sehen in beiden Zonen gleich aus.
export function fileIcon(type, isDocuments = false) {
  if (type === 'folder') return isDocuments ? Archive : Folder;
  return ICONS[type] ?? File;
}

export function fileIconColor(type) {
  return COLORS[type] ?? 'text-leise';
}
