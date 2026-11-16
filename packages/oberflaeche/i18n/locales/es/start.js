// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privado, cifrado, primero en tu dispositivo',
  kopf: 'Tus notas, citas, archivos y mensajes, en un solo lugar.',
  unterzeile: 'Privado, cifrado de extremo a extremo y primero en tu dispositivo. Gratis, sin publicidad, sin contraseña.',
  ladenFuer: 'Descargar para {system}',
  alleDownloads: 'Todas las descargas',
  iosFolgt: 'openany para iPhone y iPad está en camino.',
  merkmale: {
    verschluesselt: {
      titel: 'Solo tú puedes leerlo',
      text: 'Todo se cifra en tu dispositivo antes de llegar al servidor. No podemos leer tus contenidos, ni aunque quisiéramos.',
    },
    geraet: {
      titel: 'Primero en tu dispositivo',
      text: 'openany funciona sin internet y sin cuenta. La sincronización a través de openany.de es una opción, no una obligación.',
    },
    alles: {
      titel: 'Todo en un solo lugar',
      text: 'Notas, calendario, contactos, archivos, fotos, vídeos y mensajes, en una app en lugar de diez.',
    },
    hubs: {
      titel: 'Juntos en hubs',
      text: 'Comparte carpetas, calendarios y álbumes con otras personas, dibujad juntos en un tablero y chatead.',
    },
    sparsam: {
      titel: 'Sin contraseña ni correo electrónico',
      text: 'Creas tu cuenta directamente en la app, y está ligada a tus dispositivos. No guardamos direcciones de correo ni direcciones IP.',
    },
    frei: {
      titel: 'Gratis y sin publicidad',
      text: 'Sin suscripción, sin anuncios, sin venta de datos. Cada cuenta empieza con 100 MB de almacenamiento en openany.de.',
    },
  },
  downloadsTitel: 'openany para tu dispositivo',
  version: 'Versión {version}',
  empfohlen: 'Para tu dispositivo',
  beiGithub: 'Los archivos están alojados en GitHub.',
  keineListe: 'La lista actual no se puede cargar en este momento.',
  dateien: {
    apk: 'Descargar APK',
    installer: 'Instalador (.exe)',
    msi: 'Paquete MSI',
    dmg: 'Descargar para Mac (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian y Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Directamente como archivo, no en Play Store. La primera vez, Android pregunta si quieres permitir apps de esta fuente.',
    windows: 'Para Windows 10 y 11. La app aún no está firmada: si Windows muestra un aviso, elige «Más información» y luego «Ejecutar de todas formas».',
    macos: 'Desde macOS 13.3, para chip de Apple e Intel. La app aún no está notarizada por Apple: en el primer inicio, permítela en Ajustes del Sistema, Privacidad y seguridad, «Abrir igualmente».',
    linux: 'AppImage para casi cualquier distribución, .deb para Debian y Ubuntu. 64 bits.',
  },
  fuss: {
    melden: 'Denunciar y contacto',
    agb: 'Condiciones de uso',
    datenschutz: 'Privacidad',
    impressum: 'Aviso legal',
    github: 'Todas las versiones en GitHub',
  },
  recht: {
    zurueck: 'Volver al inicio',
    nurDeutsch: 'Este texto solo es válido en su versión alemana.',
    fehler: 'El texto no se puede cargar en este momento. Inténtalo más tarde.',
  },
  plattformen: 'Para Android, Windows, macOS y Linux. iPhone y iPad están en camino.',
  kopfAlt: [
    'Pantalla de inicio de la app con hoy, agenda, notas, almacenamiento y hubs',
    'Álbum de la mediateca con paisajes',
  ],
  rundgangKnopf: 'Ver el recorrido',
  rundgangLabel: 'Recorrido',
  sprung: {
    notizen: 'Notas',
    kalender: 'Calendario',
    hubs: 'Hubs',
    nachrichten: 'Mensajes',
    mediathek: 'Mediateca',
    dokumente: 'Documentos',
    downloads: 'Descargas',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notas',
      titel: 'Escribir y conectar tus ideas',
      abschnitte: [
        {
          titel: 'Escribir, ordenar, conectar',
          absaetze: [
            'Las notas se guardan en carpetas y se escriben en Markdown, con títulos, listas y tareas para marcar.',
          ],
          punkte: [
            'Los `[[wikienlaces]]` conectan unas notas con otras',
            'Las `#etiquetas` reúnen lo que va junto',
            'La búsqueda de texto completo encuentra cada palabra',
          ],
          alt: [
            'Nota con lista, wikienlaces, etiquetas y tareas',
          ],
        },
        {
          titel: 'Ver cómo se conecta todo',
          absaetze: [
            'La vista «Conexiones» muestra tus notas como una red. Elige una carpeta o una etiqueta y aparecen las notas correspondientes.',
          ],
          alt: [
            'Lista de notas con carpetas y notas, nombres difuminados',
            'Grafo de conexiones entre notas, etiquetas difuminadas',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Calendario',
      titel: 'Citas para ti y para quienes planifican contigo',
      abschnitte: [
        {
          titel: 'Tu semana de un vistazo',
          absaetze: [
            'Mes, semana o día: citas con lugar, notas y repetición. Basta con tocar el día.',
          ],
          punkte: [
            'Varios calendarios a la vez',
            '«Hoy» directamente en la pantalla de inicio',
            'Cifrado como todo lo demás',
            'Repeticiones como «cada 2 semanas los martes, 5 veces»',
          ],
          alt: [
            'Vista mensual con citas de ejemplo',
          ],
        },
        {
          titel: 'Compartir calendarios en hubs',
          absaetze: [
            'Con «ver», los miembros ven tus citas; con «editar», añaden las suyas. Solo tú cambias el calendario en sí.',
          ],
          alt: [
            'Diálogo para compartir un calendario en un hub',
            'Hub con carpeta, calendario, álbum y expediente compartidos',
          ],
        },
        {
          titel: 'En el calendario y los contactos de tu dispositivo',
          absaetze: [
            'En Android, tus calendarios y contactos de openany aparecen en las apps del teléfono, y lo que cambies allí vuelve. En el ordenador, openany los ofrece a los programas de calendario y contactos, solo en ese equipo.',
          ],
          punkte: [
            'Recordatorios de eventos en el teléfono',
            'En el ordenador mediante CalDAV y CardDAV, por ejemplo para Thunderbird',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Hubs',
      titel: 'Juntos, no uno al lado del otro',
      abschnitte: [
        {
          titel: 'Un lugar para tu grupo',
          absaetze: [
            'Un hub es de una asociación, un piso compartido, una familia o un proyecto. Invitas a los miembros por su nombre de openany. El nombre y el chat están cifrados de extremo a extremo.',
          ],
          punkte: [
            'Compartir carpetas, calendarios, álbumes y expedientes',
            'En cada uso compartido: ver o editar',
          ],
          alt: [
            'Lista de miembros de un hub con búsqueda para invitar',
          ],
        },
        {
          titel: 'Un board para todos',
          absaetze: [
            'Cada hub tiene un board común. Todos los miembros dibujan y escriben en él, a través de openany.de o directamente cerca, en la misma red.',
          ],
          punkte: [
            'Formas, flechas, texto y lápiz',
            'Insertar imágenes',
            'Los cambios simultáneos se combinan',
          ],
          alt: [
            'Board con un plano de bancales dibujado',
          ],
        },
        {
          titel: 'Coordinarse en el chat del hub',
          absaetze: [
            'Cada hub tiene su propio chat. Con `[[` enlazas contenido del hub directamente en un mensaje.',
          ],
          punkte: [
            'Cifrado de extremo a extremo',
            'Fijar mensajes importantes',
            'Avisar a los miembros si quieres',
          ],
          alt: [
            'Chat del hub con tres mensajes de ejemplo',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Mensajes',
      titel: 'Todos los canales en una sola lista',
      abschnitte: [
        {
          titel: 'Mensajes directos, cifrados',
          absaetze: [
            'Los mensajes entre cuentas de openany están cifrados de extremo a extremo, incluidos los adjuntos. Se suman Matrix y el correo, todo en una sola lista.',
          ],
          punkte: [
            'Filtros: no leídos, con adjunto, openany, cerca',
            'Cerca funciona incluso sin servidor',
            'Por conversación o por contacto',
          ],
          alt: [
            'Lista de mensajes con búsqueda y filtros, en parte difuminada',
          ],
        },
        {
          titel: 'Matrix: llegar también a quien no usa openany',
          absaetze: [
            'Conecta una cuenta de matrix.org o de otro servidor. Tu dispositivo inicia sesión por sí mismo, con sus propias claves.',
          ],
          punkte: [
            'openany.de no lee nada',
            'El historial se queda en el dispositivo',
            'La contraseña no se guarda',
          ],
          alt: [
            'Ajustes: conectar una cuenta de Matrix',
          ],
        },
        {
          titel: 'Correo, con PGP si quieres',
          absaetze: [
            'Tu buzón de siempre se convierte en el tercer canal de la lista. El dispositivo habla él mismo con el servidor de correo.',
            'Las claves OpenPGP públicas de tus contactos llegan con sus correos; con ellas cifras tus respuestas.',
          ],
          alt: [
            'Ajustes: conectar un buzón de correo',
            'Ajustes: claves de los contactos',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Mediateca',
      titel: 'Tus imágenes, vídeos y grabaciones',
      abschnitte: [
        {
          titel: 'Fotos, vídeos y audio en álbumes',
          absaetze: [
            'Álbumes con subálbumes, llenados directamente desde la app: con cámara, vídeo o micrófono. Los álbumes se comparten en hubs.',
          ],
          punkte: [
            'Las miniaturas se crean en el dispositivo',
            'Los datos de ubicación se conservan si quieres',
            'Guardado cifrado',
          ],
          alt: [
            'Álbum de la mediateca con paisajes',
          ],
        },
        {
          titel: 'Muchos archivos a la vez',
          absaetze: [
            'Añade de una vez todas las fotos y vídeos de tu teléfono, como copia o sin ella: entonces se quedan en el teléfono y van cifrados a openany.de.',
          ],
          punkte: [
            'Fotos nuevas automáticamente, si quieres',
            'Compartir con openany desde otras apps',
            'En el ordenador, arrastrar archivos y carpetas a la ventana o sincronizar una carpeta',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Documentos',
      titel: 'Papeles en los que puedes buscar',
      abschnitte: [
        {
          titel: 'Los papeles en expedientes',
          absaetze: [
            'Cartas, contratos y documentos se guardan en expedientes. Los PDF se abren directamente en openany: hojear, buscar, rellenar formularios, resaltar y comentar.',
          ],
          punkte: [
            'Búsqueda también dentro de PDF y archivos de oficina',
            'Compartir expedientes en hubs',
          ],
          alt: [
            'Documentos con una lista de expedientes, nombres privados difuminados',
          ],
        },
        {
          pille: 'Reconocimiento de texto',
          titel: 'De la foto al PDF con búsqueda',
          absaetze: [
            'Fotografía una carta y guárdala en un expediente. Se convierte en un PDF con una capa de texto invisible: buscar, resaltar, copiar.',
            'El reconocimiento se hace en tu dispositivo. La imagen no va a ninguna parte.',
          ],
          alt: [
            'Aviso de que se está convirtiendo una foto',
            'Carta fotografiada como PDF, la búsqueda resalta una palabra',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Cada nota como PDF',
          absaetze: [
            'Un toque convierte una nota en PDF y la guarda en tus documentos. Hay cuatro versiones:',
          ],
          punkte: [
            'Estándar, con título, fecha y fuentes',
            'Clean, solo el texto',
            'Carta según DIN 5008',
            'Versión impresa con portada',
          ],
          alt: [
            'Diálogo para crear un PDF con cuatro versiones',
            'Nota como PDF terminado en el visor',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'La promesa',
    titel: 'Solo tú puedes leerlo.',
    text: 'Todo se cifra en tu dispositivo antes de llegar al servidor. Alojado en Alemania, sin publicidad y sin rastreo.',
  },
  downloadsAugenbraue: 'Descargas',
  ios: {
    titel: 'iPhone y iPad',
    pille: 'En camino',
    text: 'La app para iOS está en camino. En cuanto esté lista, la encontrarás aquí.',
  },
  alleVersionen: {
    titel: 'Todas las versiones',
    text: 'Las versiones anteriores y todos los archivos están en GitHub.',
    knopf: 'Ir a GitHub',
  },
};
