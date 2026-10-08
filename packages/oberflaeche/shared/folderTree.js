// Reine Baum-Helfer für die flache Mappen-Liste ({ id, name, parent_id }) im
// Notizen-Modul: Kind-Mappen einer Ebene, Breadcrumb-Pfad und die eingerückte
// DFS-Liste für den Verschieben-Dialog. Bewusst frei von Vue/DOM, damit sie in
// der node-Umgebung testbar sind.

// Kind-Mappen einer Ebene (parentId = null => oberste Ebene), alphabetisch (de).
export function childFolders(folders, parentId) {
  return folders
    .filter((f) => (f.parent_id ?? null) === (parentId ?? null))
    .sort((a, b) => a.name.localeCompare(b.name, 'de'));
}

// Breadcrumb-Pfad [Home, …, aktuelle Mappe]. Bricht sauber ab, wenn ein
// Elternteil fehlt (verwaiste Mappe) oder die Kette einen Zyklus bildet – so
// kann fehlerhafte Server-Antwort keinen Endlos-Loop auslösen.
export function buildBreadcrumbs(folders, currentFolderId) {
  const byId = new Map(folders.map((f) => [f.id, f]));
  const chain = [];
  const seen = new Set();
  let id = currentFolderId;
  while (id != null && byId.has(id) && !seen.has(id)) {
    seen.add(id);
    const f = byId.get(id);
    chain.unshift({ id: f.id, name: f.name });
    id = f.parent_id;
  }
  return [{ id: null, name: 'Home' }, ...chain];
}

// Alle Mappen als eingerückte DFS-Liste (jeweils mit `depth`) für Auswahl-
// Dialoge; pro Ebene alphabetisch (über childFolders).
export function folderTreeOptions(folders) {
  const result = [];
  const walk = (parentId, depth) => {
    for (const f of childFolders(folders, parentId)) {
      result.push({ ...f, depth });
      walk(f.id, depth + 1);
    }
  };
  walk(null, 0);
  return result;
}
