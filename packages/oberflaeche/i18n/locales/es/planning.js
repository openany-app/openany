// Der Reiter „Board“ eines Hubs: das eine Board, dazu
// die Texte der Zeichenfläche (whiteboard) und zum Teilen von Kalendern.
export default {
  board: {
    titel: 'Board',
    hinweis: 'El board compartido de este hub: todos los miembros dibujan y escriben en él, a través de openany.de y en persona.',
    geaendert: 'Último cambio: {zeit}',
    oeffnen: 'Abrir el board',
  },
  kalender: {
    teilenHinweis: 'Con «ver», los miembros ven las citas de este calendario; con «editar», también pueden crearlas, cambiarlas y borrarlas. El calendario en sí solo lo cambias tú. Los calendarios suscritos solo se comparten en persona.',
    beenden: 'Dejar de compartir',
    beendenFrage: '¿Dejar de compartir «{name}» en este hub? Las citas desaparecen para todos los miembros; tu calendario se queda.',
  },
  whiteboard: {
    neu: 'Nueva pizarra',
    nurAnsehen: 'Solo ver: puede guardar quien creó la pizarra.',
    nurAnsehenFern: 'Solo ver: puedes dibujar cuando un dispositivo de {name} esté cerca.',
    speichert: 'Guardando …',
    gespeichert: 'Guardado',
    nichtGespeichert: 'No guardado: se reintentará con el próximo cambio.',
    veraltet: 'Otro de tus dispositivos ha guardado mientras tanto. La pizarra se vuelve a cargar.',
    offline: 'Sin conexión: ves la última versión cargada.',
    ladenFehler: 'No se pudo abrir la pizarra.',
    confirmDelete: '¿Eliminar la pizarra «{title}»?',
    loeschen: 'Eliminar pizarra',
    laedt: 'Cargando el lienzo …',
    einbinden: 'Insertar',
    alleFreigaben: 'Todo lo compartido',
    keineFreigaben: 'En este hub todavía no hay nada compartido. Puedes compartir en «Comparticiones».',
    leer: 'Aquí no hay nada que se pueda insertar.',
    nichtMehrGeteilt: 'Ya no está compartido',
    karteNotiz: 'Nota',
    kartePdf: 'PDF',
    karteDatei: 'Archivo',
  },
};
