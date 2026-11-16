// Startseite openany.de (frontend/start.html).
// Achtung vue-i18n: kein rohes @, | oder { in den Texten (außer Platzhaltern).
export default {
  seitentitel: 'openany – privado, cifrado, primeiro no teu dispositivo',
  kopf: 'As tuas notas, compromissos, ficheiros e mensagens – num só lugar.',
  unterzeile: 'Privado, cifrado de ponta a ponta e primeiro no teu dispositivo. Gratuito, sem publicidade, sem palavra-passe.',
  ladenFuer: 'Transferir para {system}',
  alleDownloads: 'Todas as transferências',
  iosFolgt: 'O openany para iPhone e iPad está a caminho.',
  merkmale: {
    verschluesselt: {
      titel: 'Só tu o podes ler',
      text: 'Tudo é cifrado no teu dispositivo antes de chegar ao servidor. Não conseguimos ler os teus conteúdos – nem que quiséssemos.',
    },
    geraet: {
      titel: 'Primeiro no teu dispositivo',
      text: 'O openany funciona sem internet e sem conta. A sincronização através de openany.de é uma opção, não uma obrigação.',
    },
    alles: {
      titel: 'Tudo num só lugar',
      text: 'Notas, calendário, contactos, ficheiros, fotos, vídeos e mensagens – numa app em vez de dez.',
    },
    hubs: {
      titel: 'Juntos em hubs',
      text: 'Partilha pastas, calendários e álbuns com outras pessoas, desenhem juntos num quadro e conversem no chat.',
    },
    sparsam: {
      titel: 'Sem palavra-passe nem e-mail',
      text: 'Crias a tua conta diretamente na app, e ela fica ligada aos teus dispositivos. Não guardamos endereços de e-mail nem endereços IP.',
    },
    frei: {
      titel: 'Gratuito e sem publicidade',
      text: 'Sem subscrição, sem anúncios, sem venda de dados. Cada conta começa com 100 MB de armazenamento em openany.de.',
    },
  },
  downloadsTitel: 'openany para o teu dispositivo',
  version: 'Versão {version}',
  empfohlen: 'Para o teu dispositivo',
  beiGithub: 'Os ficheiros estão alojados no GitHub.',
  keineListe: 'A lista atual não pode ser carregada neste momento.',
  dateien: {
    apk: 'Transferir APK',
    installer: 'Instalador (.exe)',
    msi: 'Pacote MSI',
    dmg: 'Transferir para Mac (.dmg)',
    appimage: 'AppImage',
    deb: 'Debian e Ubuntu (.deb)',
  },
  hinweise: {
    android: 'Diretamente como ficheiro, não na Play Store. Na primeira vez, o Android pergunta se queres permitir apps desta origem.',
    windows: 'Para Windows 10 e 11. A app ainda não está assinada: se o Windows avisar, escolhe «Mais informações» e depois «Executar mesmo assim».',
    macos: 'A partir do macOS 13.3, para chip Apple e Intel. A app ainda não está autenticada pela Apple: no primeiro arranque, permite-a em Definições do Sistema, Privacidade e segurança, «Abrir mesmo assim».',
    linux: 'AppImage para quase todas as distribuições, .deb para Debian e Ubuntu. 64 bits.',
  },
  fuss: {
    melden: 'Denunciar e contacto',
    agb: 'Termos de utilização',
    datenschutz: 'Privacidade',
    impressum: 'Informação legal',
    github: 'Todas as versões no GitHub',
  },
  recht: {
    zurueck: 'Voltar ao início',
    nurDeutsch: 'Este texto só é válido na versão alemã.',
    fehler: 'Não é possível carregar o texto neste momento. Tenta mais tarde.',
  },
  plattformen: 'Para Android, Windows, macOS e Linux. iPhone e iPad estão a caminho.',
  kopfAlt: [
    'Ecrã inicial da app com hoje, contactos, notas, armazenamento e hubs',
    'Álbum da mediateca com paisagens',
  ],
  rundgangKnopf: 'Ver a visita guiada',
  rundgangLabel: 'Visita guiada',
  sprung: {
    notizen: 'Notas',
    kalender: 'Calendário',
    hubs: 'Hubs',
    nachrichten: 'Mensagens',
    mediathek: 'Mediateca',
    dokumente: 'Documentos',
    downloads: 'Transferências',
  },
  rundgang: {
    notizen: {
      augenbraue: 'Notas',
      titel: 'Escrever e ligar as tuas ideias',
      abschnitte: [
        {
          titel: 'Escrever, organizar, ligar',
          absaetze: [
            'As notas ficam em pastas e são escritas em Markdown – com títulos, listas e tarefas para assinalar.',
          ],
          punkte: [
            'As `[[wikiligações]]` ligam as notas entre si',
            'As `#etiquetas` juntam o que pertence junto',
            'A pesquisa de texto completo encontra cada palavra',
          ],
          alt: [
            'Nota com lista, wikiligações, etiquetas e tarefas',
          ],
        },
        {
          titel: 'Ver como tudo se liga',
          absaetze: [
            'A vista «Ligações» mostra as tuas notas como uma rede. Escolhe uma pasta ou uma etiqueta e as notas correspondentes aparecem.',
          ],
          alt: [
            'Lista de notas com pastas e notas, nomes desfocados',
            'Grafo das ligações entre notas, etiquetas desfocadas',
          ],
        },
      ],
    },
    kalender: {
      augenbraue: 'Calendário',
      titel: 'Compromissos para ti e para quem planeia contigo',
      abschnitte: [
        {
          titel: 'A tua semana num relance',
          absaetze: [
            'Mês, semana ou dia: compromissos com local, notas e repetição. Basta tocar no dia.',
          ],
          punkte: [
            'Vários calendários lado a lado',
            '«Hoje» logo no ecrã inicial',
            'Cifrado como tudo o resto',
            'Repetições como «de 2 em 2 semanas à terça, 5 vezes»',
          ],
          alt: [
            'Vista mensal com compromissos de exemplo',
          ],
        },
        {
          titel: 'Partilhar calendários em hubs',
          absaetze: [
            'Com «ver», os membros veem os teus compromissos; com «editar», acrescentam os seus. Só tu alteras o calendário em si.',
          ],
          alt: [
            'Diálogo para partilhar um calendário num hub',
            'Hub com pasta, calendário, álbum e processo partilhados',
          ],
        },
        {
          titel: 'No calendário e nos contactos do teu dispositivo',
          absaetze: [
            'No Android, os teus calendários e contactos do openany aparecem nas apps do telemóvel – e o que alterares lá volta. No computador, o openany disponibiliza-os aos programas de calendário e contactos, só nesse computador.',
          ],
          punkte: [
            'Lembretes de eventos no telemóvel',
            'No computador via CalDAV e CardDAV, por exemplo para o Thunderbird',
          ],
        },
      ],
    },
    hubs: {
      augenbraue: 'Hubs',
      titel: 'Juntos, não lado a lado',
      abschnitte: [
        {
          titel: 'Um lugar para o teu grupo',
          absaetze: [
            'Um hub pertence a uma associação, a uma casa partilhada, a uma família ou a um projeto. Convidas os membros pelo nome no openany. O nome e o chat são cifrados de ponta a ponta.',
          ],
          punkte: [
            'Partilhar pastas, calendários, álbuns e processos',
            'Em cada partilha: ver ou editar',
          ],
          alt: [
            'Lista de membros de um hub com pesquisa para convidar',
          ],
        },
        {
          titel: 'Um board para todos',
          absaetze: [
            'Cada hub tem um board comum. Todos os membros desenham e escrevem nele – através de openany.de ou diretamente por perto, na mesma rede.',
          ],
          punkte: [
            'Formas, setas, texto e caneta',
            'Inserir imagens',
            'Alterações em simultâneo são combinadas',
          ],
          alt: [
            'Board com uma planta de canteiros desenhada',
          ],
        },
        {
          titel: 'Combinar no chat do hub',
          absaetze: [
            'Cada hub tem o seu próprio chat. Com `[[` ligas conteúdos do hub diretamente numa mensagem.',
          ],
          punkte: [
            'Cifrado de ponta a ponta',
            'Fixar mensagens importantes',
            'Avisar os membros, se quiseres',
          ],
          alt: [
            'Chat do hub com três mensagens de exemplo',
          ],
        },
      ],
    },
    nachrichten: {
      augenbraue: 'Mensagens',
      titel: 'Todos os canais numa só lista',
      abschnitte: [
        {
          titel: 'Mensagens diretas, cifradas',
          absaetze: [
            'As mensagens entre contas openany são cifradas de ponta a ponta, anexos incluídos. A elas juntam-se Matrix e e-mail – tudo numa só lista.',
          ],
          punkte: [
            'Filtros: não lidas, com anexo, openany, por perto',
            'Por perto funciona mesmo sem servidor',
            'Por conversa ou por contacto',
          ],
          alt: [
            'Lista de mensagens com pesquisa e filtros, em parte desfocada',
          ],
        },
        {
          titel: 'Matrix: chegar também a quem não usa openany',
          absaetze: [
            'Liga uma conta em matrix.org ou noutro servidor. O teu dispositivo inicia sessão sozinho, com as suas próprias chaves.',
          ],
          punkte: [
            'openany.de não lê nada',
            'O histórico fica no dispositivo',
            'A palavra-passe não é guardada',
          ],
          alt: [
            'Definições: ligar uma conta Matrix',
          ],
        },
        {
          titel: 'E-mail, com PGP se quiseres',
          absaetze: [
            'A tua caixa de correio de sempre torna-se o terceiro canal da lista. O dispositivo fala ele próprio com o servidor de correio.',
            'As chaves OpenPGP públicas dos teus contactos chegam com os e-mails deles – usa-as para cifrar as tuas respostas.',
          ],
          alt: [
            'Definições: ligar uma caixa de correio',
            'Definições: chaves dos contactos',
          ],
        },
      ],
    },
    mediathek: {
      augenbraue: 'Mediateca',
      titel: 'As tuas imagens, vídeos e gravações',
      abschnitte: [
        {
          titel: 'Fotos, vídeos e som em álbuns',
          absaetze: [
            'Álbuns com subálbuns, preenchidos diretamente na app – com câmara, vídeo ou microfone. Os álbuns partilham-se em hubs.',
          ],
          punkte: [
            'As miniaturas são criadas no dispositivo',
            'Os dados de localização mantêm-se, se quiseres',
            'Guardado cifrado',
          ],
          alt: [
            'Álbum da mediateca com paisagens',
          ],
        },
        {
          titel: 'Muitos ficheiros de uma vez',
          absaetze: [
            'Adiciona de uma vez todas as fotos e vídeos do telemóvel – como cópia ou sem ela: assim ficam no telemóvel e seguem cifrados para o openany.de.',
          ],
          punkte: [
            'Fotos novas automaticamente, se quiseres',
            'Partilhar com o openany a partir de outras apps',
            'No computador, arrastar ficheiros e pastas para a janela ou sincronizar uma pasta',
          ],
        },
      ],
    },
    dokumente: {
      augenbraue: 'Documentos',
      titel: 'Papelada onde se pode pesquisar',
      abschnitte: [
        {
          titel: 'A papelada em processos',
          absaetze: [
            'Cartas, contratos e documentos ficam em processos. Os PDF abrem diretamente no openany: folhear, pesquisar, preencher formulários, realçar e comentar.',
          ],
          punkte: [
            'Pesquisa também dentro de PDF e ficheiros de escritório',
            'Partilhar processos em hubs',
          ],
          alt: [
            'Documentos com uma lista de processos, nomes privados desfocados',
          ],
        },
        {
          pille: 'Reconhecimento de texto',
          titel: 'Da fotografia ao PDF pesquisável',
          absaetze: [
            'Fotografa uma carta e coloca-a num processo. Torna-se um PDF com uma camada de texto invisível: pesquisar, realçar, copiar.',
            'O reconhecimento é feito no teu dispositivo. A imagem não vai para lado nenhum.',
          ],
          alt: [
            'Aviso de que uma fotografia está a ser convertida',
            'Carta fotografada em PDF, a pesquisa realça uma palavra',
          ],
        },
        {
          pille: 'PDF',
          titel: 'Cada nota em PDF',
          absaetze: [
            'Um toque transforma uma nota num PDF e guarda-o nos teus documentos. Há quatro versões à escolha:',
          ],
          punkte: [
            'Padrão, com título, data e fontes',
            'Clean, só o texto',
            'Carta segundo a DIN 5008',
            'Versão de impressão com página de rosto',
          ],
          alt: [
            'Diálogo para criar um PDF com quatro versões',
            'Nota como PDF pronto no visualizador',
          ],
        },
      ],
    },
  },
  versprechen: {
    augenbraue: 'A promessa',
    titel: 'Só tu consegues ler.',
    text: 'Tudo é cifrado no teu dispositivo antes de chegar ao servidor. Alojado na Alemanha, sem publicidade e sem rastreio.',
  },
  downloadsAugenbraue: 'Transferências',
  ios: {
    titel: 'iPhone e iPad',
    pille: 'A caminho',
    text: 'A app para iOS está a caminho. Assim que estiver pronta, encontra-la aqui.',
  },
  alleVersionen: {
    titel: 'Todas as versões',
    text: 'As versões anteriores e todos os ficheiros estão no GitHub.',
    knopf: 'Ir para o GitHub',
  },
};
