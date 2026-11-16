// Atención vue-i18n: nada de @, | ni { sin escapar en los textos.
export default {
  title: 'Ayuda',
  intro: 'Aquí encontrarás descripciones e instrucciones sobre todos los módulos y funciones de Openany. Con el índice de la izquierda saltas directamente a la sección que necesitas.',
  version: 'Versión {version}',
  aktualisieren: 'Actualiza la app',
  aktualisierenTitel: 'openany.de necesita al menos la versión {version}.',
  toc: 'Contenido',
  sections: {
    start: {
      title: 'Primeros pasos',
      subs: {
        ohneKonto: {
          title: 'La app sin cuenta',
          p1: 'La app es un programa completo que funciona también sin cuenta y sin servidor: nla vía Openany de los mensajes, por ejemplo, solo aparece cuando el dispositivo está conectado con openany.de.',
          p2: 'Si quieres sincronizar tus datos con openany.de, crea una cuenta en los ajustes, en «Sincronización», o conéctate a una existente; con tus otros dispositivos te emparejas en «Dispositivos cercanos». Después, la cabecera y el menú muestran un botón «Sincronizar» que gira mientras se sincroniza.',
        },
        kontoAnlegen: {
          title: 'Crear una cuenta',
          p1: 'Creas una cuenta en openany.de directamente en la app: en los ajustes, en «Sincronización», toca «Crear cuenta nueva» y elige un nombre. Después, tu dispositivo resuelve un pequeño cálculo que protege contra la creación masiva de cuentas; suele tardar menos de un minuto. No necesitas dirección de correo ni contraseña.',
          p2: 'Luego aceptas las condiciones de uso; solo entonces tienes espacio en openany.de, 100 MB para empezar. Conectas otros dispositivos a través de uno ya conectado, con 6 dígitos (consulta «Mis dispositivos»).',
          p3: 'Como tu cuenta no tiene contraseña, un archivo de copia de seguridad es la única forma de volver si pierdes tu único dispositivo. La app te lo ofrece justo después de crear la cuenta; créalo y guárdalo bien junto con su frase de contraseña (consulta «Copia de seguridad en un archivo»).',
        },
        startseite: {
          title: 'La página de inicio',
          p1: 'La página de inicio muestra mosaicos compactos de los módulos que elegiste: las citas de hoy, la agenda con búsqueda, las tres notas editadas más recientemente, los tres últimos archivos con el espacio ocupado y tus hubs. Cada mosaico lleva a su módulo; cuáles aparecen lo decides en los ajustes, en «Módulos en la página de inicio».',
          p2: 'Al principio están elegidos todos los mosaicos. Si los quitas todos, aparece en su lugar la bienvenida: presenta todas las funciones y te permite componer los mosaicos directamente. La selección no oculta nada: todas las áreas siguen accesibles desde la barra y el menú.',
        },
        navigation: {
          title: 'Menú y estructura',
          p1: 'En el ordenador hay una cabecera arriba: a la izquierda, el logotipo redondo lleva a la página de inicio y a su lado están Notas, Calendario, Disco y Hubs. A la derecha siguen el interruptor claro/oscuro, el sobre de los mensajes con contador de no leídos y el campo con tu nombre, que abre Ajustes, Ayuda, Papelera y Cerrar sesión.',
          p2: 'En el móvil todo baja: una barra fija muestra los módulos como iconos y, a la izquierda del todo, está el botón de menú con el contador de mensajes no leídos. Abre un panel con Mensajes, Ajustes, Ayuda, Papelera, el interruptor claro/oscuro y Cerrar sesión. El interruptor alterna entre claro, oscuro y automático; automático sigue el ajuste de tu dispositivo. Un toque en el módulo ya abierto te lleva a su punto de inicio.',
        },
      },
    },
    notes: {
      title: 'Notas',
      subs: {
        editor: {
          title: 'Escribir en el editor',
          p1: 'El editor de notas funciona como un procesador de textos moderno: títulos, listas, citas, bloques de código, negrita y cursiva se eligen en la barra de herramientas. También puedes escribir atajos de Markdown –por ejemplo, una almohadilla seguida de un espacio para un título o un guion para una lista– y se convierten en formato mientras escribes.',
          p2: 'El guardado es automático: poco después de que dejes de escribir, Openany guarda la nota y muestra el estado en la barra de herramientas. Bajo cada nota figuran además la fecha de creación y la de la última modificación. En el móvil, la barra de herramientas aparece en una sola línea justo encima del teclado en cuanto tocas el texto y se puede desplazar lateralmente: así tienes todas las herramientas a mano sin que molesten al leer.',
        },
        mappen: {
          title: 'Carpetas y estructura',
          p1: 'Las notas están en carpetas que se pueden anidar sin límite. Puedes renombrar, mover y reordenar notas y carpetas: así crece con el tiempo tu propia colección de conocimiento. La lupa de arriba abre un campo de búsqueda que busca en todas las notas a la vez, en título y contenido; si la abres dentro de una carpeta, «solo en esta carpeta» la limita a ella. Lo que puedes hacer con la carpeta abierta está en «…», a la derecha de la barra de ruta.',
          p2: 'Subes por la ruta o con la tecla Atrás, que sube un nivel; una nota abierta se cierra con la cruz de la barra de ruta. Junto a cada carpeta ves cuándo cambió por última vez una nota en ella, también en sus subcarpetas. Una carpeta sin notas no muestra fecha.',
        },
        wikilinks: {
          title: 'Wikilinks y referencias inversas',
          p1: 'Con dos corchetes enlazas notas entre sí: escribe los corchetes y el título de la nota de destino y el enlace se crea automáticamente; si la nota aún no existe, el enlace se activa en cuanto exista. Si en el texto debe aparecer algo distinto del título, indicas un texto visible propio tras una barra vertical; el enlace sigue apuntando a la nota correcta.',
          p2: 'Cada nota muestra sus referencias inversas: una lista de todas las notas que enlazan a ella. Así vuelves a encontrar las relaciones sin tener que llevar la cuenta tú.',
        },
        tags: {
          title: 'Etiquetas',
          p1: 'Las palabras clave se asignan directamente en el texto con una almohadilla delante de la palabra; al escribir, una lista de sugerencias muestra las etiquetas ya usadas. Con una barra inclinada anidas etiquetas, por ejemplo Hub/Subtema; un filtro por el tema principal encuentra también todos los subtemas. En la vista de grafo, las etiquetas se pueden mostrar como nodos propios, y un clic en una etiqueta filtra allí la lista de notas.',
        },
        einfuegen: {
          title: 'Insertar imágenes y archivos',
          p1: 'El botón del clip de la barra de herramientas inserta en una nota un archivo de tu dispositivo. Las imágenes aparecen como una vista previa compacta en la propia nota (la resolución completa está en la Mediateca), y los demás archivos como un enlace para abrir.',
          p2: 'El contenido insertado se ordena automáticamente: las imágenes en un álbum de la Mediateca, los archivos de texto y de Office en un expediente de Documentos y todo lo demás en una carpeta de archivos, cada uno en un área propia para notas.',
        },
        zitate: {
          title: 'Citas y bibliografía',
          p1: 'Para el trabajo académico, cada carpeta lleva una lista de fuentes: en la carpeta abierta eliges «Fuentes» en «…» y añades libros, artículos, sitios web, sentencias o leyes. Cada fuente recibe un nombre corto. La lista es un archivo CSL-JSON en Archivos, utilizable también con Zotero y Zettlr.',
          p2: 'En las notas de esta carpeta, el editor sugiere fuentes adecuadas al citar y coloca la cita en el texto. Así las citas surgen mientras escribes, sin teclear los datos cada vez. Cómo aparecen en el PDF se explica en «Versión impresa».',
        },
        graph: {
          title: 'Vista de grafo',
          p1: 'La vista de grafo dibuja tus notas como una red: cada nota es un punto y cada wikilink, una conexión. Los puntos se colorean según la carpeta superior, de modo que las áreas relacionadas se reconocen por el color; los puntos huecos son notas sin ninguna conexión y, cuantas más conexiones tiene una nota, más grande es su punto.',
          p2: 'Cada carpeta de primer nivel es un nivel. Si hay varios, eliges uno o más en la leyenda bajo el grafo: «Todos los niveles» los toma todos, «Ninguno» vacía la selección. En lugar de un nivel, también una etiqueta puede ser la entrada: la barra de debajo enumera las etiquetas con su número, y una etiqueta elegida muestra todas las notas que la llevan, con la etiqueta como rombo gris. Si hay niveles elegidos, la barra solo muestra las etiquetas que aparecen en ellos. Hasta que eliges algo, el grafo queda vacío, para que las colecciones grandes no se construyan de golpe; «Mostrar todas las etiquetas» incorpora todas las de la barra a la vez.',
          p3: 'En el ordenador haces zoom con los botones más/menos de arriba a la derecha o con la rueda del ratón y desplazas la vista manteniendo pulsado el botón; en el móvil haces zoom con dos dedos y desplazas con uno. Otro botón restablece la vista. Un clic en un punto (sin arrastrar) abre la nota. Un clic en una etiqueta filtra la lista de notas por ella.',
        },
        export: {
          title: 'Exportar a PDF',
          p1: 'El botón PDF de la nota abierta ofrece cuatro versiones: «Estándar» es la nota con título, fecha y bibliografía. «Clean» es solo el texto, sin título, fecha, números de página ni bibliografía; las citas quedan tal como las escribiste. «Carta» compone la nota según DIN 5008 para sobre con ventana, con el título como asunto. «Versión impresa» genera solo esta nota con portada. El PDF va al expediente «Notizen» en Almacenamiento → Documentos (las versiones anteriores se conservan), y la versión impresa al expediente «Druckfassungen»; «Abrir» en el aviso lo muestra al momento.',
        },
        druckfassung: {
          title: 'Versión impresa',
          p1: 'Una carpeta se convierte en un documento formal para entregar o imprimir: en la carpeta abierta eliges «Versión impresa» en «…», introduces título, subtítulo, nombre, institución, fecha y datos adicionales para la portada, y decides si se añade un índice. Cada nota de la carpeta será un capítulo, por orden alfabético del título; con prefijos como «01 Introducción» fijas el orden, y en el documento aparece solo «Introducción». Las subcarpetas se incluyen si está marcada «Incluir subcarpetas»: una subcarpeta se convierte en un encabezado y sus notas quedan debajo («2 Fundamentos», «2.1 Conceptos»); una nota con el mismo nombre que su subcarpeta aparece como texto justo debajo de ese encabezado.',
          p2: 'Se compone con una tipografía tipo Times y títulos sin serifa, márgenes de 2,5 cm, interlineado 1,5, texto justificado con división silábica; la numeración empieza después de la portada. Los datos quedan guardados en la carpeta. El PDF se crea en tu dispositivo (la primera vez tarda unos segundos) y se guarda en el expediente «Druckfassungen» en Almacenamiento → Documentos; cada versión se conserva.',
          p3: "Citas de la lista de fuentes de la carpeta: en el texto escribes [{'@'}nombrecorto] o [{'@'}nombrecorto, p. 12], y ^[Texto] para una nota al pie. En el PDF – nota o versión impresa – se convierten en citas Harvard como «(Müller, 2020, p. 12)» con la bibliografía al final.La barra del editor tiene un botón para notas al pie (número superíndice): inserta los corchetes, y el texto seleccionado se convierte en la nota.",
        },
        papierkorb: {
          title: 'Papelera',
          p1: 'Las notas eliminadas van a la papelera (en el menú bajo tu nombre; en el móvil, en el botón de menú) y allí se pueden restaurar o eliminar definitivamente. Al eliminar una carpeta decides si las notas que contiene se eliminan también o se mueven a la carpeta superior.',
        },
        verschluesselt: {
          title: 'Cifrado de extremo a extremo',
          p1: 'Tus notas, carpetas, datos de la versión impresa y listas de fuentes solo se guardan cifrados en openany.de: solo tus dispositivos autorizados pueden leerlos. El servidor tampoco ve los nombres de tus carpetas. Tu dispositivo calcula la búsqueda, las referencias inversas, las etiquetas y el grafo.',
        },
        ordner: {
          title: 'Notas como archivos (Zettlr)',
          p1: 'En la app del ordenador puedes vincular una de tus carpetas de notas con una carpeta del sistema: abre la carpeta, elige «Vincular con una carpeta …» en el menú «…» y selecciona el destino. Cada nota queda allí como archivo «Id Título.md», las subcarpetas como subdirectorios, tal como lo esperan Zettlr u Obsidian. Lo que cambies, renombres, muevas o crees allí lo recoge la app mientras está abierta; si han cambiado ambos lados, se crea una copia de conflicto. Si borras un archivo, la nota va a la papelera. La carpeta se guarda sin cifrar en tu ordenador; «Desvincular carpeta» deja los archivos donde están. Las imágenes de las notas están junto a la nota, con el nombre que aparece en el texto; una imagen nueva allí que una nota enlace pasa a la app (mediateca, álbum «Notizen: …»).',
        },
      },
    },
    files: {
      title: 'Disco',
      subs: {
        aufbau: {
          title: 'Mediateca, archivos, documentos',
          p1: 'Disco se divide en tres pestañas en la parte superior: la Mediateca para fotos y vídeos, Archivos para todo lo demás y Documentos para papeles en expedientes. Al cambiar de pestaña, la otra zona queda como la dejaste: las carpetas abiertas y las listas cargadas no se pierden.',
          p2: 'Qué pestaña se abre primero al entrar en Disco lo decides en los ajustes, en «Inicio del disco»; mientras no elijas nada, es Documentos.',
        },
        dokumente: {
          title: 'Archivo de documentos',
          p1: 'En la pestaña «Documentos» organizas archivos PDF, de Office y de texto en expedientes. Se suben con el botón o simplemente arrastrando los archivos al expediente abierto. En móviles y tabletas hay al lado un botón de cámara: fotografía un papel y lo guarda directamente como documento en el expediente.',
          p2: 'Los expedientes se pueden anidar sin límite. Puedes renombrar y mover expedientes y documentos; los expedientes enteros también se pueden compartir con un hub en «Comparticiones». Junto a cada expediente ves cuándo cambió algo en él por última vez, también en sus subexpedientes.',
        },
        texterkennung: {
          title: 'Documentos con búsqueda',
          p1: 'Si fotografías un documento en papel y lo guardas en un expediente –con el botón de cámara o como foto de tu colección de imágenes–, se convierte automáticamente en un PDF con una capa de texto invisible. El documento se ve como tu foto, pero se puede buscar, marcar, copiar y leer en voz alta. El reconocimiento se ejecuta en tu propio dispositivo: la imagen no se envía a ningún sitio.',
          p2: 'Si guardas varias fotos a la vez, se te pregunta si deben formar un documento de varias páginas –como las páginas de una carta– o documentos separados. La foto en sí no se guarda aparte; si quieres conservar la imagen, guárdala en la Mediateca. La primera vez la conversión tarda más, porque el reconocimiento de texto se carga una sola vez y después queda guardado.',
          p3: 'Si ya hay un PDF escaneado en un expediente, puedes aplicarle el reconocimiento después: «Reconocer texto» lee el documento página a página. Si el papel es blanco y limpio, las páginas no cambian y solo se añade el texto. Si están grises o iluminadas de forma irregular –como una hoja fotografiada–, además se aclaran; el mensaje final te dice qué ha pasado. Si el PDF ya tenía búsqueda, no se toca.',
        },
        dateien: {
          title: 'Gestionar archivos',
          p1: 'En la pestaña «Archivos» creas carpetas y subes archivos de cualquier tipo, uno a uno o varios a la vez. Al tocar un archivo se abre con un programa adecuado de tu dispositivo. El icono de descarga guarda una copia en la carpeta Download o, en el ordenador, donde elijas; en los PDF está en el visor. Carpetas y archivos se pueden renombrar y mover; las carpetas también se pueden compartir con un hub en «Comparticiones». Lo que puedes hacer con una carpeta o un expediente abierto está en «…», a la derecha de la barra de ruta. Subes por la ruta o con la tecla Atrás, que sube un nivel.',
          p2: 'Cuánto ocupas lo ves en el mosaico de la página de inicio. Cuánto se queda en este dispositivo lo decides en los ajustes, en «Almacenamiento en este dispositivo».',
        },
        suche: {
          title: 'Buscar',
          p1: 'La lupa de Archivos y Documentos abre un campo de búsqueda que recorre todas las carpetas o expedientes a la vez, no solo el que está abierto. Un clic en un resultado abre el archivo; al lado ves dónde está, y un clic en esa indicación te lleva a la carpeta.',
          p2: 'La búsqueda no solo encuentra por nombre, sino también en el texto: en PDF, textos de Word y OpenDocument y archivos de texto, CSV y Markdown de hasta 50 MB. Openany lee ese texto poco a poco en segundo plano mientras Disco está abierto; hasta que lo haya leído todo, la búsqueda te dice cuántos archivos de momento solo se encuentran por nombre. Si el resultado está en el texto, la lista muestra el pasaje con la palabra buscada resaltada. Los PDF escaneados sin capa de texto solo aparecen después de aplicarles «Reconocer texto».',
        },
        pdf: {
          title: 'Ver y editar PDF',
          p1: 'Un clic en un PDF lo abre directamente en Openany. Pasas las páginas, amplías o reduces, ajustas la vista al ancho y buscas en el documento con la lupa: los resultados se resaltan en la página. «Abrir con otra aplicación» lo pasa a otro programa. Si un PDF está protegido con contraseña, Openany te la pide primero.',
          p2: 'Los formularios del PDF los rellenas directamente. Con «Editar» aparece además una barra de herramientas: «Resaltar» marca texto en color, con el «Lápiz» dibujas libremente sobre la página y con «Texto» colocas tus propias líneas en la hoja; color, grosor del trazo y tamaño de letra los eliges tú, y cada paso se puede deshacer. Eso sí, los campos de cálculo de los formularios no calculan, porque Openany no ejecuta scripts de archivos ajenos.',
          p3: 'A los resaltados y trazos les puedes añadir comentarios: elige «Comentarios» y toca el lugar; esa misma vista lista todos los comentarios del documento. Todo se guarda como anotaciones PDF normales, que también muestran otros programas. Al guardar por primera vez, Openany pasa la versión anterior a la papelera, para que puedas volver al original; si sales del PDF con cambios sin guardar, te pregunta antes.',
        },
        galerie: {
          title: 'Mediateca y álbumes',
          p1: 'En la pestaña «Mediateca» están tus fotos y vídeos: en álbumes con miniaturas o, sin álbum, debajo, en «Medios». Los álbumes se pueden anidar –por ejemplo un álbum por viaje con un subálbum por día– y mover en cualquier momento. El visor recorre todas las imágenes de un álbum. Una foto se mueve desde la vista grande; si la mantienes pulsada en la cuadrícula, seleccionas varias y las mueves o eliminas juntas. Lo que puedes hacer con el álbum abierto está en «…», a la derecha de la barra de ruta.',
          p2: 'La toma más reciente aparece arriba, agrupada por meses con un encabezado para cada mes. Cuenta la fecha de captura de los datos de la foto; si falta –por ejemplo en capturas de pantalla o imágenes editadas–, cuenta el día en que se subió.',
          p3: 'En la vista grande descargas una foto o un vídeo como archivo original: en el móvil a la carpeta Download, en el ordenador donde elijas. Los álbumes se pueden compartir con un hub en «Comparticiones». Los originales siguen siendo tuyos. Con «Acciones» en el álbum subes tu propia imagen de portada; no cuenta como elemento.',
        },
        fotosVomGeraet: {
          title: "Fotos del dispositivo",
          p1: "En Android, en la mediateca, «Añadir» → «Añadir fotos del dispositivo» incorpora fotos y vídeos que ya están en el dispositivo: todos, solo ciertos álbumes del dispositivo o a partir de una fecha. Los álbumes del dispositivo se convierten en álbumes con el mismo nombre en openany; antes ves si hay espacio suficiente, y lo que ya se añadió se omite. Si el dispositivo está conectado a openany.de, eliges una vez cómo entran las fotos: en openany.de, y en el dispositivo solo queda la vista previa sin ocupar espacio doble, o como copia en este dispositivo.",
          p2: "Con «Añadir fotos nuevas automáticamente» – en la mediateca, en «Añadir», o en Ajustes → Almacenamiento y copia de seguridad – lo que la cámara capture a partir de ahora llega solo a la mediateca, aunque openany no esté abierto: desde los álbumes del dispositivo que elijas, desde que lo activas. Si quieres, solo con Wi-Fi o solo mientras carga. Android limita el trabajo en segundo plano, así que las fotos llegan con algo de retraso. En openany.de cuentan para tu almacenamiento. Si usas Google Fotos, desactiva allí la copia de seguridad; si no, suben las dos.",
        },
        videos: {
          title: 'Vídeos y cámara',
          p1: 'Además de fotos, la Mediateca admite vídeos. Aparecen como mosaico con una imagen fija; al tocarlos se abren con el programa de vídeo de tu dispositivo.',
          p2: 'En móviles y tabletas, la Mediateca y cada álbum tienen dos botones de cámara: uno para foto y otro para vídeo. La grabación va directamente al lugar que tienes abierto. En el ordenador no hay botones; allí subes como siempre.',
        },
        ton: {
          title: 'Grabaciones de audio y música',
          p1: 'La mediateca también guarda archivos de audio como MP3 o M4A. Aparecen como un mosaico con nombre y duración; un toque los reproduce en la app. Los audios que llegan de fuera también se guardan aquí.',
          p2: 'El botón del micrófono graba directamente, con pausa, descartar y guardar. La grabación va al álbum abierto o, si no, al álbum «Grabaciones». En Android sigue con la pantalla apagada, como indica la notificación. En Linux no existe este botón.',
          p3: 'En Android (versión 10 o posterior) y Windows puedes elegir el audio del sistema en lugar del micrófono. El sistema pide tu consentimiento cada vez. Algunas apps lo bloquean y las llamadas nunca se graban. Graba conversaciones solo si todas las personas están de acuerdo.',
        },
        karte: {
          title: 'Fecha y lugar de captura',
          p1: 'Para cada imagen, el visor muestra la fecha de captura y, si la foto lo contiene, el lugar; «Ver en el mapa» abre el sitio en OpenStreetMap. Si subes fotos desde un móvil Android, elígelas con la app Archivos y no con la Galería: si no, Android elimina la ubicación antes de que la imagen llegue a Openany.',
        },
        papierkorbSpeicher: {
          title: 'Papelera',
          p1: 'El contenido eliminado –documentos, archivos, fotos, notas, contactos, pero también hubs, calendarios y citas– pasa primero a la papelera común y allí se puede restaurar. A los 30 días la papelera se vacía automáticamente; solo entonces el espacio queda libre definitivamente.',
        },
        dateienVerschluesselt: {
          title: 'Cifrado de extremo a extremo',
          p1: 'Tus archivos, expedientes, fotos y álbumes solo se guardan cifrados en openany.de: nombres, carpetas, álbumes, fechas de captura y ubicación van en una cabecera cifrada, y los contenidos se cifran con una clave propia para cada archivo. Solo tus dispositivos autorizados pueden leerlos.',
        },
      },
    },
    calendar: {
      title: 'Calendario',
      subs: {
        verwalten: {
          title: 'Varios calendarios',
          p1: 'Puedes llevar tantos calendarios como quieras en paralelo –por ejemplo personal, familia, club–, cada uno con su color. Un clic muestra u oculta un calendario. Arriba cambias entre mes, semana y día; semana y día muestran las citas por hora, y las pasadas aparecen discretamente en gris. En móvil y tableta pasas de página deslizando: hacia arriba y abajo en el mes, de lado en semana y día. En el ordenador hay flechas para ello.',
        },
        termine: {
          title: 'Crear citas',
          p1: 'Creas citas con un clic en un día: con título, descripción, hora, o como evento de todo el día o de varios días. También hay repeticiones: diarias, semanales, mensuales o anuales. Un clic en una cita muestra los detalles, donde la editas o eliminas.',
          p2: "Un recordatorio avisa a la hora elegida antes del inicio. Si una cita llega de otro programa con una repetición más precisa – cada dos semanas, solo tres veces, los martes y jueves, cada primer lunes –, se conserva y aparece en palabras en la cita. Si eliges tú otra repetición, sustituye a la anterior. Una cita movida por separado dentro de una serie sigue siendo una cita propia, y la serie deja libre su antiguo lugar.",
        },
        abos: {
          title: 'Suscripciones de calendario',
          p1: 'Los calendarios externos –por ejemplo, festivos o el calendario de partidos del club– se integran mediante una URL de suscripción en formato ICS. Los calendarios suscritos se actualizan automáticamente. Sus citas en Openany son solo de lectura: se cambian allí de donde viene el calendario, porque cada actualización vuelve a sustituir el contenido por el original.',
        },
        verschluesselt: {
          title: 'Cifrado de extremo a extremo',
          p1: 'Tus calendarios y citas solo se guardan cifrados en openany.de: solo tus dispositivos autorizados pueden leerlos. La dirección de una suscripción también va cifrada; cada dispositivo la consulta por sí mismo, openany.de no consulta nada.',
        },
        importExport: {
          title: 'Importar y exportar',
          p1: 'Con «Importar» cargas un archivo ICS en uno de tus calendarios. Si importas el mismo archivo otra vez, las citas se actualizan en lugar de duplicarse. El icono de compartir junto a un calendario lo guarda como archivo ICS, para llevarlo a cualquier otra aplicación de calendario.',
        },
        imTelefon: {
          title: "En el calendario del teléfono",
          p1: "En Android, Ajustes → Calendario → «Mostrar en el calendario del teléfono» muestra tus calendarios en cualquier app de calendario del teléfono, en la cuenta «openany» – también los compartidos en hubs, y las suscripciones si las añades. Lo que cambies, crees o borres allí vuelve a openany. Los recordatorios los muestra entonces el propio teléfono.",
          p2: "Google Calendar solo muestra otras cuentas tras un paso tuyo: allí, Ajustes → Gestionar cuentas → activar «openany». Las citas solo se descifran en este dispositivo. Al desactivarlo desaparecen del calendario del teléfono; en openany se conserva todo.",
        },
        andereProgramme: {
          title: "Para otros programas en el ordenador",
          p1: "En el ordenador, Ajustes → Calendario → «Ofrecer a otros programas de este ordenador» ofrece tus calendarios y tu agenda por CalDAV y CardDAV – para Thunderbird, Calendario y Contactos de Apple y otros. El mosaico muestra dirección, usuario y contraseña para copiar; con ellos configuras una cuenta en el programa. En Thunderbird: Nuevo calendario → En la red, o Nueva libreta CardDAV. En Apple: Otra cuenta CalDAV o CardDAV → Manual. Si varias cuentas de usuario comparten el ordenador, actívalo solo mientras lo necesites: cuando Openany no está abierto, otra cuenta podría responder en su lugar y recibir la contraseña que envía tu programa.",
          p2: "El servicio solo es accesible en este ordenador y solo mientras openany esté abierto; si la app está cerrada, el programa se pone al día más tarde. Los cambios hechos allí vuelven a openany; los calendarios compartidos en hubs allí solo se pueden leer. «Nueva contraseña» deja fuera a los programas ya configurados hasta que reciban la nueva.",
        },
      },
    },
    projects: {
      title: 'Hubs',
      subs: {
        grundlagen: {
          title: 'Hubs y miembros',
          p1: 'Un hub es un espacio de trabajo compartido: invitas a otros usuarios como miembros y ellos aceptan o rechazan la invitación. El propietario del hub gestiona a los miembros y también puede ceder el hub a otro miembro.',
          p2: 'Cada hub tiene cuatro pestañas: Comparticiones, Board, Miembros y Chat. Se abre en Comparticiones. Los números de las pestañas indican los mensajes de chat no leídos o cuánto contienen.',
        },
        chat: {
          title: 'Chat del hub',
          p1: 'Cada hub tiene un chat común en tiempo real: los mensajes aparecen al instante para todos los miembros, sin recargar la página. El chat está cifrado de extremo a extremo: solo los miembros pueden leerlo en sus dispositivos, openany.de no. Si alguien sale, la clave cambia sola y ya no lee nada nuevo.',
          p2: 'Cualquier miembro puede fijar mensajes importantes: los mensajes fijados se encuentran rápido en la barra de fijados sobre el historial y allí también se pueden soltar.',
          p3: 'Con dos corchetes enlazas, en mitad de una frase, contenidos del hub: notas y carpetas, archivos, álbumes e imágenes compartidos. Al escribir, una lista de sugerencias muestra lo que encaja e inserta el enlace terminado. El botón con el símbolo de cadena junto al de enviar hace lo mismo – en el móvil suele ser más cómodo que buscar dos corchetes.',
          p4: 'Un clic en un enlace lleva directamente al destino: a la nota, a la carpeta con el archivo resaltado o a la imagen en la vista ampliada. Solo puedes enlazar lo que ya es visible en el hub – lo que nadie ha compartido no aparece en la lista de sugerencias ni se vuelve un enlace pulsable.',
        },
        board: {
          title: 'Board',
          p1: 'Cada hub tiene exactamente un board: un lienzo compartido que se crea junto con el hub. Al tocar «Board» se abre; «Atrás» vuelve a la pestaña de la que venías. Todos los miembros dibujan, escriben y ordenan en él; se guarda solo, poco después de cada cambio. Con «Insertar» colocas en el board imágenes, notas y archivos compartidos del hub como tarjeta.',
          p2: 'Si varias personas dibujan a la vez, la app une sus cambios: cada elemento toma su versión más reciente, y lo borrado sigue borrado. Con miembros cerca dibujáis juntos en directo y veis los punteros de los demás; si no, los cambios llegan con la próxima sincronización. El board está cifrado de extremo a extremo.',
        },
        geteilteKalender: {
          title: 'Compartir calendarios',
          p1: 'En «Comparticiones», con «Compartir» (en persona: «Compartir algo») compartes también uno de tus calendarios con el hub. Aparece entonces en la lista bajo «Calendarios»; al tocarlo se abre en el calendario – para los miembros en «De hubs». Con «ver» solo lo leen; con «editar» crean, cambian y borran citas. El calendario en sí solo lo cambias tú. Si dos personas cambian la misma cita a la vez, la segunda versión queda al lado como copia, con «Conflicto», fecha y hora en el título. Todo está cifrado con una clave propia. Con la X junto al calendario dejas de compartirlo; tu calendario se queda. Sin openany.de, los miembros lo reciben en cuanto sincronizan cerca – allí solo para leer; los calendarios suscritos solo se comparten así.',
        },
        freigaben: {
          title: 'Compartir contenido',
          p1: 'En la pestaña «Comparticiones», «Compartir» comparte con el hub una carpeta de notas, un álbum o una carpeta (o expediente): primero eliges el tipo y luego cuál. Todo sigue cifrado de extremo a extremo, con una clave propia por cada compartición que cambia cuando dejas de compartir o alguien sale del hub. El contenido sigue siendo tuyo; si dejas de compartir, lo conservas todo.',
          p2: 'Los álbumes y carpetas se comparten solo para ver: los miembros los abren en el hub, ven subálbumes y subcarpetas, abren imágenes y PDF y con «Copiar a lo mío» guardan una copia en su propio almacenamiento, en «De hubs». Solo tú puedes cambiarlos.',
          p3: 'Las carpetas de notas se comparten con el nivel «ver» o «editar». Los miembros las abren en el editor de notas habitual y ven también sus imágenes; con «editar» cambian notas y crean otras nuevas. De momento, solo quien es dueño de la carpeta puede insertar imágenes.',
        },
        lokal: {
          title: 'En openany.de o solo en persona',
          p1: 'Con «+ Hub» creas un hub en este dispositivo. Si el dispositivo está conectado a openany.de, va también allí enseguida; en la lista lleva entonces una pequeña nube. Quien fundó un hub lo activa y desactiva en Ajustes → Sincronización → «Hubs en openany.de». Desactivado, solo se sincroniza en persona; en openany.de se borran el hub, el chat, la planificación y lo compartido. Con nube, invitas a los miembros por su nombre de openany (solo propietarios, y solo a quien ya haya sincronizado la app con openany.de alguna vez). Sin nube, invitas en «Invitar a un miembro cercano»: en el otro dispositivo debe estar abierto Openany, en la misma wifi o punto de acceso, y ambos muestran los mismos seis dígitos, que comparáis y confirmáis. El hub lo gestionan los dispositivos del propietario; si solo hay uno, la app avisa: empareja antes un segundo dispositivo propio. Cuando eliminas un hub, primero queda firmado en su lista de miembros; solo así saben los dispositivos de los demás que ya no existe. Si openany.de indica que un hub se eliminó sin esa constancia, se queda en tu dispositivo y sigue sincronizando cerca.',
          p2: 'Los miembros comparten sus propias carpetas de notas, carpetas y álbumes en el hub con «Compartir algo», al principio solo para leer. Las carpetas de notas también se pueden compartir para editar: entonces solo una persona edita una nota a la vez, y se guarda en el dispositivo de la persona a la que pertenece la carpeta; si ese dispositivo no está cerca, la nota queda solo de lectura. El chat llega al instante a todos los que están cerca, y a los demás en la siguiente sincronización. El board viaja de la misma manera.',
          p3: 'El resto lo sincronizas con «Sincronizar» en cuanto haya un miembro cerca. El propietario puede quitar miembros; quien quiera irse sale con «Salir», y eso solo funciona si otro miembro está cerca para enterarse. Lo que habían compartido quienes se van desaparece para los demás; lo que ellos ya tenían se queda en sus dispositivos.',
        },
      },
    },
    messages: {
      title: 'Mensajes',
      subs: {
        direkt: {
          title: 'Mensajes directos',
          p1: 'Abres tus mensajes desde el menú. Con «Mensaje» arriba a la derecha escribes directamente a cualquier otro usuario – basta con su nombre de Openany. Los enlaces de los mensajes se pueden pulsar.',
          p2: 'Debajo de cada mensaje recibido, «Responder» abre el editor con el remitente ya puesto, por el mismo canal por el que llegó el mensaje. Los mensajes largos aparecen plegados; «Leer más» los muestra enteros. El contador de no leídos en el botón de menú indica si hay algo nuevo.',
          p3: 'Los mensajes directos están cifrados de extremo a extremo: openany.de solo guarda datos ilegibles y solo tus dispositivos pueden leerlos. Todos tus dispositivos comparten una clave. Un dispositivo nuevo muestra 6 dígitos; los introduces en otro ya configurado y queda conectado y con la clave. La clave también está en tu archivo de copia de seguridad. Si pierdes un dispositivo, lo retiras en los ajustes en «Mis dispositivos»; al hacerlo, la clave cambia. Puedes escribir a cualquiera que haya abierto la app al menos una vez.',
        },
        suche: {
          title: 'Buscar y filtrar',
          p1: 'Encima del historial hay un campo de búsqueda. Encuentra mensajes por su texto, por el nombre de la otra persona y por identificadores de Matrix, y resalta las coincidencias. Debajo, unos interruptores acotan la lista: a los mensajes no leídos, a los que tienen adjunto y –si usas más de una vía– a vías concretas como Openany o Matrix. Los interruptores se pueden combinar.',
          p2: 'Junto al campo de búsqueda cambias entre dos vistas: «Historial» muestra todos los mensajes por orden de tiempo, «Por contacto» los resume en una fila por persona, con el último mensaje y el número de no leídos. Un clic en una fila muestra el historial con esa persona; «Cerrar conversación» te devuelve a todos.',
        },
        anhaenge: {
          title: 'Adjuntos',
          p1: 'Por Matrix también envías archivos: el clip del campo de escritura adjunta un archivo de hasta 10 MB, y en móviles y tabletas el botón de cámara de al lado hace directamente una foto. En el historial, las imágenes aparecen como vista previa y los demás archivos con nombre y tamaño; un clic abre imágenes y PDF directamente en Openany, lo demás lo descargas. Por la vía Openany van archivos de hasta 10 MB, cifrados de extremo a extremo como el texto. Cuentan para tu almacenamiento y el del destinatario hasta que cada uno borre el mensaje.',
        },
        matrixApp: {
          title: 'Mensajes por Matrix',
          p1: 'En la app, tu propio dispositivo es el dispositivo de Matrix. Inicias sesión con una cuenta existente en los ajustes, en «Cuentas de Matrix», y tus mensajes van cifrados de extremo a extremo entre este dispositivo y la otra persona: openany.de no los lee. Para que otras apps de Matrix como Element confíen en este dispositivo, verifícalo allí, en la cuenta: con la clave de recuperación de Element o, si tu cuenta aún no tiene copia de seguridad, openany crea una y te muestra la nueva clave una vez.',
          p2: 'Puedes conectar varias cuentas de Matrix. Sus mensajes comparten historial; los nuevos salen de la cuenta predeterminada, las respuestas de la cuenta en la que llegó el mensaje, y «Hacer predeterminado» convierte otra cuenta en la predeterminada. Si borras un mensaje de Matrix, desaparece solo en este dispositivo; en la sala sigue estando.',
        },
        email: {
          title: 'Correo con tu buzón',
          p1: 'En la app se suma el correo electrónico como otra vía: no un programa de correo propio, sino tu buzón de siempre, por ejemplo en Posteo, mailbox.org o GMX. En los ajustes, en «Buzones de correo», introduces dirección y contraseña; normalmente es una contraseña de aplicación que creas antes en tu proveedor. La app busca los servidores por sí misma; si no los encuentra, los introduces en «Servidores a mano». La contraseña queda guardada bajo llave en este dispositivo, y el propio dispositivo habla con el servidor de correo: openany.de no ve ningún correo.',
          p2: 'Los correos aparecen con su asunto en el historial común de Mensajes; «Recoger ahora» trae los nuevos al momento. Si conectas varios buzones, los correos nuevos salen del buzón predeterminado y las respuestas del buzón al que llegó el correo; al escribir eliges en «De». Al borrar decides si un correo solo desaparece aquí o pasa también a la carpeta de papelera del servidor. Si tu proveedor ha clasificado algo como spam, aparece un aviso encima del historial; allí, «No es spam» devuelve el correo a la bandeja de entrada.',
          p3: 'Por correo, los adjuntos pueden ocupar hasta 20 MB. Los adjuntos recibidos los guardas en tus archivos con «Guardar» o en la carpeta de descargas con «En el dispositivo». Si un adjunto llegó cifrado, la app pregunta antes: en Archivos queda sin cifrar y se sincroniza con openany.de.',
        },
        pgp: {
          title: 'Correo cifrado con OpenPGP',
          p1: 'Puedes cifrar correos de extremo a extremo con OpenPGP. Para ello cada buzón necesita su propia clave: en los ajustes, junto al buzón, la creas con «Crear clave»; o, si ya usas el buzón con PGP –por ejemplo en Thunderbird–, importas la existente con «Importar clave», para que ambos programas lean los mismos correos. La clave secreta está solo en este dispositivo. «Guardar copia de seguridad» la guarda, cerrada con una frase de contraseña, en la carpeta de descargas; «Guardar clave pública» te da el archivo que envías a los demás.',
          p2: 'Las claves públicas de tus contactos las reúne la app por sí sola en «Claves de los contactos», a partir de los correos que las incluyen. Si falta alguna, la buscas por la dirección –si quieres, también en keys.openpgp.org, que entonces sabrá por quién preguntas– o la importas desde un archivo. Lo mejor es comparar una vez la huella con la otra persona; si una clave cambia, la app te avisa.',
          p3: 'Al escribir activas «Enviar cifrado»; debajo verás si se conoce la clave del destinatario. Si respondes a un correo cifrado, el interruptor ya está activado. En el historial, los correos llevan la marca «cifrado» y, si están firmados, «firmado», o un aviso si la firma no es válida.',
        },
        vorOrt: {
          title: 'Mensajes cerca',
          p1: 'Con la vía «Cerca» escribes directamente a dispositivos próximos, sin ningún servidor. En el otro dispositivo Openany tiene que estar abierto, y ambos deben estar en la misma wifi. Podéis escribiros en cuanto os conozcáis: tus propios dispositivos, miembros de un hub común o personas que hayáis confirmado en persona. Para ello eliges «Confirmar en persona», ambos dispositivos muestran los mismos 6 dígitos y los dos pulsáis «Coincide». Si el otro dispositivo no está en ese momento, el mensaje espera y se envía en el siguiente encuentro.',
          p2: 'De los desconocidos, al principio, no llega nada. Si en los ajustes, en «Dispositivos cercanos», activas «Permitir solicitudes de dispositivos cercanos», podrán enviarte hasta 3 mensajes de 500 caracteres cada uno; aparecen en «Solicitudes» encima del historial, no en el historial. Solo podrás responder tras confirmaros con los 6 dígitos. A quien no deba enviarte nada más lo frenas con «Bloquear». Si uno de tus mensajes fue rechazado, muestra «no entregado», y el motivo aparece al señalarlo.',
        },
        systemnachrichten: {
          title: 'Avisos y respuestas',
          p1: 'Los eventos de tus hubs, como nuevos elementos compartidos, te llegan como aviso si el remitente informa a los miembros; también el aviso de que alguien te ha añadido a un hub y los mensajes de openany.de sobre tu cuenta, como la petición de confirmarla. Los avisos no están cifrados de extremo a extremo. Aquí también encuentras la respuesta a un mensaje enviado con el formulario de soporte.',
        },
      },
    },
    contacts: {
      title: 'Agenda',
      subs: {
        adressbuch: {
          title: 'Contactos y sus vías',
          p1: 'La agenda se abre desde su mosaico en la página de inicio o arriba en la página de mensajes; el botón de al lado crea directamente un contacto nuevo. Un contacto recoge cómo se puede localizar a alguien: nombre, imagen, teléfono y correo, además de tantas vías como quieras –Matrix, Meshtastic, nombre en Openany, dirección postal, empresa, cumpleaños u otros–, cada una con su etiqueta, como personal o trabajo. El campo de búsqueda encuentra contactos por nombre.',
          p2: 'Las vías se pueden pulsar: un número de teléfono llama, una dirección de correo abre el programa de correo, y un nombre en Openany o un identificador de Matrix abre el campo de escritura con el destinatario ya introducido. Los contactos eliminados van a la papelera.',
        },
        verschluesselt: {
          title: 'Cifrado de extremo a extremo',
          p1: 'Tu libreta de direcciones está cifrada de extremo a extremo, con la misma clave que tus mensajes directos. Solo tus dispositivos autorizados pueden leerla; openany.de la guarda y la sincroniza, pero no ve nombres, números ni imágenes. Un dispositivo nuevo solo muestra la libreta cuando lo has autorizado en otro dispositivo.',
        },
        vcard: {
          title: 'Guardar e importar vCard',
          p1: 'En Ajustes → Agenda guardas todos los contactos como archivo vCard o importas uno de otro programa. Importar solo añade: quien ya está se reconoce por su identificador o su nombre y conserva lo que tiene. El archivo en sí no está cifrado: guárdalo con cuidado.',
        },
        imTelefon: {
          title: "En los contactos del teléfono",
          p1: "En Android, Ajustes → Agenda → «Mostrar en los contactos del teléfono» muestra tus contactos en la agenda del teléfono, en la cuenta «openany» – cuando alguien llama, aparece su nombre. Nombre, teléfono, correo, dirección, empresa, cumpleaños, web, apodo y foto pasan al teléfono; lo que cambies, crees o borres allí vuelve. El teléfono no conoce los identificadores de Matrix, Meshtastic ni Openany: se quedan solo en openany. openany restablece una foto cambiada allí. Al desactivarlo desaparecen los contactos de allí.",
        },
        andereProgramme: {
          title: "Para otros programas en el ordenador",
          p1: "En el ordenador, por ejemplo Thunderbird o Contactos de Apple leen y escriben tu agenda por CardDAV – se activa en Ajustes → Calendario, con los mismos datos que para los calendarios (ver allí). Viajan todas las vías de contacto, también Matrix, Meshtastic y el nombre de Openany.",
        },
        sicherheitsnummer: {
          title: 'Número de seguridad',
          p1: 'Con el número de seguridad compruebas que nadie lee en medio, ni siquiera openany.de. Ábrelo en el contacto, en «Seguridad», o en los mensajes tras abrir una conversación desde «Por contacto». Comparad los doce grupos de cifras en vuestros dos dispositivos; si coinciden, pulsa «Coincide». Los mensajes de esa persona llevarán entonces una marca junto al candado. Si su clave cambia más adelante, por ejemplo tras perder un dispositivo, la marca desaparece y un aviso os pide comparar de nuevo.',
        },
      },
    },
    settings: {
      title: 'Ajustes y cuenta',
      subs: {
        module: {
          title: 'Módulos en la página de inicio',
          p1: 'Aquí eliges qué módulos aparecen como mosaico en la página de inicio: Notas, Calendario, Disco, Hubs y Agenda. La selección no oculta nada; todos los módulos siguen accesibles. Sin selección, la página de inicio muestra la bienvenida.',
        },
        spracheDesign: {
          title: 'Apariencia e idioma',
          p1: 'En «Apariencia» eliges entre dos diseños: «Nítido», en petróleo con esquinas más marcadas y superficie tranquila, o «Lila», con esquinas redondeadas y fondo con patrón. Ambos existen en claro y oscuro; eso lo cambias por separado con el interruptor de la cabecera o del menú del móvil. En «Idioma» eliges en qué idioma te habla Openany: alemán, inglés, español, francés, portugués, polaco o ucraniano. Mientras no elijas, sigue el idioma de tu dispositivo o navegador; si no está disponible, Openany habla inglés.',
        },
        abgleich: {
          title: 'Sincronizar con Openany',
          p1: 'En «Sincronización» conectas la app con openany.de. Si aún no tienes cuenta, crea una con «Crear cuenta nueva» (consulta «Crear una cuenta»). Si ya tienes un dispositivo conectado, autorizas el nuevo en él con 6 dígitos (consulta «Mis dispositivos»). Una cuenta antigua con contraseña se conecta con «Conectar con openany.de»: la app muestra un código que introduces y confirmas en la página de anyid en el navegador. Conectar es opcional: sin conexión todo se queda en este dispositivo, y «Desconectar» lo deshace en cualquier momento sin perder datos aquí.',
          p2: 'Después, «Sincronizar» sincroniza en ambos sentidos y luego te dice qué se ha traído, enviado u omitido. «Volver a traerlo todo» vuelve a descargarlo todo en lugar de solo lo nuevo; no se borra nada.',
          p3: 'Con «Actualizar en segundo plano» la app sincroniza sola más o menos cada hora, solo con conexión y no con la batería baja. Con datos móviles solo llega la lista; archivos e imágenes, solo con wifi. Con wifi también lo intenta con los dispositivos cercanos emparejados si Openany está abierto en ellos. En el ordenador, Openany sincroniza cada hora mientras está abierto; con «Seguir en segundo plano» se inicia al iniciar sesión y, al cerrar la ventana, se queda como icono en la barra de menús.',
        },
        geraeteNah: {
          title: 'Dispositivos cercanos',
          p1: 'Tus propios dispositivos también se sincronizan directamente entre sí, sin servidor, siempre que estén en la misma wifi o punto de acceso y Openany esté abierto en ambos. En «Dispositivos cercanos» los encuentras con «Buscar» y los conectas con «Emparejar»: los dos muestran el mismo código y en ambos confirmas que coincide. A partir de entonces se sincronizan solos mientras Openany esté abierto en ambos, y en cada «Sincronizar»; «Olvidar» deshace el emparejamiento.',
          p2: 'En «Tu nombre» decides cómo apareces ante los demás en persona. Con ese nombre te invitan a hubs –a ti como persona, no a un dispositivo concreto–, y vale para todos tus dispositivos emparejados.',
        },
        meineGeraete: {
          title: 'Mis dispositivos',
          p1: 'Lo que cifras de extremo a extremo solo pueden leerlo tus dispositivos; para ello comparten una clave. Un dispositivo nuevo muestra 6 dígitos, y los introduces aquí en un dispositivo ya configurado. Si pierdes un dispositivo, lo retiras aquí; la clave cambia al hacerlo. La clave también está en tu archivo de copia de seguridad.',
          p2: 'Abajo del todo en «Mis dispositivos» eliminas tú mismo tu cuenta en openany.de, al instante y con todo lo que hay allí. Por seguridad, escribes tu nombre de usuario. Los hubs que aún tienen otros miembros los traspasas antes a alguien.',
        },
        fristen: {
          title: 'Confirmar la cuenta y seguir activo',
          p1: 'Para que no se acumulen cuentas sin uso, openany.de te pide dos días después de crear la cuenta que la confirmes otra vez; arriba aparece una franja con «Conservar cuenta». Si no confirmas en 14 días, la cuenta se elimina.',
          p2: 'Si no abres la app durante cuatro meses, llega la misma petición, con un último recordatorio dos semanas antes de la eliminación. Basta con tocar «Conservar cuenta». Cuando se elimina una cuenta, solo desaparece el contenido de openany.de; lo que hay en tus dispositivos se queda.',
        },
        speicherGeraet: {
          title: 'Almacenamiento en este dispositivo',
          p1: 'En «Almacenamiento en este dispositivo» decides qué queda aquí. «Conservarlo todo» trae todo lo de tus álbumes, expedientes y carpetas mientras haya espacio; está pensado para ordenadores y tabletas. Con «Álbumes, expedientes y carpetas seleccionados» aquí solo queda lo que eliges en Almacenamiento con «Conservar en este dispositivo»; de todo lo demás solo ves el nombre. Las fotos nuevas de la cámara van al álbum «Cámara». Lo que no está en un álbum, expediente o carpeta se queda en su dispositivo y nunca se sincroniza.',
          p2: 'Al servidor van las notas, los calendarios, los contactos, los mensajes y los nombres de álbumes, expedientes y carpetas. Las fotos y los archivos solo van si están compartidos en un hub, adjuntos a una nota o si eliges «Poner en el servidor», uno a uno o para todo un álbum, expediente o carpeta. Todo lo demás se sincroniza directamente entre tus dispositivos. «Quitar del servidor» no borra nada: tus dispositivos conservan sus copias y la última copia siempre se queda.',
          p3: 'En «Almacenamiento en este dispositivo» también ves cuánto hay en el servidor, repartido entre mensajes, notas, mediateca, archivos y documentos.',
        },
        sofort: {
          title: 'Avisar al instante',
          p1: 'Con «Avisar al instante» la app mantiene conexiones ligeras con Openany y con tus buzones de correo, para que los mensajes y correos nuevos lleguen al momento, sin Google. Para los buzones no hace falta Openany. Android muestra por ello un aviso permanente, que puedes ocultar en los ajustes del sistema. En el ordenador, Openany vigila mientras está abierto y avisa con una notificación del sistema.',
        },
        sicherung: {
          title: 'Copia de seguridad en un archivo',
          p1: 'En «Copia de seguridad», «Crear copia» genera un archivo cifrado con todo lo que Openany tiene en este dispositivo: notas, calendario, contactos, archivos, mediateca, hubs, mensajes y tus buzones con contraseña y tu propia clave OpenPGP. La app propone una frase de seis palabras; apúntala y guárdala aparte del archivo. Sin ella el archivo no se puede abrir, ni siquiera nosotros.',
          p2: 'Con «Restaurar copia» abres un archivo así en otro dispositivo. Sustituye todo lo que había; después queda desconectado de openany.de y vuelves a emparejar los dispositivos cercanos. Lo que identifica a un dispositivo no se incluye.',
        },
        tresor: {
          title: 'Credenciales y claves en la caja fuerte',
          p1: 'Aquello con lo que este dispositivo se identifica ante Openany y otros dispositivos se guarda cerrado en el dispositivo; la clave queda en el llavero del sistema: el Keystore en Android y, en el ordenador, el llavero de macOS, el administrador de credenciales de Windows o el Secret Service en Linux. Ahí también están las contraseñas de tus buzones y tus claves OpenPGP secretas. «Desconectar» solo quita las credenciales aquí; un dispositivo se revoca definitivamente en «Mis dispositivos».',
        },
      },
    },
    contact: {
      title: 'Denunciar, normas y contacto',
      subs: {
        kontakt: {
          title: 'Contactar con nosotros',
          p1: 'Envía preguntas, problemas o deseos con el formulario de soporte al final de esta ayuda; llega cifrado de extremo a extremo a los administradores, y la respuesta la encuentras en Mensajes. Sin cuenta nos contactas con el formulario de openany.de/denunciar en «Pregunta o consulta». Respondemos en alemán o en inglés.',
        },
        bedingungen: {
          title: 'Condiciones de uso y privacidad',
          p1: 'Las condiciones de uso y la política de privacidad están en «Acerca de openany», debajo del formulario de soporte, y el aviso legal al final de esta ayuda; los tres están también en openany.de. En resumen: openany es gratuito y sin publicidad, tu contenido se cifra en tus dispositivos y los servidores están en Alemania.',
        },
        melden: {
          title: 'Denunciar contenido',
          p1: 'Denuncias contenidos o cuentas que infringen la ley o las condiciones de uso con el formulario de openany.de/denunciar, también sin cuenta. «Abrir formulario» encima del formulario de soporte te lleva allí en el idioma de la app. La denuncia se cifra en tu navegador; solo los administradores pueden leerla.',
          p2: 'Después de enviarla recibes un código de acceso. Con él ves en la misma página el estado y nuestra respuesta y puedes preguntar. Guárdalo bien: sin él ya no llegas a tu denuncia. Se elimina tres meses después del último mensaje.',
        },
        entscheidungen: {
          title: 'Si suspendemos o eliminamos tu cuenta',
          p1: 'Si una cuenta incumple las condiciones de uso, los administradores pueden suspenderla o eliminarla. Si tu cuenta está suspendida, arriba aparece una franja roja; al tocarla ves desde cuándo, en qué se basa la decisión y por qué. La sincronización con openany.de se detiene; lo que hay en tus dispositivos se queda. Cuando se levanta la suspensión, todo sigue como antes.',
          p2: 'Si tu cuenta se elimina, la app muestra una vez el motivo en la siguiente sincronización y luego se desconecta; la app sigue funcionando sin openany.de. Si no estás de acuerdo con una decisión, toca «Recurrir»: se abre el formulario de openany.de/denunciar con «Recurso contra una decisión», y volvemos a revisar la decisión.',
        },
      },
    },
  },
};
