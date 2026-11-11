// Atenção vue-i18n: nada de @, | ou { sem escape nos textos.
export default {
  title: 'Ajuda',
  intro: 'Aqui encontras descrições e instruções sobre todos os módulos e funções do Openany. Pelo índice à esquerda saltas diretamente para a secção pretendida.',
  version: 'Versão {version}',
  aktualisieren: 'Atualiza a app',
  aktualisierenTitel: 'O openany.de precisa pelo menos da versão {version}.',
  toc: 'Conteúdo',
  sections: {
    start: {
      title: 'Primeiros passos',
      subs: {
        ohneKonto: {
          title: 'A app sem conta',
          p1: 'A app é um programa completo que funciona também sem conta e sem servidor: notas, calendário, disco, livro de endereços e mensagens ficam então só neste dispositivo. O que aqui não está configurado ou não funciona, a app oculta em vez de o mostrar vazio – a via Openany das mensagens, por exemplo, só aparece quando o dispositivo está ligado ao openany.de.',
          p2: 'Se quiseres sincronizar os teus dados com openany.de, cria uma conta nas definições, em «Sincronização», ou liga-te a uma existente; com os teus outros dispositivos emparelhas em «Dispositivos por perto». Depois, o cabeçalho e o menu mostram um botão «Sincronizar» que gira enquanto sincroniza.',
        },
        kontoAnlegen: {
          title: 'Criar uma conta',
          p1: 'Crias uma conta em openany.de diretamente na app: nas definições, em «Sincronização», toca em «Criar nova conta» e escolhe um nome. Depois, o teu dispositivo resolve um pequeno cálculo que protege contra a criação de contas em massa; costuma demorar menos de um minuto. Não precisas de endereço de e-mail nem de palavra-passe.',
          p2: 'Depois aceitas os termos de utilização – só então tens espaço em openany.de, 100 MB para começar. Ligas outros dispositivos através de um já ligado, com 6 dígitos (ver «Os meus dispositivos»).',
          p3: 'Como a tua conta não tem palavra-passe, um ficheiro de cópia de segurança é a única forma de voltar se perderes o teu único dispositivo. A app oferece-o logo depois de criares a conta; cria-o e guarda-o bem com a sua frase-passe (ver «Cópia de segurança num ficheiro»).',
        },
        startseite: {
          title: 'A página inicial',
          p1: 'A página inicial mostra mosaicos compactos dos módulos que escolheste: os compromissos de hoje, os contactos com pesquisa, as três notas editadas mais recentemente, os três últimos ficheiros com o espaço ocupado e os teus hubs. Cada mosaico leva ao seu módulo; quais aparecem defines nas definições, em «Módulos na página inicial».',
          p2: 'No início estão escolhidos todos os mosaicos. Se os retirares todos, aparece em vez disso a apresentação de boas-vindas: mostra todas as funções e deixa-te compor os mosaicos diretamente. A escolha não esconde nada – todas as áreas continuam acessíveis pela barra e pelo menu.',
        },
        navigation: {
          title: 'Menu e estrutura',
          p1: 'No computador há um cabeçalho em cima: à esquerda, o logótipo redondo leva à página inicial e ao lado estão Notas, Calendário, Disco e Hubs. À direita seguem o interruptor claro/escuro, o envelope das mensagens com contador de não lidas e o campo com o teu nome – abre Definições, Ajuda, Reciclagem e Terminar sessão.',
          p2: 'No telemóvel tudo passa para baixo: uma barra fixa mostra os módulos como ícones e, mais à esquerda, fica o botão de menu com o contador de mensagens não lidas. Abre um painel com Mensagens, Definições, Ajuda, Reciclagem, o interruptor claro/escuro e Terminar sessão. O interruptor alterna entre claro, escuro e automático – automático segue a definição do teu dispositivo. Um toque no módulo já aberto leva-te ao seu ponto de partida.',
        },
      },
    },
    notes: {
      title: 'Notas',
      subs: {
        editor: {
          title: 'Escrever no editor',
          p1: 'O editor de notas funciona como um processador de texto moderno: títulos, listas, citações, blocos de código, negrito e itálico escolhem-se na barra de ferramentas. Em alternativa, escreves atalhos Markdown – por exemplo um cardinal seguido de espaço para um título ou um hífen para uma lista – que se transformam em formatação enquanto escreves.',
          p2: 'A gravação é automática: pouco depois de parares de escrever, o Openany guarda a nota e mostra o estado na barra de ferramentas. Por baixo de cada nota estão ainda a data de criação e a da última alteração. No telemóvel, a barra de ferramentas aparece numa só linha logo acima do teclado assim que tocas no texto e desliza lateralmente – todas as ferramentas ficam à mão sem atrapalhar a leitura.',
        },
        mappen: {
          title: 'Pastas e estrutura',
          p1: 'As notas ficam em pastas que se podem aninhar sem limite. Podes mudar o nome, mover e reordenar notas e pastas – assim cresce com o tempo a tua própria coleção de conhecimento. A lupa em cima abre um campo de pesquisa que procura em todas as notas de uma vez, título e conteúdo; aberta dentro de uma pasta, «só nesta pasta» limita-a a essa pasta. O que podes fazer com a pasta aberta está em «…», à direita da barra de caminho.',
          p2: 'Sobes pelo caminho ou com a tecla Retroceder, que sobe um nível; a cruz na barra de caminho fecha uma nota aberta. Ao lado de cada pasta vês quando uma nota nela mudou pela última vez, incluindo subpastas. Uma pasta sem notas não mostra data.',
        },
        wikilinks: {
          title: 'Wikilinks e retroligações',
          p1: 'Com dois parênteses retos ligas notas entre si: escreve os parênteses e o título da nota de destino e a ligação surge automaticamente – se a nota ainda não existir, a ligação fica ativa assim que existir. Se no texto deve aparecer outra coisa que não o título, indicas um texto apresentado próprio depois de uma barra vertical; a ligação continua a apontar para a nota certa.',
          p2: 'Cada nota mostra as suas retroligações: uma lista de todas as notas que apontam para ela. Assim reencontras relações sem teres de tomar nota tu.',
        },
        tags: {
          title: 'Etiquetas',
          p1: 'As palavras-chave atribuem-se diretamente no texto com um cardinal antes da palavra; ao escrever, uma lista sugere as etiquetas já usadas. Com uma barra aninhas palavras-chave – por exemplo Hub/Subtema; um filtro no tema principal encontra também todos os subtemas. Na vista de grafo, as palavras-chave podem aparecer como nós próprios, e um clique numa palavra-chave filtra lá a lista de notas.',
        },
        einfuegen: {
          title: 'Inserir imagens e ficheiros',
          p1: 'O botão do clipe na barra de ferramentas insere numa nota um ficheiro do teu dispositivo. As imagens aparecem como pré-visualização compacta na própria nota (a resolução completa está na Mediateca) e os outros ficheiros como ligação para abrir.',
          p2: 'Os conteúdos inseridos são arrumados automaticamente: imagens num álbum da Mediateca, ficheiros de texto e Office num dossiê dos Documentos, tudo o resto numa pasta de ficheiros – sempre numa área própria para notas.',
        },
        zitate: {
          title: 'Citações e bibliografia',
          p1: 'Para trabalho académico, cada pasta tem uma lista de fontes: na pasta aberta escolhes «Fontes» em «…» e registas livros, artigos, sites, decisões judiciais ou leis. Cada fonte recebe um nome curto. A lista é um ficheiro CSL-JSON em Ficheiros, utilizável também com Zotero e Zettlr.',
          p2: 'Nas notas desta pasta, o editor sugere fontes adequadas ao citar e coloca a citação no texto. Assim as citações surgem enquanto escreves, sem escrever os dados de cada vez. Como aparecem no PDF está em «Versão de impressão».',
        },
        graph: {
          title: 'Vista de grafo',
          p1: 'A vista de grafo desenha as tuas notas como uma rede: cada nota é um ponto, cada wikilink uma ligação. Os pontos são coloridos pela pasta de nível superior, de modo que as áreas relacionadas se reconhecem pela cor; pontos ocos são notas sem qualquer ligação, e quanto mais ligações uma nota tiver, maior é o seu ponto.',
          p2: 'Cada pasta de primeiro nível é um nível. Se houver vários, escolhes um ou mais na legenda por baixo do grafo – «Todos os níveis» escolhe todos, «Nenhum» limpa a seleção. Em vez de um nível, também uma etiqueta pode ser a entrada: a barra por baixo lista as etiquetas com o seu número, e uma etiqueta escolhida mostra todas as notas que a têm, com a etiqueta como losango cinzento. Com níveis escolhidos, a barra só lista as etiquetas que neles aparecem. Até escolheres algo, o grafo fica vazio, para que coleções grandes não sejam construídas de uma vez; «Mostrar todas as etiquetas» traz todas as etiquetas da barra de uma vez.',
          p3: 'No computador fazes zoom com os botões mais/menos em cima à direita ou com a roda do rato e deslocas a vista com o botão premido; no telemóvel fazes zoom com dois dedos e deslocas com um. Outro botão repõe a vista. Um clique num ponto (sem arrastar) abre a nota. Um clique numa etiqueta filtra a lista de notas por ela.',
        },
        export: {
          title: 'Exportar para PDF',
          p1: 'O botão PDF na nota aberta oferece quatro versões: «Padrão» é a nota com título, data e bibliografia. «Clean» é só o texto, sem título, data, números de página nem bibliografia; as citações ficam como as escreveste. «Carta» compõe a nota segundo a DIN 5008 para envelope de janela, com o título como assunto. «Versão de impressão» gera apenas esta nota com folha de rosto. O PDF vai para o processo «Notizen» em Armazenamento → Documentos (as versões anteriores ficam ao lado) e a versão de impressão para o processo «Druckfassungen»; «Abrir» no aviso mostra-o de imediato.',
        },
        druckfassung: {
          title: 'Versão para impressão',
          p1: 'Uma pasta torna-se um documento formal para entregar ou imprimir: na pasta aberta escolhes «Versão para impressão» em «…», introduzes título, subtítulo, nome, instituição, data e informações adicionais para a folha de rosto, e decides se queres um índice. Cada nota da pasta torna-se um capítulo, por ordem alfabética do título; com prefixos como «01 Introdução» defines a ordem, e no documento aparece apenas «Introdução». As subpastas entram quando «Incluir subpastas» está marcado: uma subpasta torna-se um título e as suas notas ficam por baixo («2 Fundamentos», «2.1 Conceitos»); uma nota com o mesmo nome da subpasta aparece como texto logo abaixo desse título.',
          p2: 'A composição usa uma letra do tipo Times com títulos sem serifa, margens de 2,5 cm, espaçamento de 1,5 linhas, texto justificado com hifenização; a numeração começa depois da folha de rosto. Os dados ficam guardados na pasta. O PDF é criado no teu dispositivo – da primeira vez demora alguns segundos – e vai para o processo «Druckfassungen» em Armazenamento → Documentos; cada versão é mantida.',
          p3: "Citações da lista de fontes da pasta: no texto escreves [{'@'}nomecurto] ou [{'@'}nomecurto, p. 12], e ^[Texto] para uma nota de rodapé. No PDF – nota ou versão para impressão – surgem citações Harvard como «(Müller, 2020, p. 12)» e, no fim, a bibliografia.A barra do editor tem um botão para notas de rodapé (número em expoente): insere os parênteses retos, e o texto selecionado torna-se a nota.",
        },
        papierkorb: {
          title: 'Reciclagem',
          p1: 'As notas eliminadas vão para a reciclagem (no menu por baixo do teu nome; no telemóvel, no botão de menu) e podem lá ser restauradas ou eliminadas definitivamente. Ao eliminar uma pasta decides se as notas que contém são eliminadas também ou movidas para a pasta superior.',
        },
        verschluesselt: {
          title: 'Encriptado de ponta a ponta',
          p1: 'As tuas notas, pastas, dados da versão para impressão e listas de fontes ficam guardados no openany.de apenas encriptados: só os teus dispositivos autorizados os conseguem ler. O servidor também não vê os nomes das tuas pastas. A pesquisa, as referências inversas, as etiquetas e o grafo são calculados no teu dispositivo.',
        },
        ordner: {
          title: 'Notas como ficheiros (Zettlr)',
          p1: 'Na app do computador podes ligar uma das tuas pastas de notas a uma pasta do sistema: abre a pasta, escolhe «Ligar a uma pasta …» no menu «…» e seleciona o destino. Cada nota fica lá como ficheiro «Id Título.md», as subpastas como subdiretórios – como o Zettlr ou o Obsidian esperam. O que alterares, renomeares, moveres ou criares lá é recolhido pela app enquanto estiver aberta; se ambos os lados mudaram, é criada uma cópia de conflito. Se apagares um ficheiro, a nota vai para o lixo. A pasta fica sem cifra no teu computador; «Desligar pasta» deixa os ficheiros onde estão. As imagens das notas ficam ao lado da nota, com o nome usado no texto; uma imagem nova aí que uma nota referencie passa para a app (mediateca, álbum «Notizen: …»).',
        },
      },
    },
    files: {
      title: 'Disco',
      subs: {
        aufbau: {
          title: 'Mediateca, ficheiros, documentos',
          p1: 'O Disco divide-se em três separadores no topo da página: a Mediateca para fotografias e vídeos, Ficheiros para tudo o resto e Documentos para papéis em dossiês. Ao mudares de separador, a outra área fica como a deixaste – pastas abertas e listas carregadas não se perdem.',
          p2: 'Qual o separador que abre primeiro quando entras no Disco defines nas definições, em «Início do disco»; enquanto não escolheres, são os Documentos.',
        },
        dokumente: {
          title: 'Arquivo de documentos',
          p1: 'No separador «Documentos» organizas ficheiros PDF, Office e de texto em dossiês. Carregas com o botão ou simplesmente arrastando os ficheiros para o dossiê aberto. No telemóvel e no tablet há ao lado um botão de câmara: fotografa um papel e guarda-o logo como documento no dossiê.',
          p2: 'Os processos podem aninhar-se sem limite. Podes mudar o nome e mover processos e documentos; processos inteiros também se podem partilhar com um hub em «Partilhas». Ao lado de cada processo vês quando algo nele mudou pela última vez, incluindo subprocessos.',
        },
        texterkennung: {
          title: 'Documentos pesquisáveis',
          p1: 'Se fotografares um documento em papel e o guardares num dossiê – com o botão de câmara ou como fotografia da tua coleção de imagens –, transforma-se automaticamente num PDF com uma camada de texto invisível. O documento parece a tua fotografia, mas pode ser pesquisado, marcado, copiado e lido em voz alta. O reconhecimento corre no teu próprio dispositivo – a imagem não é enviada para lado nenhum.',
          p2: 'Se guardares várias fotografias de uma vez, é-te perguntado se devem formar um documento com várias páginas – como as páginas de uma carta – ou documentos separados. A fotografia em si não é guardada à parte; quem quiser manter a imagem guarda-a na Mediateca. Da primeira vez a conversão demora mais, porque o reconhecimento de texto é carregado uma única vez e depois fica guardado.',
          p3: 'Se já houver um PDF digitalizado num dossiê, podes aplicar-lhe o reconhecimento depois: «Reconhecer texto» lê o documento página a página. Se o papel for branco e limpo, as páginas ficam inalteradas e só se acrescenta o texto. Se estiverem cinzentas ou com iluminação irregular – como uma folha fotografada –, são também clareadas; a mensagem final diz-te o que aconteceu. Se o PDF já era pesquisável, não é tocado.',
        },
        dateien: {
          title: 'Gerir ficheiros',
          p1: 'No separador «Ficheiros» crias pastas e carregas ficheiros de qualquer tipo – um a um ou vários de uma vez. Tocar num ficheiro abre-o com uma aplicação adequada do teu dispositivo. O ícone de transferência guarda uma cópia na pasta Download ou, no computador, onde escolheres; nos PDF está no visualizador. Pastas e ficheiros podem mudar de nome e ser movidos; as pastas também se podem partilhar com um hub em «Partilhas». O que podes fazer com uma pasta ou um processo aberto está em «…», à direita da barra de caminho. Sobes pelo caminho ou com a tecla Retroceder, que sobe um nível.',
          p2: 'Quanto ocupas vê-se no mosaico da página inicial. Quanto fica neste dispositivo defines nas definições, em «Armazenamento neste dispositivo».',
        },
        suche: {
          title: 'Pesquisar',
          p1: 'A lupa em Ficheiros e Documentos abre um campo de pesquisa que percorre todas as pastas ou dossiês de uma vez, não só o que está aberto. Um clique num resultado abre o ficheiro; ao lado vês onde está, e um clique nessa indicação leva-te à pasta.',
          p2: 'A pesquisa não encontra só pelo nome, mas também no texto: em PDF, textos Word e OpenDocument e ficheiros de texto, CSV e Markdown até 50 MB. O Openany lê esse texto aos poucos em segundo plano enquanto o Disco está aberto; até ter lido tudo, a pesquisa diz-te quantos ficheiros por enquanto só se encontram pelo nome. Num resultado no texto, a lista mostra a passagem com a palavra pesquisada realçada. PDF digitalizados sem camada de texto só aparecem depois de lhes aplicares «Reconhecer texto».',
        },
        pdf: {
          title: 'Ver e editar PDF',
          p1: 'Um clique num PDF abre-o diretamente no Openany. Folheias as páginas, aumentas ou reduzes, ajustas a vista à largura e pesquisas no documento com a lupa – os resultados ficam realçados na página. «Abrir com outra aplicação» passa-o a outra aplicação. Se um PDF estiver protegido por palavra-passe, o Openany pede-a primeiro.',
          p2: 'Os formulários do PDF preenches diretamente. Com «Editar» aparece ainda uma barra de ferramentas: «Realçar» marca texto a cores, com a «Caneta» desenhas livremente na página e com «Texto» colocas as tuas próprias linhas na folha – cor, espessura do traço e tamanho de letra escolhes tu, e cada passo pode ser anulado. Os campos de cálculo dos formulários, porém, não calculam, porque o Openany não executa scripts de ficheiros alheios.',
          p3: 'Aos realces e traços podes juntar comentários: escolhe «Comentários» e toca no sítio; a mesma vista lista todos os comentários do documento. Tudo é guardado como anotações PDF normais, que outros programas também mostram. Ao guardar pela primeira vez, o Openany põe a versão anterior na reciclagem, para poderes voltar ao original; se saíres do PDF com alterações por guardar, pergunta antes.',
        },
        galerie: {
          title: 'Mediateca e álbuns',
          p1: 'No separador «Mediateca» estão as tuas fotografias e vídeos – em álbuns com miniaturas ou, sem álbum, por baixo, em «Multimédia». Os álbuns podem aninhar-se – por exemplo um álbum por viagem com um subálbum por dia – e mover-se a qualquer momento. O visualizador percorre todas as imagens de um álbum. Moves uma fotografia na vista grande; mantendo-a premida na grelha, selecionas várias e moves ou eliminas em conjunto. O que podes fazer com o álbum aberto está em «…», à direita da barra de caminho.',
          p2: 'A captura mais recente fica em cima, agrupada por meses com um título para cada mês. Conta a data de captura dos dados da fotografia; se faltar – por exemplo em capturas de ecrã ou imagens editadas –, conta o dia do carregamento.',
          p3: 'Na vista grande transferes uma fotografia ou um vídeo como ficheiro original: no telemóvel para a pasta Download, no computador para onde escolheres. Os álbuns podem ser partilhados com um hub em «Partilhas». Os originais ficam contigo. Em «Ações» no álbum carregas a tua própria imagem de capa; não conta como item.',
        },
        fotosVomGeraet: {
          title: "Fotos do dispositivo",
          p1: "No Android, na mediateca, «Adicionar» → «Adicionar fotos do dispositivo» traz fotos e vídeos que já estão no dispositivo: todos, só certos álbuns do dispositivo ou a partir de uma data. Os álbuns do dispositivo passam a álbuns com o mesmo nome no openany; antes vês se há espaço suficiente, e o que já foi adicionado é saltado. Se o dispositivo estiver ligado ao openany.de, escolhes uma vez como as fotos entram: no openany.de, e no dispositivo fica só a pré-visualização, sem espaço a dobrar – ou como cópia neste dispositivo.",
          p2: "Com «Adicionar fotos novas automaticamente» – na mediateca, em «Adicionar», ou em Definições → Armazenamento e cópia de segurança – o que a câmara captar a partir de agora vai sozinho para a mediateca, mesmo com o openany fechado: dos álbuns do dispositivo que escolheres, a partir de quando ativas. Se quiseres, só em Wi-Fi ou só a carregar. O Android limita o trabalho em segundo plano, por isso as fotos chegam com algum atraso. No openany.de contam para o teu armazenamento. Se usas o Google Fotos, desativa lá a cópia de segurança – senão ambos enviam.",
        },
        videos: {
          title: 'Vídeos e câmara',
          p1: 'Além de fotografias, a Mediateca também aceita vídeos. Aparecem como mosaico com uma imagem fixa; tocar neles abre-os com a aplicação de vídeo do teu dispositivo.',
          p2: 'No telemóvel e no tablet, a Mediateca e cada álbum têm dois botões de câmara: um para fotografia, outro para vídeo. A gravação vai diretamente para o sítio que está aberto. No computador não há estes botões; carregas como de costume.',
        },
        ton: {
          title: 'Gravações de áudio e música',
          p1: 'A mediateca também guarda ficheiros de áudio, como MP3 ou M4A. Aparecem como um mosaico com nome e duração; um toque reprodu-los na app. Os áudios vindos de fora também ficam aqui.',
          p2: 'O botão do microfone grava diretamente, com pausa, descartar e guardar. A gravação vai para o álbum aberto ou, caso contrário, para o álbum «Gravações». No Android continua com o ecrã desligado, como indica a notificação. No Linux este botão não existe.',
          p3: 'No Android (versão 10 ou superior) e no Windows podes escolher o áudio do sistema em vez do microfone. O sistema pede o teu consentimento de cada vez. Algumas apps bloqueiam isto e as chamadas nunca são gravadas. Grava conversas apenas com o acordo de todas as pessoas.',
        },
        karte: {
          title: 'Data e local de captura',
          p1: 'Para cada imagem, o visualizador mostra a data de captura e – se a fotografia o contiver – o local; «Ver no mapa» abre o sítio no OpenStreetMap. Se carregares fotografias de um telemóvel Android, escolhe-as pela app Ficheiros e não pela Galeria: caso contrário, o Android remove a localização antes de a imagem chegar ao Openany.',
        },
        papierkorbSpeicher: {
          title: 'Reciclagem',
          p1: 'Os conteúdos eliminados – documentos, ficheiros, fotografias, notas, contactos, mas também hubs, calendários e compromissos – vão primeiro para a reciclagem comum e podem lá ser restaurados. Após 30 dias, a reciclagem esvazia-se automaticamente; só então o espaço fica definitivamente livre.',
        },
        dateienVerschluesselt: {
          title: 'Encriptado de ponta a ponta',
          p1: 'Os teus ficheiros, processos, fotos e álbuns só são guardados encriptados no openany.de: nomes, pastas, álbuns, datas de captura e local estão num cabeçalho encriptado, e os conteúdos são encriptados com uma chave própria para cada ficheiro. Só os teus dispositivos autorizados os conseguem ler.',
        },
      },
    },
    calendar: {
      title: 'Calendário',
      subs: {
        verwalten: {
          title: 'Vários calendários',
          p1: 'Podes ter quantos calendários quiseres lado a lado – por exemplo pessoal, família, clube –, cada um com a sua cor. Um clique mostra ou oculta um calendário. Em cima alternas entre mês, semana e dia; semana e dia mostram os compromissos por hora, e os passados aparecem discretamente a cinzento. No telemóvel e no tablet mudas de página deslizando: para cima e para baixo no mês, para o lado na semana e no dia. No computador há setas para isso.',
        },
        termine: {
          title: 'Criar compromissos',
          p1: 'Crias compromissos com um clique num dia: com título, descrição, hora, ou como evento de dia inteiro ou de vários dias. Também há repetições – diárias, semanais, mensais ou anuais. Um clique num compromisso mostra os detalhes, onde o editas ou eliminas.',
          p2: "Um lembrete avisa à hora escolhida antes do início. Se um evento vier de outro programa com uma repetição mais precisa – de duas em duas semanas, só três vezes, às terças e quintas, em cada primeira segunda-feira –, ela mantém-se e aparece descrita por palavras no evento. Se escolheres tu outra repetição, ela substitui a antiga. Um evento de uma série movido individualmente continua a ser um evento próprio, e a série deixa livre o seu lugar antigo.",
        },
        abos: {
          title: 'Subscrições de calendário',
          p1: 'Calendários externos – por exemplo feriados ou o calendário de jogos do clube – integras através de um URL de subscrição em formato ICS. Os calendários subscritos atualizam-se automaticamente. Os seus compromissos são só de leitura no Openany: alteram-se onde o calendário tem origem, porque cada atualização volta a substituir o conteúdo pelo original.',
        },
        verschluesselt: {
          title: 'Encriptado de ponta a ponta',
          p1: 'Os teus calendários e eventos ficam guardados no openany.de apenas encriptados: só os teus dispositivos autorizados os conseguem ler. O endereço de uma subscrição também é encriptado; cada dispositivo obtém-na sozinho, o openany.de não obtém nada.',
        },
        importExport: {
          title: 'Importar e exportar',
          p1: 'Com «Importar» carregas um ficheiro ICS para um dos teus calendários. Se importares o mesmo ficheiro outra vez, os eventos são atualizados em vez de duplicados. O ícone de partilha junto a um calendário guarda-o como ficheiro ICS – para levar para qualquer outra aplicação de calendário.',
        },
        imTelefon: {
          title: "No calendário do telefone",
          p1: "No Android, Definições → Calendário → «Mostrar no calendário do telefone» mostra os teus calendários em qualquer app de calendário do telefone, na conta «openany» – também os partilhados em hubs, e as subscrições se as juntares. O que lá alterares, criares ou apagares volta para o openany. Os lembretes passam a ser mostrados pelo próprio telefone.",
          p2: "O Google Calendar só mostra outras contas depois de um passo teu: lá, Definições → Gerir contas → ativar «openany». Os eventos só ficam desencriptados neste dispositivo. Desativar retira-os do calendário do telefone; no openany fica tudo.",
        },
        andereProgramme: {
          title: "Para outros programas no computador",
          p1: "No computador, Definições → Calendário → «Oferecer a outros programas neste computador» oferece os teus calendários e o teu livro de endereços por CalDAV e CardDAV – para o Thunderbird, o Calendário e os Contactos da Apple e outros. O mosaico mostra endereço, utilizador e palavra-passe para copiar; com eles configuras uma conta no programa. No Thunderbird: Novo calendário → Na rede, ou Novo livro CardDAV. Na Apple: Outra conta CalDAV ou CardDAV → Manual.",
          p2: "O serviço só está acessível neste computador e só enquanto o openany estiver aberto; com a app fechada, o programa atualiza mais tarde. As alterações feitas lá voltam para o openany; os calendários partilhados em hubs são lá apenas de leitura. «Nova palavra-passe» bloqueia os programas já configurados até receberem a nova.",
        },
      },
    },
    projects: {
      title: 'Hubs',
      subs: {
        grundlagen: {
          title: 'Hubs e membros',
          p1: 'Um hub é um espaço de trabalho partilhado: convidas outros utilizadores como membros e os convidados aceitam ou recusam o convite. O proprietário do hub gere os membros e pode também passar o hub a outro membro.',
          p2: 'Cada hub tem quatro separadores: Partilhas, Board, Membros e Chat. Abre nas Partilhas. Os números nos separadores mostram mensagens de chat por ler ou quanto contêm.',
        },
        chat: {
          title: 'Chat do hub',
          p1: 'Cada hub tem um chat comum em tempo real: as mensagens aparecem de imediato para todos os membros, sem recarregar a página. O chat está cifrado de ponta a ponta: só os membros o leem nos seus dispositivos, o openany.de não. Se alguém sair, a chave muda sozinha – deixa de ler o que é novo.',
          p2: 'Qualquer membro pode afixar mensagens importantes – as mensagens afixadas encontram-se rapidamente na barra de afixadas por cima do histórico e também podem lá ser desafixadas.',
          p3: 'Com dois parênteses retos remetes, a meio da frase, para conteúdos do hub: notas e pastas, ficheiros, álbuns e imagens partilhados. Ao escrever, uma lista de sugestões mostra o que serve e insere a ligação pronta. O botão com o símbolo de corrente ao lado do botão de enviar faz o mesmo – no telemóvel costuma ser mais prático do que procurar dois parênteses.',
          p4: 'Um clique numa ligação leva diretamente ao destino: à nota, à pasta com o ficheiro realçado ou à imagem na vista ampliada. Só podes remeter para o que já está visível no hub – o que ninguém partilhou não aparece na lista de sugestões nem se torna uma ligação clicável.',
        },
        board: {
          title: 'Board',
          p1: 'Cada hub tem exatamente um board – uma tela de desenho partilhada, criada juntamente com o hub. Abres-o em «Board» com «Abrir o board». Todos os membros desenham, escrevem e organizam nele; guarda-se sozinho, pouco depois de cada alteração. Com «Inserir» colocas no board imagens, notas e ficheiros partilhados do hub como cartão.',
          p2: 'Se várias pessoas desenham ao mesmo tempo, a app junta as alterações: cada elemento fica com a versão mais recente, e o que foi apagado continua apagado. Com membros por perto desenham juntos em direto e veem os ponteiros uns dos outros; caso contrário, as alterações chegam na sincronização seguinte. O board é encriptado de ponta a ponta.',
        },
        geteilteKalender: {
          title: 'Partilhar calendários',
          p1: 'Em «Partilhas», com «Partilhar» (por perto: «Partilhar algo») partilhas também um dos teus calendários com o hub. Aparece então na lista em «Calendários»; um toque abre-o no calendário – para os membros em «De hubs». Com «ver» só o leem; com «editar» criam, alteram e apagam eventos. O calendário em si só tu o alteras. Se duas pessoas alterarem o mesmo evento ao mesmo tempo, a segunda versão fica ao lado como cópia, com «Conflito», data e hora no título. Tudo é encriptado com uma chave própria. O X junto ao calendário termina a partilha; o teu calendário fica. Sem openany.de, os membros recebem-no assim que sincronizam por perto – aí só leitura; os calendários subscritos só se partilham assim.',
        },
        freigaben: {
          title: 'Partilhar conteúdos',
          p1: 'No separador «Partilhas», «Partilhar» partilha com o hub uma pasta de notas, um álbum ou uma pasta (ou processo): primeiro escolhes o tipo, depois qual. Tudo continua cifrado ponta a ponta – com uma chave própria por partilha, que muda quando deixas de partilhar ou alguém sai do hub. O conteúdo fica contigo; quando a partilha termina, guardas tudo.',
          p2: 'Os álbuns e pastas são partilhados só para ver: os membros abrem-nos no hub, veem subálbuns e subpastas, abrem imagens e PDF e, com «Copiar para mim», guardam uma cópia no seu próprio armazenamento em «De hubs». Só tu os podes alterar.',
          p3: 'As pastas de notas partilham-se com o nível «ver» ou «editar». Os membros abrem-nas no editor de notas habitual e veem também as imagens; com «editar» alteram notas e criam novas. Por agora, só quem é dono da pasta pode inserir imagens.',
        },
        lokal: {
          title: 'No openany.de ou só presencialmente',
          p1: 'Com «+ Hub» crias um hub neste dispositivo. Se o dispositivo estiver ligado ao openany.de, vai logo também para lá – na lista leva então uma pequena nuvem. Quem fundou um hub ativa-o e desativa-o em Definições → Sincronização → «Hubs no openany.de». Desativado, só sincroniza presencialmente; no openany.de são apagados o hub, o chat, o planeamento e as partilhas. Com nuvem, convidas os membros pelo nome no openany (só proprietários, e só quem já tenha sincronizado a app com o openany.de alguma vez). Sem nuvem, convidas em «Convidar um membro por perto»: no outro dispositivo o Openany tem de estar aberto, na mesma rede Wi-Fi ou hotspot, e ambos mostram os mesmos seis dígitos, que comparam e confirmam. O hub é gerido pelos dispositivos do proprietário; se houver só um, a app avisa – emparelha antes um segundo dispositivo teu.',
          p2: 'Os membros partilham as suas próprias pastas de notas, pastas e álbuns no hub em «Partilhar algo», de início só para leitura. As pastas de notas também se podem partilhar para edição: aí só uma pessoa edita uma nota de cada vez, e guarda-se no dispositivo da pessoa a quem a pasta pertence – se esse não estiver por perto, a nota fica só de leitura. O chat chega logo a todos os que estão por perto, e aos restantes na sincronização seguinte. O board viaja da mesma forma.',
          p3: 'O resto sincronizas com «Sincronizar» assim que houver um membro por perto. O proprietário pode remover membros; quem quiser sair fá-lo com «Sair», o que só funciona se outro membro estiver por perto para ficar a saber. O que quem saiu tinha partilhado desaparece para os outros; o que já tinham fica nos seus dispositivos.',
        },
      },
    },
    messages: {
      title: 'Mensagens',
      subs: {
        direkt: {
          title: 'Mensagens diretas',
          p1: 'Abres as mensagens a partir do menu. Com «Mensagem» em cima à direita escreves diretamente a qualquer outro utilizador – basta o nome dele no Openany. As ligações nas mensagens são clicáveis.',
          p2: 'Por baixo de cada mensagem recebida, «Responder» abre o editor com o remetente já preenchido, pelo mesmo canal por onde a mensagem chegou. As mensagens longas aparecem recolhidas; «Ler mais» mostra-as por inteiro. O contador de não lidas no botão de menu mostra se há algo novo.',
          p3: 'As mensagens diretas são encriptadas de ponta a ponta: o openany.de só guarda dados ilegíveis e só os teus dispositivos as podem ler. Todos os teus dispositivos partilham uma chave. Um dispositivo novo mostra 6 dígitos; introduze-los noutro já configurado e ele fica com sessão iniciada e com a chave. A chave também está no teu ficheiro de cópia de segurança. Se perderes um dispositivo, retira-o nas definições em «Os meus dispositivos»; a chave muda nessa altura. Podes escrever a qualquer pessoa que tenha aberto a app pelo menos uma vez.',
        },
        suche: {
          title: 'Pesquisar e filtrar',
          p1: 'Por cima do histórico há um campo de pesquisa. Encontra mensagens pelo texto, pelo nome do interlocutor e por identificadores Matrix, e realça os resultados. Por baixo, interruptores restringem a lista: às mensagens não lidas, às que têm anexo e – se usares mais de uma via – a vias concretas como Openany ou Matrix. Os interruptores podem combinar-se.',
          p2: 'Ao lado do campo de pesquisa alternas entre duas vistas: «Histórico» mostra todas as mensagens por ordem temporal, «Por contacto» reúne-as numa linha por pessoa, com a última mensagem e o número de não lidas. Um clique numa linha mostra o histórico com essa pessoa; «Fechar conversa» leva-te de volta a todas.',
        },
        anhaenge: {
          title: 'Anexos',
          p1: 'Pelo Matrix envias também ficheiros: o clipe no campo de escrita anexa um ficheiro até 10 MB, e no telemóvel e no tablet o botão de câmara ao lado tira logo uma fotografia. No histórico, as imagens aparecem em pré-visualização e os outros ficheiros com nome e tamanho; um clique abre imagens e PDF diretamente no Openany, o resto transferes. Pela via Openany vão ficheiros até 10 MB, encriptados de ponta a ponta como o texto. Contam para o teu armazenamento e para o do destinatário até cada um apagar a mensagem.',
        },
        matrixApp: {
          title: 'Mensagens pelo Matrix',
          p1: 'Na app, o teu próprio dispositivo é o dispositivo Matrix. Inicias sessão com uma conta existente nas definições, em «Contas Matrix», e as tuas mensagens são cifradas ponta a ponta entre este dispositivo e o interlocutor – o openany.de não as lê. Para que outras apps Matrix como o Element confiem neste dispositivo, verifica-o aí, na conta: com a chave de recuperação do Element ou – se a tua conta ainda não tiver cópia de segurança – o openany cria uma e mostra-te a nova chave uma vez.',
          p2: 'Podes ligar várias contas Matrix. As mensagens de todas partilham o histórico; as novas saem da conta predefinida, as respostas da conta onde a mensagem chegou, e «Tornar predefinido» torna outra conta a predefinida. Se apagares uma mensagem Matrix, desaparece só neste dispositivo; na sala continua lá.',
        },
        email: {
          title: 'E-mail pela tua caixa de correio',
          p1: 'Na app, o e-mail junta-se como mais uma via – não um programa de correio à parte, mas a tua caixa existente, por exemplo no Posteo, mailbox.org ou GMX. Nas definições, em «Caixas de correio», introduzes endereço e palavra-passe; normalmente é uma palavra-passe de aplicação que crias antes no fornecedor. A app procura os servidores sozinha; se não encontrar, introduzes em «Servidores à mão». A palavra-passe fica guardada a sete chaves neste dispositivo, e é o próprio dispositivo que fala com o servidor de correio – o openany.de não vê nenhum e-mail.',
          p2: 'Os e-mails aparecem com o assunto no histórico comum das Mensagens; «Buscar agora» vai buscar os novos de imediato. Se ligares várias caixas, os e-mails novos saem da caixa predefinida e as respostas da caixa onde o e-mail chegou; ao escrever escolhes em «De». Ao apagar decides se um e-mail desaparece só aqui ou vai também para a pasta de reciclagem do servidor. Se o fornecedor classificou algo como spam, aparece um aviso por cima do histórico; aí, «Não é spam» devolve o e-mail à caixa de entrada.',
          p3: 'Por e-mail, os anexos podem ter até 20 MB. Os anexos recebidos guardas nos teus ficheiros com «Guardar» ou na pasta de transferências com «No dispositivo». Se um anexo chegou cifrado, a app pergunta antes: em Ficheiros fica sem cifra e é sincronizado com o openany.de.',
        },
        pgp: {
          title: 'E-mail cifrado com OpenPGP',
          p1: 'Podes cifrar e-mails ponta a ponta com OpenPGP. Para isso, cada caixa precisa de uma chave própria: nas definições, junto à caixa, crias com «Criar chave» – ou, se já usas a caixa com PGP, por exemplo no Thunderbird, importas a existente com «Importar chave», para que os dois programas leiam os mesmos e-mails. A chave secreta está só neste dispositivo. «Guardar cópia de segurança» guarda-a, fechada com uma frase-passe, na pasta de transferências; «Guardar chave pública» dá-te o ficheiro que envias aos outros.',
          p2: 'As chaves públicas dos teus contactos, a app reúne-as sozinha em «Chaves dos contactos», a partir dos e-mails que as trazem. Se faltar alguma, procuras pelo endereço – se quiseres, também em keys.openpgp.org, que fica então a saber por quem perguntas – ou importas de um ficheiro. O melhor é comparar uma vez a impressão digital com o interlocutor; se uma chave mudar, a app avisa.',
          p3: 'Ao escrever ativas «Enviar cifrado»; por baixo vês se a chave do destinatário é conhecida. Se responderes a um e-mail cifrado, o interruptor já está ligado. No histórico, os e-mails têm a marca «cifrado» e, se estiverem assinados, «assinado» – ou um aviso se a assinatura não bater certo.',
        },
        vorOrt: {
          title: 'Mensagens por perto',
          p1: 'Com a via «Por perto» escreves diretamente a dispositivos próximos, sem servidor nenhum. No outro dispositivo o Openany tem de estar aberto, e ambos têm de estar na mesma rede Wi-Fi. Podem escrever-se assim que se conhecerem: os teus próprios dispositivos, membros de um hub comum – ou pessoas que confirmaram presencialmente. Para isso escolhes «Confirmar presencialmente», os dois dispositivos mostram os mesmos 6 algarismos e ambos tocam em «Coincide». Se o outro dispositivo não estiver lá, a mensagem espera e segue no próximo encontro.',
          p2: 'De desconhecidos, de início, nada chega. Se nas definições, em «Dispositivos por perto», ligares «Permitir pedidos de dispositivos por perto», podem enviar-te até 3 mensagens de 500 caracteres cada; estas aparecem em «Pedidos» por cima do histórico, não no próprio histórico. Só podes responder depois da confirmação com os 6 algarismos. Quem não deve enviar-te mais nada, travas com «Bloquear». Se uma mensagem tua foi recusada, mostra «não entregue», e o motivo aparece quando apontas para ela.',
        },
        systemnachrichten: {
          title: 'Notificações e respostas',
          p1: 'Os eventos dos teus hubs – por exemplo, novas partilhas – chegam-te como aviso, se o remetente informar os membros; também o aviso de que alguém te adicionou a um hub e as mensagens de openany.de sobre a tua conta, como o pedido para a confirmares. Os avisos não são cifrados de ponta a ponta. Aqui encontras também a resposta a uma mensagem enviada pelo formulário de suporte.',
        },
      },
    },
    contacts: {
      title: 'Contactos',
      subs: {
        adressbuch: {
          title: 'Contactos e as suas vias',
          p1: 'Os contactos abrem-se pelo mosaico na página inicial ou no topo da página de mensagens; o botão ao lado cria diretamente um novo contacto. Um contacto regista como chegar a alguém: nome, imagem, telefone e e-mail, e ainda quantas vias quiseres – Matrix, Meshtastic, nome no Openany, morada, empresa, aniversário ou outro, cada uma com etiqueta própria como pessoal ou trabalho. O campo de pesquisa encontra contactos pelo nome.',
          p2: 'As vias são clicáveis: um número de telefone liga, um endereço de e-mail abre o programa de correio, um nome no Openany ou um identificador Matrix abre o campo de escrita com o destinatário já preenchido. Os contactos eliminados vão para a reciclagem.',
        },
        verschluesselt: {
          title: 'Cifrado de ponta a ponta',
          p1: 'O teu livro de endereços é cifrado de ponta a ponta, com a mesma chave que as tuas mensagens diretas. Só os teus dispositivos autorizados o podem ler; openany.de guarda-o e sincroniza-o, mas não vê nomes, números nem imagens. Um dispositivo novo só mostra o livro de endereços depois de o autorizares noutro dispositivo.',
        },
        vcard: {
          title: 'Guardar e importar vCard',
          p1: 'Em Definições → Contactos guardas todos os contactos como ficheiro vCard ou importas um de outro programa. Importar só acrescenta: quem já existe é reconhecido pelo identificador ou pelo nome e mantém o que tem. O ficheiro em si não está cifrado – guarda-o em conformidade.',
        },
        imTelefon: {
          title: "Nos contactos do telefone",
          p1: "No Android, Definições → Contactos → «Mostrar nos contactos do telefone» mostra os teus contactos no livro de endereços do telefone, na conta «openany» – quando alguém liga, aparece o nome. Nome, telefone, e-mail, morada, empresa, aniversário, site, alcunha e foto passam para lá; o que lá alterares, criares ou apagares volta. O telefone não conhece os identificadores Matrix, Meshtastic ou Openany – ficam só no openany. Uma foto alterada lá é reposta pelo openany. Desativar retira de lá os contactos.",
        },
        andereProgramme: {
          title: "Para outros programas no computador",
          p1: "No computador, por exemplo o Thunderbird ou os Contactos da Apple leem e escrevem o teu livro de endereços por CardDAV – ativado em Definições → Calendário, com os mesmos dados que para os calendários (ver lá). Seguem todas as vias de contacto, também Matrix, Meshtastic e o nome Openany.",
        },
        sicherheitsnummer: {
          title: 'Número de segurança',
          p1: 'Com o número de segurança verificas que ninguém lê pelo meio – nem o openany.de. Abre-o no contacto, em «Segurança», ou nas mensagens depois de abrires uma conversa a partir de «Por contacto». Comparem os doze grupos de algarismos nos vossos dois dispositivos; se coincidirem, toca em «Coincide». As mensagens dessa pessoa passam a ter um visto junto ao cadeado. Se a chave dela mudar mais tarde, por exemplo após perder um dispositivo, o visto desaparece e um aviso pede-vos para comparar de novo.',
        },
      },
    },
    settings: {
      title: 'Definições e conta',
      subs: {
        module: {
          title: 'Módulos na página inicial',
          p1: 'Aqui escolhes que módulos aparecem como mosaico na página inicial – Notas, Calendário, Disco, Hubs e Contactos. A escolha não esconde nada; todos os módulos continuam acessíveis. Sem escolha, a página inicial mostra a apresentação de boas-vindas.',
        },
        spracheDesign: {
          title: 'Aspeto e idioma',
          p1: 'Em «Aspeto» escolhes entre dois designs: «Nítido», em petróleo com cantos mais marcados e superfície calma, ou «Lilás», com cantos arredondados e fundo com padrão. Ambos existem em claro e escuro – isso mudas à parte com o interruptor no cabeçalho ou no menu do telemóvel. Em «Idioma» escolhes em que idioma o Openany fala contigo: alemão, inglês, espanhol, francês, português, polaco ou ucraniano. Enquanto não escolheres, segue o idioma do teu dispositivo ou navegador; se não estiver disponível, o Openany fala inglês.',
        },
        abgleich: {
          title: 'Sincronizar com o Openany',
          p1: 'Em «Sincronização» ligas a app a openany.de. Se ainda não tens conta, cria uma com «Criar nova conta» (ver «Criar uma conta»). Se já tens um dispositivo ligado, autorizas nele o novo com 6 dígitos (ver «Os meus dispositivos»). Uma conta antiga com palavra-passe liga-se com «Ligar ao openany.de»: a app mostra um código que introduzes e confirmas na página do anyid no navegador. Ligar é opcional: sem ligação tudo fica neste dispositivo, e «Desligar» desfaz isso a qualquer momento sem perder dados aqui.',
          p2: 'Depois, «Sincronizar» sincroniza nos dois sentidos e diz no fim o que foi buscado, enviado ou ignorado. «Voltar a buscar tudo» volta a descarregar tudo em vez de só o que é novo; nada é apagado.',
          p3: 'Com «Atualizar em segundo plano» a app sincroniza sozinha mais ou menos de hora a hora – só com ligação e não com bateria fraca. Em dados móveis só vem a lista, ficheiros e imagens só em Wi-Fi. Em Wi-Fi tenta também com os dispositivos por perto emparelhados, se o Openany estiver aberto neles. No computador, o Openany sincroniza de hora a hora enquanto estiver aberto; com «Continuar em segundo plano» arranca ao iniciar sessão e, ao fechar a janela, fica como ícone na barra de menus.',
        },
        geraeteNah: {
          title: 'Dispositivos por perto',
          p1: 'Os teus próprios dispositivos também se sincronizam diretamente, sem servidor nenhum – desde que estejam na mesma rede Wi-Fi ou hotspot e o Openany esteja aberto em ambos. Em «Dispositivos por perto» encontras-os com «Procurar» e ligas-os com «Emparelhar»: os dois mostram o mesmo código, e em ambos confirmas que coincide. A partir daí sincronizam-se sozinhos enquanto o Openany estiver aberto em ambos, e em cada «Sincronizar»; «Esquecer» desfaz o emparelhamento.',
          p2: 'Em «O teu nome» defines como apareces aos outros presencialmente. É com esse nome que te convidam para hubs – a ti como pessoa, não a um dispositivo; vale para todos os teus dispositivos emparelhados.',
        },
        meineGeraete: {
          title: 'Os meus dispositivos',
          p1: 'O que encriptas ponto a ponto só pode ser lido pelos teus dispositivos; para isso partilham uma chave. Um dispositivo novo mostra 6 dígitos, e introduzes-os aqui num dispositivo já configurado. Se perderes um dispositivo, retiras-lhe o acesso aqui; a chave muda nessa altura. A chave também está no teu ficheiro de cópia de segurança.',
          p2: 'Ao fundo de «Os meus dispositivos» eliminas tu mesmo a tua conta em openany.de – de imediato e com tudo o que lá está. Por segurança, escreves o teu nome de utilizador. Os hubs que ainda têm outros membros passas antes a alguém.',
        },
        fristen: {
          title: 'Confirmar a conta e manter-se ativo',
          p1: 'Para que não se acumulem contas sem uso, openany.de pede-te dois dias depois de criares a conta que a confirmes mais uma vez; no topo aparece uma faixa com «Manter conta». Se não confirmares em 14 dias, a conta é eliminada.',
          p2: 'Se não abrires a app durante quatro meses, chega o mesmo pedido, com um último lembrete duas semanas antes da eliminação. Basta tocar em «Manter conta». Quando uma conta é eliminada, só desaparece o conteúdo em openany.de; o que está nos teus dispositivos fica.',
        },
        speicherGeraet: {
          title: 'Armazenamento neste dispositivo',
          p1: 'Em «Armazenamento neste dispositivo» decides o que fica aqui. «Manter tudo» traz tudo dos teus álbuns, dossiês e pastas enquanto houver espaço; é pensado para computadores e tablets. Com «Álbuns, dossiês e pastas selecionados» só fica aqui o que escolhes em Armazenamento com «Manter neste dispositivo»; de tudo o resto só vês o nome. As novas fotos da câmara vão para o álbum «Câmara». O que não está num álbum, dossiê ou pasta fica no seu dispositivo e nunca é sincronizado.',
          p2: 'Para o servidor vão as notas, os calendários, os contactos, as mensagens e os nomes de álbuns, dossiês e pastas. Fotos e ficheiros só vão se estiverem partilhados num hub, anexados a uma nota, ou se escolheres «Colocar no servidor», um a um ou para todo um álbum, dossiê ou pasta. Tudo o resto é sincronizado diretamente entre os teus dispositivos. «Retirar do servidor» não apaga nada: os teus dispositivos ficam com as suas cópias e a última cópia fica sempre.',
          p3: 'Em «Armazenamento neste dispositivo» vês também quanto está no servidor, dividido por mensagens, notas, mediateca, ficheiros e documentos.',
        },
        sofort: {
          title: 'Notificar de imediato',
          p1: 'Com «Notificar de imediato» a app mantém ligações económicas ao Openany e às tuas caixas de correio, para que novas mensagens e e-mails cheguem logo – sem Google. As caixas de correio não precisam do Openany para isso. O Android mostra por isso um aviso permanente, que podes ocultar nas definições do sistema. No computador, o Openany vigia enquanto estiver aberto e avisa com uma notificação do sistema.',
        },
        sicherung: {
          title: 'Cópia de segurança num ficheiro',
          p1: 'Em «Cópia de segurança», «Criar cópia» gera um ficheiro cifrado com tudo o que o Openany tem neste dispositivo: notas, calendário, contactos, ficheiros, mediateca, hubs, mensagens e as tuas caixas de correio com palavra-passe e a tua própria chave OpenPGP. A app sugere uma frase-passe de seis palavras; aponta-a e guarda-a separada do ficheiro. Sem ela o ficheiro não pode ser aberto – nem por nós.',
          p2: 'Com «Repor cópia» abres um ficheiro assim noutro dispositivo. Substitui tudo o que lá estava; depois a ligação ao openany.de fica removida e voltas a emparelhar os dispositivos próximos. O que identifica um dispositivo não é incluído.',
        },
        tresor: {
          title: 'Credenciais e chaves no cofre',
          p1: 'Aquilo com que este dispositivo se identifica perante o Openany e outros dispositivos fica fechado no dispositivo; a chave fica no porta-chaves do sistema – o Keystore no Android e, no computador, o porta-chaves do macOS, o gestor de credenciais do Windows ou o Secret Service no Linux. Aí ficam também as palavras-passe das tuas caixas de correio e as tuas chaves OpenPGP secretas. «Desligar» só remove as credenciais aqui – um dispositivo é revogado de vez em «Os meus dispositivos».',
        },
      },
    },
    contact: {
      title: 'Denunciar, regras e contacto',
      subs: {
        kontakt: {
          title: 'Contactar-nos',
          p1: 'Envia perguntas, problemas ou pedidos pelo formulário de suporte no fundo desta ajuda; segue cifrado de ponta a ponta para os administradores, e a resposta encontras em Mensagens. Sem conta, contactas-nos pelo formulário em openany.de/denuncia, em «Pergunta ou pedido». Respondemos em alemão ou inglês.',
        },
        bedingungen: {
          title: 'Termos de utilização e privacidade',
          p1: 'Os termos de utilização e a política de privacidade estão em «Sobre o openany», por baixo do formulário de suporte, e a informação legal no fim desta ajuda; os três estão também em openany.de. Em resumo: o openany é gratuito e sem publicidade, o teu conteúdo é cifrado nos teus dispositivos e os servidores estão na Alemanha.',
        },
        melden: {
          title: 'Denunciar conteúdo',
          p1: 'Denuncias conteúdos ou contas que violam a lei ou os termos de utilização pelo formulário em openany.de/denuncia – mesmo sem conta. «Abrir formulário», por cima do formulário de suporte, leva-te lá no idioma da app. A denúncia é cifrada no teu navegador; só os administradores a podem ler.',
          p2: 'Depois de enviar recebes um código de acesso. Com ele vês na mesma página o estado e a nossa resposta e podes perguntar mais. Guarda-o bem – sem ele já não chegas à tua denúncia. É eliminada três meses após a última mensagem.',
        },
        entscheidungen: {
          title: 'Se suspendermos ou eliminarmos a tua conta',
          p1: 'Se uma conta violar os termos de utilização, os administradores podem suspendê-la ou eliminá-la. Se a tua conta estiver suspensa, aparece uma faixa vermelha no topo; um toque mostra desde quando, em que se baseia a decisão e porquê. A sincronização com openany.de fica parada; o que está nos teus dispositivos fica. Quando a suspensão é levantada, tudo continua como antes.',
          p2: 'Se a tua conta for eliminada, a app mostra uma vez o motivo na sincronização seguinte e depois desliga-se; a app continua a funcionar sem openany.de. Se não concordares com uma decisão, toca em «Contestar»: abre o formulário em openany.de/denuncia com «Contestação de uma decisão», e voltamos a analisar a decisão.',
        },
      },
    },
  },
};
