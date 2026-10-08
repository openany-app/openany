// Atención vue-i18n: nada de @, | ni { sin escapar en los textos.
export default {
  title: 'Ayuda',
  intro: 'Aquí encontrarás descripciones e instrucciones sobre todos los módulos y funciones de Openany. Con el índice de la izquierda saltas directamente a la sección que necesitas.',
  version: 'Versión {version}',
  toc: 'Contenido',
  sections: {
    start: {
      title: 'Primeros pasos',
      subs: {
        konto: {
          title: 'Cuenta e inicio de sesión',
          p1: 'Las cuentas de Openany se crean exclusivamente por invitación personal: no hay registro abierto. La dirección de correo es opcional: Openany funciona también sin ella, pero con correo puedes restablecer tú mismo una contraseña olvidada.',
          p2: 'Se inicia sesión con nombre de usuario y contraseña a través de anyid, el servicio común de inicio de sesión. El mismo inicio de sesión vale para Openany, anyitem y anytail; por eso la contraseña y el segundo factor se gestionan allí y no en Openany (consulta «Seguridad» en los ajustes).',
        },
        ohneKonto: {
          title: 'La app sin cuenta',
          p1: 'La app es un programa completo que funciona también sin cuenta y sin servidor: nla vía Openany de los mensajes, por ejemplo, solo aparece cuando el dispositivo está conectado con openany.de.',
          p2: 'Si quieres sincronizar tus datos con Openany en la red, pulsa «Conectar con openany.de» en los ajustes, en «Sincronización»; con tus otros dispositivos te emparejas en «Dispositivos cercanos». Después aparece en la cabecera y en el menú un botón «Sincronizar», que gira mientras sincroniza.',
        },
        startseite: {
          title: 'La página de inicio',
          p1: 'La página de inicio muestra mosaicos compactos de los módulos que hayas elegido: las próximas citas, las notas editadas recientemente, los últimos archivos con la barra de almacenamiento, tus proyectos con contador de no leídos, los mensajes más recientes de la bandeja de entrada y la agenda. Cada mosaico lleva a su módulo; cuáles aparecen lo decides en los ajustes, en «Módulos en la página de inicio».',
          p2: 'Si no has elegido ningún módulo para la página de inicio, aparece en su lugar la bienvenida: presenta todas las funciones y te permite componer los mosaicos directamente. La selección no oculta nada: todas las áreas siguen accesibles desde la barra y el menú.',
        },
        navigation: {
          title: 'Menú y estructura',
          p1: 'En el ordenador hay una cabecera arriba: a la izquierda, el logotipo redondo lleva a la página de inicio y a su lado están Notas, Calendario, Disco y Proyectos. A la derecha siguen el interruptor claro/oscuro, el sobre de los mensajes con contador de no leídos y el campo con tu nombre, que abre Ajustes, Ayuda, Papelera y Cerrar sesión.',
          p2: 'En el móvil todo baja: una barra fija muestra los módulos como iconos y, a la izquierda del todo, está el botón de menú con el contador de mensajes no leídos. Abre un panel con Mensajes, Ajustes, Ayuda, Papelera, el interruptor claro/oscuro y Cerrar sesión. El interruptor alterna entre claro, oscuro y automático; automático sigue el ajuste de tu dispositivo.',
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
          p1: 'Las notas están en carpetas que se pueden anidar sin límite. Puedes renombrar, mover y reordenar notas y carpetas: así crece con el tiempo tu propia colección de conocimiento. El campo de búsqueda sobre la lista busca en todas las notas a la vez, en título y contenido, independientemente de la carpeta abierta.',
          p2: 'Una carpeta completa con sus subcarpetas se descarga como archivo ZIP; las notas van dentro como archivos Markdown y siguen siendo legibles en cualquier parte.',
        },
        wikilinks: {
          title: 'Wikilinks y referencias inversas',
          p1: 'Con dos corchetes enlazas notas entre sí: escribe los corchetes y el título de la nota de destino y el enlace se crea automáticamente; si la nota aún no existe, el enlace se activa en cuanto exista. Si en el texto debe aparecer algo distinto del título, indicas un texto visible propio tras una barra vertical; el enlace sigue apuntando a la nota correcta.',
          p2: 'Cada nota muestra sus referencias inversas: una lista de todas las notas que enlazan a ella. Así vuelves a encontrar las relaciones sin tener que llevar la cuenta tú.',
        },
        tags: {
          title: 'Etiquetas',
          p1: 'Las palabras clave se asignan directamente en el texto con una almohadilla delante de la palabra; al escribir, una lista de sugerencias muestra las etiquetas ya usadas. Con una barra inclinada anidas etiquetas, por ejemplo Proyecto/Subtema; un filtro por el tema principal encuentra también todos los subtemas. En la vista de grafo, las etiquetas se pueden mostrar como nodos propios, y un clic en una etiqueta filtra allí la lista de notas.',
        },
        einfuegen: {
          title: 'Insertar imágenes y archivos',
          p1: 'Con el botón del clip de la barra de herramientas insertas contenido en una nota: recién subido o como referencia a contenido ya guardado en Openany. Las imágenes aparecen como vista previa compacta directamente en la nota (la resolución completa está en la Galería); los demás archivos, como enlace para descargar.',
          p2: 'El contenido insertado se ordena automáticamente: las imágenes en un álbum de la Galería, los archivos de texto y de Office en un expediente de Documentos y todo lo demás en una carpeta de archivos, cada uno en un área propia para notas. Mientras la referencia esté en al menos una nota, el archivo no se puede mover a la papelera desde Disco o Galería; si quitas la referencia del texto, vuelve a poder eliminarse.',
        },
        zitate: {
          title: 'Citas y bibliografía',
          p1: 'Para el trabajo académico puedes asignar a una carpeta un archivo bibliográfico en formato CSL-JSON, por ejemplo una exportación de Zotero que hayas subido antes a Disco. El botón «Vincular biblioteca» de la carpeta establece la conexión; las subcarpetas heredan la biblioteca automáticamente.',
          p2: 'En las notas de esa carpeta, el editor sugiere fuentes adecuadas al citar e inserta una referencia breve en el texto. Así las referencias surgen mientras escribes, sin teclear los datos de la fuente cada vez.',
        },
        graph: {
          title: 'Vista de grafo',
          p1: 'La vista de grafo dibuja tus notas como una red: cada nota es un punto y cada wikilink, una conexión. Los puntos se colorean según la carpeta superior, de modo que las áreas relacionadas se reconocen por el color; los puntos huecos son notas sin ninguna conexión y, cuantas más conexiones tiene una nota, más grande es su punto.',
          p2: 'Cada carpeta de primer nivel es un nivel. Si hay varios, eliges uno o más en la leyenda bajo el grafo: «Todos los niveles» los toma todos, «Ninguno» vacía la selección. En lugar de un nivel, también una etiqueta puede ser la entrada: la barra de debajo enumera las etiquetas con su número, y una etiqueta elegida muestra todas las notas que la llevan, con la etiqueta como rombo gris. Si hay niveles elegidos, la barra solo muestra las etiquetas que aparecen en ellos. Hasta que eliges algo, el grafo queda vacío, para que las colecciones grandes no se construyan de golpe; «Mostrar todas las etiquetas» incorpora todas las de la barra a la vez.',
          p3: 'En el ordenador haces zoom con los botones más/menos de arriba a la derecha o con la rueda del ratón y desplazas la vista manteniendo pulsado el botón; en el móvil haces zoom con dos dedos y desplazas con uno. Otro botón restablece la vista. Un clic en un punto (sin arrastrar) abre la nota. Un clic en una etiqueta filtra la lista de notas por ella.',
        },
        uebersicht: {
          title: 'Índice automático',
          p1: 'Además del grafo hay un índice automático: enumera todos los títulos de notas de la A a la Z y muestra debajo un árbol desplegable de tus etiquetas (las etiquetas anidadas aparecen como ramas). Un clic en un título abre la nota y un clic en una etiqueta filtra la lista, sin que tengas que mantener tú un índice.',
        },
        export: {
          title: 'Exportar a PDF',
          p1: 'Cada nota se puede descargar como PDF, para imprimir, archivar o compartir. El formato del editor se conserva.',
        },
        papierkorb: {
          title: 'Papelera',
          p1: 'Las notas eliminadas van a la papelera (en el menú bajo tu nombre; en el móvil, en el botón de menú) y allí se pueden restaurar o eliminar definitivamente. Al eliminar una carpeta decides si las notas que contiene se eliminan también o se mueven a la carpeta superior.',
        },
      },
    },
    files: {
      title: 'Disco',
      subs: {
        aufbau: {
          title: 'Galería, archivos, documentos',
          p1: 'Disco se divide en tres pestañas en la parte superior: la Galería para fotos y vídeos, Archivos para todo lo demás y Documentos para papeles en expedientes. Al cambiar de pestaña, la otra zona queda como la dejaste: las carpetas abiertas y las listas cargadas no se pierden.',
          p2: 'Qué pestaña se abre primero al entrar en Disco lo decides en los ajustes, en «Inicio del disco»; mientras no elijas nada, es Documentos.',
        },
        dokumente: {
          title: 'Archivo de documentos',
          p1: 'En la pestaña «Documentos» organizas archivos PDF, de Office y de texto en expedientes. Se suben con el botón o simplemente arrastrando los archivos al expediente abierto. En móviles y tabletas hay al lado un botón de cámara: fotografía un papel y lo guarda directamente como documento en el expediente.',
          p2: 'Los expedientes se pueden anidar sin límite. Puedes renombrar y mover tanto expedientes como documentos sueltos; los expedientes completos, además, se pueden compartir en proyectos y descargar como ZIP.',
        },
        texterkennung: {
          title: 'Documentos con búsqueda',
          p1: 'Si fotografías un documento en papel y lo guardas en un expediente –con el botón de cámara o como foto de tu colección de imágenes–, se convierte automáticamente en un PDF con una capa de texto invisible. El documento se ve como tu foto, pero se puede buscar, marcar, copiar y leer en voz alta. El reconocimiento se ejecuta en tu propio dispositivo: la imagen no se envía a ningún sitio.',
          p2: 'Si guardas varias fotos a la vez, se te pregunta si deben formar un documento de varias páginas –como las páginas de una carta– o documentos separados. La foto en sí no se guarda aparte; si quieres conservar la imagen, guárdala en la Galería. La primera vez la conversión tarda más, porque el reconocimiento de texto se carga una sola vez y después queda guardado.',
          p3: 'Si ya hay un PDF escaneado en un expediente, puedes aplicarle el reconocimiento después: «Reconocer texto» lee el documento página a página. Si el papel es blanco y limpio, las páginas no cambian y solo se añade el texto. Si están grises o iluminadas de forma irregular –como una hoja fotografiada–, además se aclaran; el mensaje final te dice qué ha pasado. Si el PDF ya tenía búsqueda, no se toca.',
        },
        dateien: {
          title: 'Gestionar archivos',
          p1: 'En la pestaña «Archivos» creas carpetas y subes archivos de cualquier tipo, uno a uno o varios a la vez. Carpetas y archivos se pueden renombrar y mover, y los archivos volver a descargar; las carpetas también se pueden compartir en proyectos y descargar como ZIP.',
          p2: 'Cada cuenta tiene una cuota de almacenamiento; el uso actual lo ves en el mosaico de la página de inicio y en los ajustes, en «Espacio de almacenamiento».',
        },
        suche: {
          title: 'Buscar',
          p1: 'La lupa de Archivos y Documentos abre un campo de búsqueda que recorre todas las carpetas o expedientes a la vez, no solo el que está abierto. Un clic en un resultado abre el archivo; al lado ves dónde está, y un clic en esa indicación te lleva a la carpeta.',
          p2: 'La búsqueda no solo encuentra por nombre, sino también en el texto: en PDF, textos de Word y OpenDocument y archivos de texto, CSV y Markdown de hasta 50 MB. Openany lee ese texto poco a poco en segundo plano mientras Disco está abierto; hasta que lo haya leído todo, la búsqueda te dice cuántos archivos de momento solo se encuentran por nombre. Si el resultado está en el texto, la lista muestra el pasaje con la palabra buscada resaltada. Los PDF escaneados sin capa de texto solo aparecen después de aplicarles «Reconocer texto».',
        },
        pdf: {
          title: 'Ver y editar PDF',
          p1: 'Un clic en un PDF lo abre directamente en Openany, sin descargarlo. Pasas las páginas, amplías o reduces, ajustas la vista al ancho y buscas en el documento con la lupa: los resultados se resaltan en la página. Si un PDF está protegido con contraseña, Openany te la pide primero.',
          p2: 'Los formularios del PDF los rellenas directamente. Con «Editar» aparece además una barra de herramientas: «Resaltar» marca texto en color, con el «Lápiz» dibujas libremente sobre la página y con «Texto» colocas tus propias líneas en la hoja; color, grosor del trazo y tamaño de letra los eliges tú, y cada paso se puede deshacer. Eso sí, los campos de cálculo de los formularios no calculan, porque Openany no ejecuta scripts de archivos ajenos.',
          p3: 'A los resaltados y trazos les puedes añadir comentarios: elige «Comentarios» y toca el lugar; esa misma vista lista todos los comentarios del documento. Todo se guarda como anotaciones PDF normales, que también muestran otros programas. Al guardar por primera vez, Openany pasa la versión anterior a la papelera, para que puedas volver al original; si sales del PDF con cambios sin guardar, te pregunta antes.',
        },
        galerie: {
          title: 'Galería de fotos y álbumes',
          p1: 'En la pestaña «Galería» están tus fotos y vídeos: en álbumes con miniaturas o, sin álbum, debajo, en «Imágenes». Los álbumes se pueden anidar –por ejemplo un álbum por viaje con un subálbum por día– y mover en cualquier momento. El visor recorre todas las imágenes de un álbum.',
          p2: 'La toma más reciente aparece arriba, agrupada por meses con un encabezado para cada mes. Cuenta la fecha de captura de los datos de la foto; si falta –por ejemplo en capturas de pantalla o imágenes editadas–, cuenta el día en que se subió.',
          p3: 'Los álbumes completos se descargan como ZIP y las imágenes sueltas como archivo original. Los álbumes también se pueden compartir en proyectos.',
        },
        videos: {
          title: 'Vídeos y cámara',
          p1: 'Además de fotos, la Galería admite vídeos. Aparecen como miniatura con un fotograma y se reproducen en el visor. Algunos móviles graban en formato HEVC, que no todos los navegadores pueden reproducir; en ese caso descarga el vídeo y ábrelo con otro programa.',
          p2: 'En móviles y tabletas, la Galería y cada álbum tienen dos botones de cámara: uno para foto y otro para vídeo. La grabación va directamente al lugar que tienes abierto. En el ordenador no hay botones; allí subes como siempre.',
        },
        karte: {
          title: 'Fecha y lugar de captura',
          p1: 'Para cada imagen, el visor muestra la fecha de captura y, si la foto lo contiene, el lugar; «Ver en el mapa» abre el sitio en OpenStreetMap. Si subes fotos desde un móvil Android, elígelas con la app Archivos y no con la Galería: si no, Android elimina la ubicación antes de que la imagen llegue a Openany.',
        },
        papierkorbSpeicher: {
          title: 'Papelera',
          p1: 'El contenido eliminado –documentos, archivos, fotos, notas, contactos, pero también proyectos, tableros, calendarios y citas– pasa primero a la papelera común y allí se puede restaurar. A los 30 días la papelera se vacía automáticamente; solo entonces el espacio queda libre definitivamente.',
        },
      },
    },
    calendar: {
      title: 'Calendario',
      subs: {
        verwalten: {
          title: 'Varios calendarios',
          p1: 'Puedes llevar tantos calendarios como quieras a la vez –por ejemplo personal, familia, asociación–, cada uno con su color. Muestras u ocultas cada calendario con un clic. Arriba cambias entre vista mensual y semanal; la semana muestra las citas según la hora, y las citas pasadas aparecen atenuadas discretamente.',
        },
        termine: {
          title: 'Crear citas',
          p1: 'Las citas se crean con un clic en un día: con título, descripción y hora, o como evento de todo el día o de varios días. También se pueden repetir: diaria, semanal, mensual o anualmente. Las citas existentes se editan o eliminan directamente desde la vista. Las horas se refieren a la zona horaria configurada en tu perfil.',
        },
        tagesansicht: {
          title: 'Vista de día',
          p1: 'Un clic en el número de un día abre la vista de día: un repaso en lugar de una cuadrícula, pensado para la pregunta de la víspera: ¿qué necesita el niño mañana? Arriba del todo, el plan de cuidado indica con quién está («contigo» o con quién si no). Debajo siguen por orden las clases del horario, cada una con lo que hay que llevar hecho; los exámenes y otros plazos de los próximos días aparecen ya por adelantado con «en … días». Con las flechas pasas al día anterior o al siguiente.',
          p2: 'En cada clase anotas rápidamente unos deberes con «Deberes»; se guardan como tarjeta con la asignatura en el tablero del proyecto. «Cuaderno» abre la carpeta de la asignatura. Lo que vence pero no pertenece a ninguna clase aparece en «Sin hora de clase». Un clic en una entrada te lleva a su lugar en el proyecto.',
        },
        ausProjekten: {
          title: 'De proyectos',
          p1: 'En «De proyectos», en la lista de calendarios, muestras lo que tus proyectos aportan al calendario: horario escolar, cuidado y vencimientos –tarjetas e hitos con plazo–. Cada fuente se puede mostrar u ocultar por separado, y los planes nuevos aparecen solos. Las entradas son solo de lectura; un clic muestra los detalles y «Abrir en el proyecto» te lleva a donde se cambian.',
        },
        abos: {
          title: 'Suscripciones de calendario',
          p1: 'Los calendarios externos –por ejemplo, festivos o el calendario de partidos del club– se integran mediante una URL de suscripción en formato ICS. Los calendarios suscritos se actualizan automáticamente. Sus citas en Openany son solo de lectura: se cambian allí de donde viene el calendario, porque cada actualización vuelve a sustituir el contenido por el original.',
          p2: 'A la inversa, puedes compartir tus calendarios con un enlace de suscripción: el icono de feed en la lista de calendarios crea una URL secreta a la que se puede suscribir en Google, Apple u Outlook, y los cambios llegan allí automáticamente. Trata el enlace como una contraseña; con «Renovar enlace» el anterior deja de valer al instante.',
        },
        importExport: {
          title: 'Importar y exportar',
          p1: 'Los calendarios existentes se importan como archivo ICS y, del mismo modo, exportas tus calendarios: portabilidad total en ambas direcciones, sin atarte a Openany.',
        },
      },
    },
    projects: {
      title: 'Proyectos',
      subs: {
        grundlagen: {
          title: 'Proyectos y miembros',
          p1: 'Un proyecto es un espacio de trabajo compartido: invitas a otros usuarios como miembros y ellos aceptan o rechazan la invitación. El propietario del proyecto gestiona a los miembros y también puede ceder el proyecto a otro miembro.',
          p2: 'Cada proyecto tiene cuatro pestañas: Chat, Planificación, Miembros y Comparticiones. Los números de las pestañas indican los mensajes de chat no leídos o cuánto contienen.',
        },
        chat: {
          title: 'Chat del proyecto',
          p1: 'Cada proyecto tiene un chat común en tiempo real: los mensajes aparecen al instante para todos los miembros, sin recargar la página.',
          p2: 'Cualquier miembro puede fijar mensajes importantes: los mensajes fijados se encuentran rápido en la barra de fijados sobre el historial y allí también se pueden soltar.',
          p3: 'Con dos corchetes haces referencia, en mitad de una frase, a contenido del proyecto: notas, tableros, hojas de ruta y colecciones de lugares con sus tarjetas, hitos y lugares, así como carpetas, archivos, álbumes e imágenes compartidos. Al escribir, una lista de sugerencias muestra lo que encaja e inserta la referencia terminada. El botón con el icono de cadena junto al botón de enviar hace lo mismo; en el móvil suele ser más cómodo que buscar dos corchetes.',
          p4: 'Un clic en una referencia lleva directamente al destino: a la nota, al tablero con la tarjeta abierta, a la carpeta con el archivo resaltado o a la imagen en el visor. Solo puedes hacer referencia a lo que ya es visible en el proyecto: lo que nadie ha compartido no aparece en la lista de sugerencias ni se puede pulsar.',
        },
        planung: {
          title: 'La pestaña Planificación',
          p1: 'En la pestaña «Planificación» el proyecto reúne sus herramientas de planificación. Con el botón «Nuevo» eliges el tipo adecuado: Buscar fecha, Encuesta, Turnos, Lista de qué traer, Lista de invitados, Tablero, Hoja de ruta, Lugares, Vacaciones escolares, Horario escolar o Plan de cuidado. La plantilla «Preajuste: colegio» crea de una vez vacaciones escolares, horario, exámenes y un tablero de deberes. Al hacerlo eliges tu estado federado, para que las vacaciones ya estén puestas, y si quieres el horario de timbres de tu colegio. Cualquier miembro puede crear y editar; eliminar puede quien lo creó o el propietario del proyecto.',
        },
        terminfindung: {
          title: 'Buscar fecha',
          p1: 'Al buscar fecha propones varias opciones y todos votan sí, no o quizá: así quedáis sin tanto ir y venir.',
        },
        umfragen: {
          title: 'Encuestas',
          p1: 'Una encuesta plantea una pregunta con respuestas fijas: sí/no o propias, anónima si quieres. Con opciones marcables sirve también como lista de tareas común; las votaciones se pueden duplicar y cerrar.',
        },
        listen: {
          title: 'Turnos, lista de qué traer, lista de invitados',
          p1: 'Tres tipos de lista quitan trabajo de organización: en Turnos creas turnos con plazas y los miembros se apuntan. La lista de qué traer aclara quién trae qué; las entradas se pueden asumir y ampliar. La lista de invitados controla a los invitados (invitado, acepta, rechaza), también a quienes no tienen cuenta en Openany.',
        },
        boards: {
          title: 'Tableros Kanban',
          p1: 'En los tableros Kanban organizáis las tareas en columnas y tarjetas libres, por ejemplo «Pendiente», «En curso», «Hecho». Las tarjetas se mueven arrastrándolas; los cambios aparecen en directo para todos los miembros. Una tarjeta puede llevar además un lugar de las colecciones de lugares del proyecto y una asignatura: así unos deberes quedan ligados a la clase correspondiente.',
        },
        roadmap: {
          title: 'Hoja de ruta',
          p1: 'La hoja de ruta muestra hitos con fecha y estado en una línea de tiempo: así todo el equipo ve qué debe estar listo y cuándo. Los hitos se pueden marcar como alcanzados y los vencidos quedan señalados. Igual que las tarjetas, los hitos pueden llevar un lugar y una asignatura, por ejemplo un examen.',
        },
        orte: {
          title: 'Lugares',
          p1: 'En Lugares reunís puntos en OpenStreetMap: puntos de encuentro, direcciones, aparcamientos. El punto se coloca con un clic en el mapa, con nombre y nota opcionales; un clic en el nombre de un mosaico de lugar abre el punto directamente en OpenStreetMap. Una vez reunidos, eliges los lugares en otros sitios sin más: en tarjetas, hitos, asignaturas y entradas del horario o del plan de cuidado.',
        },
        schuljahr: {
          title: 'Vacaciones escolares',
          p1: 'Las vacaciones escolares recogen un curso: el periodo y los días libres. Las vacaciones de tu estado federado (Alemania) se pueden importar como archivo ICS; los días de formación y festivos sueltos los añades a mano. La diferencia importa: en vacaciones suele valer otro acuerdo de cuidado, mientras que en un día libre suelto sigue el ritmo habitual. La importación solo añade y no sustituye nada de lo que hayas introducido tú.',
        },
        wochenplan: {
          title: 'Horario escolar y plan de cuidado',
          p1: 'Ambos son la misma pieza: una cuadrícula semanal de lunes a domingo que se repite semana tras semana. El horario escolar lleva horas y asignaturas; el plan de cuidado, días completos y una persona: «quién tiene al niño y cuándo». Si lo vinculas con unas vacaciones escolares, las clases se saltan solas en vacaciones; un plan de cuidado se puede limitar al periodo lectivo o a las vacaciones. Si vuestro ritmo cambia cada semana, pasas a semanas A/B: o bien cada dos semanas a partir del lunes de la semana A, o bien por semanas naturales pares e impares si vuestro acuerdo lo dice así; en ese caso, eso sí, el ritmo salta en los años con 53 semanas.',
          p2: 'Debajo de la cuadrícula están los tramos. Sustituyen la cuadrícula en su periodo y cubren lo que no es ritmo: las vacaciones de verano repartidas a medias, un fin de semana intercambiado, una excursión escolar. La zona horaria pertenece al plan y no a quien lo mira: así el horario muestra la misma hora a ambos progenitores, aunque uno viva en el extranjero.',
        },
        faecher: {
          title: 'Asignaturas',
          p1: 'Las asignaturas de un proyecto las gestionas desde «Asignaturas» en el horario escolar: nombre, abreviatura, docente, color y, si quieres, un lugar. Pertenecen al proyecto y no a un plan concreto, así que sobreviven al cambio de semestre. Una asignatura determina lo que muestra una clase en el horario y vincula tarjetas de deberes y exámenes con la clase correspondiente. Si borras una asignatura, tarjetas, hitos y clases solo pierden la referencia; ellos se mantienen.',
        },
        hefte: {
          title: 'Cuadernos de las asignaturas',
          p1: 'Para cada asignatura, «Crear carpeta y compartir» crea una carpeta de notas –el cuaderno– y la comparte de inmediato en el proyecto. La carpeta es tuya, no del proyecto, y cuenta para tu espacio; por eso la asignatura indica de quién es la carpeta. Un proyecto no posee contenido propio: lo que hay en él lo han compartido sus miembros.',
        },
        planAbo: {
          title: 'Suscribirse al horario y al cuidado',
          p1: 'Con «Suscripción» generas un enlace con el que el horario o el plan de cuidado se pueden suscribir en Google, Apple o Thunderbird, sin abrir Openany. El enlace vale para el tipo, no para un plan concreto: si lleváis el cuidado en vacaciones como segundo plan, está en la misma suscripción. Los lugares solo se incluyen si lo pides expresamente: el enlace es público y no requiere inicio de sesión, y con lugares estarían vuestras direcciones detrás. Los programas de calendario solo recogen la suscripción cada pocas horas; «Renovar» invalida el enlace anterior al instante, «Revocar» lo desactiva.',
        },
        freigaben: {
          title: 'Compartir contenido',
          p1: 'En la pestaña «Comparticiones» está todo lo que los miembros han puesto a disposición del proyecto: carpetas de notas, álbumes y carpetas y expedientes de Disco. Se comparte desde donde está el contenido –en Notas, en la Galería o en Disco–, con el nivel «Solo lectura» o «Editar». Una compartición vale para todos los miembros del proyecto, se hereda a subcarpetas y subálbumes y se puede quitar en cualquier momento. El contenido sigue siendo del propietario; si termina la compartición, lo conserva todo.',
          p2: 'Un clic en una compartición la abre en el proyecto: las notas en el espacio de trabajo habitual, con búsqueda, referencias inversas y un grafo de todas las notas compartidas del proyecto; los álbumes como cuadrícula de fotos con visor; las carpetas como lista de archivos. Con «Editar», los miembros también pueden aportar: escribir notas, subir fotos, guardar archivos y crear subcarpetas; «Solo lectura» permite ver y descargar.',
        },
        projektNotizen: {
          title: 'Notas de proyecto compartidas',
          p1: 'En una carpeta de notas compartida con «Editar», los miembros pueden crear, editar y eliminar notas directamente en el proyecto, así como crear subcarpetas; las imágenes y archivos incrustados los ven todos los miembros, y quien puede editar también puede insertar los suyos. Todo lo creado –también imágenes y archivos subidos– pertenece al propietario de la carpeta y cuenta en su almacenamiento; las notas eliminadas van a su papelera, así que nada se pierde definitivamente.',
        },
        ki: {
          title: 'IA en el proyecto',
          p1: 'A un proyecto se le puede conectar una IA. Lo configura el propietario del proyecto en la pestaña «Miembros», bajo la lista de miembros: proveedor, modelo y una clave de API propia. La clave se guarda cifrada y después no se muestra a nadie, tampoco al propietario; los costes corren a cargo de la cuenta a la que pertenece la clave. En el mismo lugar, todos los miembros ven si hay una IA conectada, con qué proveedor y quién la configuró.',
          p2: 'Se pregunta desde contenido compartido: bajo una nota con «Preguntar a la IA», en documentos de expedientes y carpetas compartidos (PDF, Word, OpenDocument y archivos de texto) y –si el modelo entiende imágenes– en fotos del visor de un álbum compartido. Escribes una petición, por ejemplo «Resúmelo en tres frases», y recibes la respuesta para copiarla; quien puede editar la nota también puede insertarla directamente al final.',
          p3: 'Importante: lo que preguntas sale de Openany. Sobre el campo de entrada se indica cada vez qué se envía y a qué proveedor: todo el texto de la nota, el texto del documento o la foto, reducida y sin lugar de captura. Los PDF escaneados sin capa de texto necesitan antes el reconocimiento de texto; los documentos muy largos se rechazan.',
        },
        benachrichtigungen: {
          title: 'Avisar a los miembros',
          p1: 'Con nuevas encuestas, búsquedas de fecha o comparticiones puedes avisar a los miembros por mensaje directo si quieres: así nadie se pierde las novedades del proyecto.',
        },
        lokal: {
          title: 'Proyectos solo en persona',
          p1: 'En la app, «+ Proyecto» crea un proyecto solo en este dispositivo, sin cuenta y sin servidor; lleva la marca «Solo en persona». A otras personas las invitas en «Invitar a un miembro cercano»: en el otro dispositivo Openany tiene que estar abierto, en la misma wifi o punto de acceso, y ambos dispositivos muestran los mismos seis dígitos, que comparáis y confirmáis. El proyecto lo gestionan los dispositivos del propietario; si solo hay uno, la app avisa: empareja antes un segundo dispositivo propio.',
          p2: 'Los miembros comparten sus propias carpetas de notas, carpetas y álbumes en el proyecto con «Compartir algo», al principio solo para leer. Las carpetas de notas también se pueden compartir para editar: entonces solo una persona edita una nota a la vez, y se guarda en el dispositivo de la persona a la que pertenece la carpeta; si ese dispositivo no está cerca, la nota queda solo de lectura. El chat llega al instante a todos los que están cerca, y a los demás en la siguiente sincronización. La planificación –tableros, hojas de ruta, lugares y la planificación escolar– viaja de la misma manera.',
          p3: 'El resto lo sincronizas con «Sincronizar» en cuanto haya un miembro cerca. El propietario puede quitar miembros; quien quiera irse sale con «Salir», y eso solo funciona si otro miembro está cerca para enterarse. Lo que habían compartido quienes se van desaparece para los demás; lo que ellos ya tenían se queda en sus dispositivos.',
        },
      },
    },
    messages: {
      title: 'Mensajes',
      subs: {
        direkt: {
          title: 'Mensajes directos',
          p1: 'En el ordenador llegas a los mensajes por el sobre de la cabecera; en el móvil, por el botón de menú abajo a la izquierda. Con «Mensaje» arriba a la derecha escribes directamente a cualquier otro usuario: como destinatario basta su nombre en Openany. Los mensajes nuevos aparecen al instante, sin recargar la página, y los enlaces que contienen se pueden pulsar.',
          p2: 'Bajo cada mensaje recibido, «Responder» abre el campo de escritura con el remitente ya puesto, por la misma vía por la que llegó el mensaje. Los mensajes largos aparecen plegados; «Seguir leyendo» los muestra enteros. El contador de no leídos te indica en todo momento si hay algo nuevo: en el ordenador, en el sobre; en el móvil, en el botón de menú de la barra inferior.',
        },
        suche: {
          title: 'Buscar y filtrar',
          p1: 'Encima del historial hay un campo de búsqueda. Encuentra mensajes por su texto, por el nombre de la otra persona y por identificadores de Matrix, y resalta las coincidencias. Debajo, unos interruptores acotan la lista: a los mensajes no leídos, a los que tienen adjunto y –si usas más de una vía– a vías concretas como Openany o Matrix. Los interruptores se pueden combinar.',
          p2: 'Junto al campo de búsqueda cambias entre dos vistas: «Historial» muestra todos los mensajes por orden de tiempo, «Por contacto» los resume en una fila por persona, con el último mensaje y el número de no leídos. Un clic en una fila muestra el historial con esa persona; «Cerrar conversación» te devuelve a todos.',
        },
        anhaenge: {
          title: 'Adjuntos',
          p1: 'Por Matrix también envías archivos: el clip del campo de escritura adjunta un archivo de hasta 10 MB, y en móviles y tabletas el botón de cámara de al lado hace directamente una foto. En el historial, las imágenes aparecen como vista previa y los demás archivos con nombre y tamaño; un clic abre imágenes y PDF directamente en Openany, lo demás lo descargas. Por la vía Openany solo van textos.',
        },
        matrix: {
          title: 'Mensajes por Matrix',
          p1: 'A las personas sin cuenta en Openany las alcanzas por Matrix. Para ello conectas en los ajustes, en «Cuenta de Matrix», una cuenta existente de matrix.org u otro homeserver; tienes que crearla allí. La contraseña solo se usa para iniciar sesión y no se guarda. Openany inicia sesión como dispositivo propio y desde entonces puede leer todos los mensajes nuevos de esa cuenta, también los que escribes en otro programa de Matrix; hacia fuera todo sigue cifrado.',
          p2: 'Con una cuenta conectada, al escribir eliges la vía: Openany o Matrix, y en ese caso el identificador de Matrix como destinatario. Las respuestas llegan al mismo historial y van marcadas con «Matrix». Si borras un mensaje de Matrix, desaparece solo en Openany; en la sala sigue estando. Con «Desconectar» das de baja el dispositivo; los mensajes ya recibidos se conservan.',
        },
        matrixApp: {
          title: 'Mensajes por Matrix',
          p1: 'En la app, tu propio dispositivo es el dispositivo de Matrix. Inicias sesión con una cuenta existente en los ajustes, en «Cuentas de Matrix», y tus mensajes van cifrados de extremo a extremo entre este dispositivo y la otra persona: openany.de no los lee. La verificación del dispositivo por comparación de emojis aún no la sabe hacer la app; por eso otros programas de Matrix la muestran como dispositivo no verificado, pero el cifrado funciona igual.',
          p2: 'Puedes conectar varias cuentas de Matrix. Sus mensajes comparten historial; los nuevos salen de la cuenta predeterminada, las respuestas de la cuenta en la que llegó el mensaje, y «Hacer predeterminado» convierte otra cuenta en la predeterminada. Si borras un mensaje de Matrix, desaparece solo en este dispositivo; en la sala sigue estando.',
        },
        email: {
          title: 'Correo con tu buzón',
          p1: 'En la app se suma el correo electrónico como otra vía: no un programa de correo propio, sino tu buzón de siempre, por ejemplo en Posteo, mailbox.org o GMX. En los ajustes, en «Buzones de correo», introduces dirección y contraseña; normalmente es una contraseña de aplicación que creas antes en tu proveedor. La app busca los servidores por sí misma; si no los encuentra, los introduces en «Servidores a mano». La contraseña queda guardada bajo llave en este dispositivo, y el propio dispositivo habla con el servidor de correo: openany.de no ve ningún correo.',
          p2: 'Los correos aparecen con su asunto en el historial común de Mensajes; «Recoger ahora» trae los nuevos al momento. Si conectas varios buzones, los correos nuevos salen del buzón predeterminado y las respuestas del buzón al que llegó el correo; al escribir eliges en «De». Al borrar decides si un correo solo desaparece aquí o pasa también a la carpeta de papelera del servidor. Si tu proveedor ha clasificado algo como spam, aparece un aviso encima del historial; allí, «No es spam» devuelve el correo a la bandeja de entrada.',
          p3: 'Por correo, los adjuntos pueden ocupar hasta 15 MB. Los adjuntos recibidos los guardas en tus archivos con «Guardar» o en la carpeta de descargas con «En el dispositivo». Si un adjunto llegó cifrado, la app pregunta antes: en Archivos queda sin cifrar y se sincroniza con openany.de.',
        },
        pgp: {
          title: 'Correo cifrado con OpenPGP',
          p1: 'Puedes cifrar correos de extremo a extremo con OpenPGP. Para ello cada buzón necesita su propia clave: en los ajustes, junto al buzón, la creas con «Crear clave»; o, si ya usas el buzón con PGP –por ejemplo en Thunderbird–, importas la existente con «Importar clave», para que ambos programas lean los mismos correos. La clave secreta está solo en este dispositivo. «Guardar copia de seguridad» la guarda, cerrada con una frase de contraseña, en la carpeta de descargas; «Guardar clave pública» te da el archivo que envías a los demás.',
          p2: 'Las claves públicas de tus contactos las reúne la app por sí sola en «Claves de los contactos», a partir de los correos que las incluyen. Si falta alguna, la buscas por la dirección –si quieres, también en keys.openpgp.org, que entonces sabrá por quién preguntas– o la importas desde un archivo. Lo mejor es comparar una vez la huella con la otra persona; si una clave cambia, la app te avisa.',
          p3: 'Al escribir activas «Enviar cifrado»; debajo verás si se conoce la clave del destinatario. Si respondes a un correo cifrado, el interruptor ya está activado. En el historial, los correos llevan la marca «cifrado» y, si están firmados, «firmado», o un aviso si la firma no es válida.',
        },
        vorOrt: {
          title: 'Mensajes cerca',
          p1: 'Con la vía «Cerca» escribes directamente a dispositivos próximos, sin ningún servidor. En el otro dispositivo Openany tiene que estar abierto, y ambos deben estar en la misma wifi. Podéis escribiros en cuanto os conozcáis: tus propios dispositivos, miembros de un proyecto común o personas que hayáis confirmado en persona. Para ello eliges «Confirmar en persona», ambos dispositivos muestran los mismos 6 dígitos y los dos pulsáis «Coincide». Si el otro dispositivo no está en ese momento, el mensaje espera y se envía en el siguiente encuentro.',
          p2: 'De los desconocidos, al principio, no llega nada. Si en los ajustes, en «Dispositivos cercanos», activas «Permitir solicitudes de dispositivos cercanos», podrán enviarte hasta 3 mensajes de 500 caracteres cada uno; aparecen en «Solicitudes» encima del historial, no en el historial. Solo podrás responder tras confirmaros con los 6 dígitos. A quien no deba enviarte nada más lo frenas con «Bloquear». Si uno de tus mensajes fue rechazado, muestra «no entregado», y el motivo aparece al señalarlo.',
        },
        systemnachrichten: {
          title: 'Avisos y respuestas',
          p1: 'Los eventos de tus proyectos –por ejemplo, nuevas encuestas o comparticiones– te llegan como mensaje directo automático, siempre que quien los envía avise a los miembros. Aquí encuentras también la respuesta a un mensaje enviado con el formulario de contacto.',
        },
      },
    },
    contacts: {
      title: 'Agenda',
      subs: {
        adressbuch: {
          title: 'Contactos y sus vías',
          p1: 'La agenda se abre arriba en la página de mensajes; el botón de al lado crea directamente un contacto nuevo. Un contacto recoge cómo se puede localizar a alguien: nombre, imagen, teléfono y correo, además de tantas vías como quieras –Matrix, Meshtastic, nombre en Openany, dirección postal, empresa, cumpleaños u otros–, cada una con su etiqueta, como personal o trabajo. El campo de búsqueda encuentra contactos por nombre.',
          p2: 'Las vías se pueden pulsar: un número de teléfono llama, una dirección de correo abre el programa de correo, y un nombre en Openany o un identificador de Matrix abre el campo de escritura con el destinatario ya introducido. Un contacto puede estar vinculado a una cuenta de Openany: es solo una referencia y no da acceso a proyectos ni a contenidos. Los contactos eliminados van a la papelera.',
        },
        adressbuchDatei: {
          title: 'Guardar y traer la agenda',
          p1: 'En los ajustes, en «Agenda», descargas todos los contactos como archivo vCard o importas un archivo vCard de otro programa. Importar solo añade: los contactos existentes se reconocen por su identificador o su nombre y conservan lo que tienen. Todos los datos viajan en campos estándar; como mucho se pueden perder las etiquetas propias, porque muchas agendas solo conocen «personal» y «trabajo».',
        },
      },
    },
    settings: {
      title: 'Ajustes y cuenta',
      subs: {
        profil: {
          title: 'Perfil, zona horaria y almacenamiento',
          p1: 'Tu nombre en Openany es fijo y no se puede cambiar. La dirección de correo es opcional y solo sirve para fines de la cuenta, como restablecer la contraseña, nunca para publicidad; solo puedes cambiarla con tu contraseña. La zona horaria determina cómo se entienden las horas de tus citas; un botón toma la zona del dispositivo. Debajo, «Espacio de almacenamiento» muestra cuánto ocupas.',
        },
        module: {
          title: 'Módulos en la página de inicio',
          p1: 'Aquí eliges qué módulos aparecen como mosaico en la página de inicio: Notas, Calendario, Disco, Proyectos, Mensajes y Agenda. La selección no oculta nada; todos los módulos siguen accesibles. Sin selección, la página de inicio muestra la bienvenida.',
        },
        sicherheit: {
          title: 'Seguridad: contraseña y segundo factor',
          p1: 'Contraseña, segundo factor y tus dispositivos están en anyid, no en Openany. La tarjeta «Cuenta y seguridad» de los ajustes te lleva allí directamente con tres enlaces: «Cambiar la contraseña», «Segundo factor» y «Tus dispositivos», donde ves los dispositivos vinculados y puedes echarlos. Los cambios valen a la vez para Openany, anyitem y anytail.',
          p2: 'El segundo factor es un código de una app de autenticación que se pide después de la contraseña; sin él no se puede crear ninguna contraseña de aplicación. Al configurarlo recibes códigos de recuperación; guárdalos: sin ellos y sin la app no podrás volver a entrar. Si tienes una dirección de correo guardada, puedes restablecer tú mismo una contraseña olvidada.',
        },
        spracheDesign: {
          title: 'Apariencia e idioma',
          p1: 'En «Apariencia» eliges entre dos diseños: «Nítido», en petróleo con esquinas más marcadas y superficie tranquila, o «Lila», con esquinas redondeadas y fondo con patrón. Ambos existen en claro y oscuro; eso lo cambias por separado con el interruptor de la cabecera o del menú del móvil. En «Idioma» eliges en qué idioma te habla Openany: alemán, inglés, español, francés, portugués, polaco o ucraniano. Mientras no elijas, sigue el idioma de tu dispositivo o navegador; si no está disponible, Openany habla inglés.',
        },
        zugriff: {
          title: 'Acceso externo: contraseñas de aplicación',
          p1: 'Los programas de tu ordenador no inician sesión con la contraseña de tu cuenta, sino con una contraseña de aplicación propia. Al crearla le das un nombre y eliges el uso: «Programas» para WebDAV, Zettlr o un gestor de archivos que leen y escriben tus notas, o «Sincronización» para programas que reflejan tu cuenta y escriben de vuelta. Solo se puede crear si has iniciado sesión con segundo factor.',
          p2: 'La contraseña nueva se muestra exactamente una vez: cópiala de inmediato. En el programa introduces tu nombre en Openany como nombre de usuario; la dirección WebDAV está en la indicación bajo la contraseña. La lista muestra cuándo se usó cada contraseña por última vez; si revocas una, el programa pierde el acceso al instante.',
        },
        abgleich: {
          title: 'Sincronizar con Openany',
          p1: 'En «Sincronización» conectas la app con tu cuenta mediante «Conectar con openany.de». La app muestra un código que tecleas y confirmas en la página de anyid en el navegador. Esa confirmación en el navegador no es un rodeo, sino la prueba de que de verdad eres tú quien añade el dispositivo. Conectar es opcional: sin conexión todo se queda en este dispositivo, y «Desconectar» lo deshace en cualquier momento sin que aquí se pierdan datos.',
          p2: 'Después, «Sincronizar» sincroniza en ambos sentidos y luego te dice qué se ha traído, enviado u omitido. «Volver a traerlo todo» vuelve a descargarlo todo en lugar de solo lo nuevo; no se borra nada.',
          p3: 'Con «Actualizar en segundo plano» la app sincroniza sola más o menos cada hora, solo con conexión y no con la batería baja. Con datos móviles solo llega la lista; archivos e imágenes, solo con wifi. Los dispositivos cercanos no entran; para ellos la app tiene que estar abierta.',
        },
        geraeteNah: {
          title: 'Dispositivos cercanos',
          p1: 'Tus propios dispositivos también se sincronizan directamente entre sí, sin servidor, siempre que estén en la misma wifi o punto de acceso y Openany esté abierto en ambos. En «Dispositivos cercanos» los encuentras con «Buscar» y los conectas con «Emparejar»: los dos muestran el mismo código y en ambos confirmas que coincide. A partir de entonces se sincronizan entre sí en cada «Sincronizar»; «Olvidar» deshace el emparejamiento.',
          p2: 'En «Tu nombre» decides cómo apareces ante los demás en persona. Con ese nombre te invitan a proyectos –a ti como persona, no a un dispositivo concreto–, y vale para todos tus dispositivos emparejados.',
        },
        speicherGeraet: {
          title: 'Almacenamiento en este dispositivo',
          p1: 'En «Almacenamiento en este dispositivo» decides cuánto de tus archivos e imágenes guarda la app. «Según se necesite» muestra todos los archivos en la lista, pero trae el contenido solo al abrirlo. Con «Carpetas y álbumes elegidos» se queda siempre en el dispositivo lo que marcaste en Disco con «Conservar en este dispositivo», subcarpetas incluidas. «Conservarlo todo» trae tras cada sincronización lo que falte, mientras haya espacio.',
        },
        sofort: {
          title: 'Avisar al instante',
          p1: 'Con «Avisar al instante» la app mantiene conexiones ligeras con Openany y con tus buzones de correo, para que los mensajes y correos nuevos lleguen al momento, sin Google. Para los buzones no hace falta Openany. Android muestra por ello un aviso permanente, que puedes ocultar en los ajustes del sistema.',
        },
        sicherung: {
          title: 'Copia de seguridad en un archivo',
          p1: 'En «Copia de seguridad», «Crear copia» genera un archivo cifrado con todo lo que Openany tiene en este dispositivo: notas, calendario, contactos, archivos, galería, proyectos, mensajes y tus buzones con contraseña y tu propia clave OpenPGP. La app propone una frase de seis palabras; apúntala y guárdala aparte del archivo. Sin ella el archivo no se puede abrir, ni siquiera nosotros.',
          p2: 'Con «Restaurar copia» abres un archivo así en otro dispositivo. Sustituye todo lo que había; después queda desconectado de openany.de y vuelves a emparejar los dispositivos cercanos. Lo que identifica a un dispositivo no se incluye.',
        },
        tresor: {
          title: 'Credenciales y claves en la caja fuerte',
          p1: 'Aquello con lo que este dispositivo se identifica ante Openany y otros dispositivos está guardado bajo llave en el dispositivo; la llave queda en el Android Keystore. Allí están también las contraseñas de tus buzones y tus claves OpenPGP secretas. «Desconectar» elimina las credenciales solo aquí; se revocan definitivamente en la lista de dispositivos de anyid y en las contraseñas de aplicación de Openany.',
        },
      },
    },
    contact: {
      title: 'Ayuda y contacto',
      subs: {
        kontakt: {
          title: 'Contacto con los operadores',
          p1: '¿Preguntas, problemas o sugerencias? Con el enlace «Contacto» del pie de página llegas al formulario de contacto. Tu mensaje llega como mensaje directo a los administradores y la respuesta la encuentras después en Mensajes.',
        },
        bedingungen: {
          title: 'Condiciones de uso',
          p1: 'Las condiciones de uso de la beta y el aviso legal están siempre en los enlaces del pie de página. En resumen: durante la beta Openany es gratuito y sin publicidad, y tus datos están en servidores en Alemania. Solo sale lo que tú mismo envías fuera, por ejemplo mensajes por Matrix o consultas a la IA de un proyecto.',
        },
      },
    },
  },
};
