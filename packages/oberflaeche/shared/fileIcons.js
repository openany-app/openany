// Symbol und Farbe je Anzeige-Typ einer Datei. Der Typ kommt vom Server
// (FileNode::displayType()); dieselbe Zuordnung überall, damit eine Datei
// überall gleich aussieht.
import { Folder, FileText, Music, Video, File, Archive, Image as ImageIcon } from 'lucide-vue-next';

const ICONS = {
  pdf: FileText,
  image: ImageIcon,
  audio: Music,
  video: Video,
  text: FileText,
};

// Tailwind kennt nur die Stufen 50, 100–900 und 950 -- für eine andere
// Stufe entsteht keine CSS-Regel, und das Symbol erbte still eine andere
// Farbe.
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
