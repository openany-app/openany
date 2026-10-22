// Atenção vue-i18n: nada de @, | ou { sem escape nos textos.
export default {
  title: 'Ajuda',
  intro: 'Aqui encontras descrições e instruções sobre todos os módulos e funções do Openany. Pelo índice à esquerda saltas diretamente para a secção pretendida.',
  version: 'Versão {version}',
  toc: 'Conteúdo',
  sections: {
    start: {
      title: 'Primeiros passos',
      subs: {
        konto: {
          title: 'Conta e início de sessão',
          p1: 'As contas do Openany são criadas exclusivamente mediante convite pessoal – não há registo aberto. O endereço de e-mail é opcional: o Openany funciona também sem ele, mas com e-mail podes repor tu mesmo uma palavra-passe esquecida.',
          p2: 'O início de sessão faz-se com nome de utilizador e palavra-passe através do anyid, o serviço comum de início de sessão. O mesmo início de sessão vale para Openany, anyitem e anytail; por isso a palavra-passe e o segundo fator são geridos lá e não no próprio Openany (ver «Segurança» nas definições).',
        },
        ohneKonto: {
          title: 'A app sem conta',
          p1: 'A app é um programa completo que funciona também sem conta e sem servidor: notas, calendário, disco, livro de endereços e mensagens ficam então só neste dispositivo. O que aqui não está configurado ou não funciona, a app oculta em vez de o mostrar vazio – a via Openany das mensagens, por exemplo, só aparece quando o dispositivo está ligado ao openany.de.',
          p2: 'Se quiseres sincronizar os teus dados com o Openany na rede, toca em «Ligar ao openany.de» nas definições, em «Sincronização»; com os teus outros dispositivos emparelhas-te em «Dispositivos por perto». Depois aparece no cabeçalho e no menu um botão «Sincronizar», que roda enquanto sincroniza.',
        },
        startseite: {
          title: 'A página inicial',
          p1: 'A página inicial mostra mosaicos compactos dos módulos que escolheste: os próximos compromissos, as notas editadas recentemente, os últimos ficheiros com a barra de armazenamento, os teus projetos com contador de não lidas, as mensagens mais recentes da caixa de entrada e os contactos. Cada mosaico leva ao seu módulo; quais aparecem defines nas definições, em «Módulos na página inicial».',
          p2: 'Se não escolheste módulos para a página inicial, aparece em vez disso a apresentação de boas-vindas: mostra todas as funções e deixa-te compor os mosaicos diretamente. A escolha não esconde nada – todas as áreas continuam acessíveis pela barra e pelo menu.',
        },
        navigation: {
          title: 'Menu e estrutura',
          p1: 'No computador há um cabeçalho em cima: à esquerda, o logótipo redondo leva à página inicial e ao lado estão Notas, Calendário, Disco e Projetos. À direita seguem o interruptor claro/escuro, o envelope das mensagens com contador de não lidas e o campo com o teu nome – abre Definições, Ajuda, Reciclagem e Terminar sessão.',
          p2: 'No telemóvel tudo passa para baixo: uma barra fixa mostra os módulos como ícones e, mais à esquerda, fica o botão de menu com o contador de mensagens não lidas. Abre um painel com Mensagens, Definições, Ajuda, Reciclagem, o interruptor claro/escuro e Terminar sessão. O interruptor alterna entre claro, escuro e automático – automático segue a definição do teu dispositivo.',
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
          p1: 'As notas ficam em pastas que se podem aninhar sem limite. Podes mudar o nome, mover e reordenar notas e pastas – assim cresce com o tempo a tua própria coleção de conhecimento. A lupa em cima abre um campo de pesquisa que procura em todas as notas de uma vez, título e conteúdo; aberta dentro de uma pasta, «só nesta pasta» limita-a a essa pasta. O que podes fazer com a pasta aberta está em «Pasta», à direita da barra de caminho.',
          p2: 'Uma pasta inteira com subpastas transfere-se como arquivo ZIP; as notas vão lá dentro como ficheiros Markdown e continuam legíveis em qualquer lado.',
        },
        wikilinks: {
          title: 'Wikilinks e retroligações',
          p1: 'Com dois parênteses retos ligas notas entre si: escreve os parênteses e o título da nota de destino e a ligação surge automaticamente – se a nota ainda não existir, a ligação fica ativa assim que existir. Se no texto deve aparecer outra coisa que não o título, indicas um texto apresentado próprio depois de uma barra vertical; a ligação continua a apontar para a nota certa.',
          p2: 'Cada nota mostra as suas retroligações: uma lista de todas as notas que apontam para ela. Assim reencontras relações sem teres de tomar nota tu.',
        },
        tags: {
          title: 'Etiquetas',
          p1: 'As palavras-chave atribuem-se diretamente no texto com um cardinal antes da palavra; ao escrever, uma lista sugere as etiquetas já usadas. Com uma barra aninhas palavras-chave – por exemplo Projeto/Subtema; um filtro no tema principal encontra também todos os subtemas. Na vista de grafo, as palavras-chave podem aparecer como nós próprios, e um clique numa palavra-chave filtra lá a lista de notas.',
        },
        einfuegen: {
          title: 'Inserir imagens e ficheiros',
          p1: 'Com o botão do clipe na barra de ferramentas inseres conteúdos numa nota – acabados de carregar ou como referência a conteúdos já guardados no Openany. As imagens aparecem como pré-visualização compacta diretamente na nota (a resolução completa está na Galeria); os outros ficheiros, como ligação clicável para transferir.',
          p2: 'Os conteúdos inseridos são arrumados automaticamente: imagens num álbum da Galeria, ficheiros de texto e Office num dossiê dos Documentos, tudo o resto numa pasta de ficheiros – sempre numa área própria para notas. Enquanto a referência estiver em pelo menos uma nota, o ficheiro não pode ir para a reciclagem a partir do Disco ou da Galeria; se removeres a referência do texto, volta a poder ser eliminado.',
        },
        zitate: {
          title: 'Citações e bibliografia',
          p1: 'Para trabalho académico, podes atribuir a uma pasta um ficheiro bibliográfico em formato CSL-JSON – por exemplo uma exportação do Zotero que carregaste antes para o Disco. Na pasta aberta, «Pasta» → «Associar biblioteca» faz a ligação; as subpastas herdam a biblioteca automaticamente.',
          p2: 'Nas notas dessa pasta, o editor sugere fontes adequadas ao citar e insere uma referência curta no texto. Assim as referências surgem enquanto escreves, sem teres de digitar os dados da fonte de cada vez.',
        },
        graph: {
          title: 'Vista de grafo',
          p1: 'A vista de grafo desenha as tuas notas como uma rede: cada nota é um ponto, cada wikilink uma ligação. Os pontos são coloridos pela pasta de nível superior, de modo que as áreas relacionadas se reconhecem pela cor; pontos ocos são notas sem qualquer ligação, e quanto mais ligações uma nota tiver, maior é o seu ponto.',
          p2: 'Cada pasta de primeiro nível é um nível. Se houver vários, escolhes um ou mais na legenda por baixo do grafo – «Todos os níveis» escolhe todos, «Nenhum» limpa a seleção. Em vez de um nível, também uma etiqueta pode ser a entrada: a barra por baixo lista as etiquetas com o seu número, e uma etiqueta escolhida mostra todas as notas que a têm, com a etiqueta como losango cinzento. Com níveis escolhidos, a barra só lista as etiquetas que neles aparecem. Até escolheres algo, o grafo fica vazio, para que coleções grandes não sejam construídas de uma vez; «Mostrar todas as etiquetas» traz todas as etiquetas da barra de uma vez.',
          p3: 'No computador fazes zoom com os botões mais/menos em cima à direita ou com a roda do rato e deslocas a vista com o botão premido; no telemóvel fazes zoom com dois dedos e deslocas com um. Outro botão repõe a vista. Um clique num ponto (sem arrastar) abre a nota. Um clique numa etiqueta filtra a lista de notas por ela.',
        },
        export: {
          title: 'Exportar para PDF',
          p1: 'Cada nota pode ser exportada em PDF – para imprimir, arquivar ou partilhar. A formatação do editor mantém-se. O PDF vai para o processo «Notizen» em Armazenamento → Documentos (as versões anteriores ficam ao lado); «Abrir» no aviso mostra-o de imediato.',
        },
        druckfassung: {
          title: 'Versão para impressão',
          p1: 'Uma pasta torna-se um documento formal para entregar ou imprimir: na pasta aberta escolhes «Versão para impressão» em «Pasta», introduzes título, subtítulo, nome, instituição, data e informações adicionais para a folha de rosto, e decides se queres um índice. Cada nota da pasta torna-se um capítulo, por ordem alfabética do título; com prefixos como «01 Introdução» defines a ordem, e no documento aparece apenas «Introdução». As subpastas entram quando «Incluir subpastas» está marcado: uma subpasta torna-se um título e as suas notas ficam por baixo («2 Fundamentos», «2.1 Conceitos»); uma nota com o mesmo nome da subpasta aparece como texto logo abaixo desse título.',
          p2: 'A composição usa uma letra do tipo Times com títulos sem serifa, margens de 2,5 cm, espaçamento de 1,5 linhas, texto justificado com hifenização; a numeração começa depois da folha de rosto. Os dados ficam guardados na pasta. O PDF é criado no teu dispositivo – da primeira vez demora alguns segundos – e vai para o processo «Druckfassungen» em Armazenamento → Documentos; cada versão é mantida.',
          p3: "Citações: na pasta aberta abres «Pasta» → «Fontes» e registas livros, artigos, sites, decisões judiciais ou leis; cada uma recebe um nome curto. No texto escreves [{'@'}nomecurto] ou [{'@'}nomecurto, p. 12], e ^[Texto] para uma nota de rodapé. No PDF – nota ou versão para impressão – surgem citações Harvard como «(Müller, 2020, p. 12)» e, no fim, a bibliografia. A lista é um ficheiro CSL-JSON em Ficheiros, utilizável também com Zotero e Zettlr. A barra do editor tem um botão para notas de rodapé (número em expoente): insere os parênteses retos, e o texto selecionado torna-se a nota.",
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
          p1: 'Na app do computador podes ligar uma das tuas pastas de notas a uma pasta do sistema: abre a pasta, escolhe «Ligar a uma pasta …» no menu «…» e seleciona o destino. Cada nota fica lá como ficheiro «Id Título.md», as subpastas como subdiretórios – como o Zettlr ou o Obsidian esperam. O que alterares, renomeares, moveres ou criares lá é recolhido pela app enquanto estiver aberta; se ambos os lados mudaram, é criada uma cópia de conflito. Se apagares um ficheiro, a nota vai para o lixo. A pasta fica sem cifra no teu computador; «Desligar pasta» deixa os ficheiros onde estão. As imagens das notas ficam ao lado da nota, com o nome usado no texto; uma imagem nova aí que uma nota referencie passa para a app (galeria, álbum «Notizen: …»).',
        },
      },
    },
    files: {
      title: 'Disco',
      subs: {
        aufbau: {
          title: 'Galeria, ficheiros, documentos',
          p1: 'O Disco divide-se em três separadores no topo da página: a Galeria para fotografias e vídeos, Ficheiros para tudo o resto e Documentos para papéis em dossiês. Ao mudares de separador, a outra área fica como a deixaste – pastas abertas e listas carregadas não se perdem.',
          p2: 'Qual o separador que abre primeiro quando entras no Disco defines nas definições, em «Início do disco»; enquanto não escolheres, são os Documentos.',
        },
        dokumente: {
          title: 'Arquivo de documentos',
          p1: 'No separador «Documentos» organizas ficheiros PDF, Office e de texto em dossiês. Carregas com o botão ou simplesmente arrastando os ficheiros para o dossiê aberto. No telemóvel e no tablet há ao lado um botão de câmara: fotografa um papel e guarda-o logo como documento no dossiê.',
          p2: 'Os dossiês podem aninhar-se sem limite. Podes mudar o nome e mover tanto dossiês como documentos; dossiês inteiros podem ainda ser partilhados em projetos e transferidos como ZIP.',
        },
        texterkennung: {
          title: 'Documentos pesquisáveis',
          p1: 'Se fotografares um documento em papel e o guardares num dossiê – com o botão de câmara ou como fotografia da tua coleção de imagens –, transforma-se automaticamente num PDF com uma camada de texto invisível. O documento parece a tua fotografia, mas pode ser pesquisado, marcado, copiado e lido em voz alta. O reconhecimento corre no teu próprio dispositivo – a imagem não é enviada para lado nenhum.',
          p2: 'Se guardares várias fotografias de uma vez, é-te perguntado se devem formar um documento com várias páginas – como as páginas de uma carta – ou documentos separados. A fotografia em si não é guardada à parte; quem quiser manter a imagem guarda-a na Galeria. Da primeira vez a conversão demora mais, porque o reconhecimento de texto é carregado uma única vez e depois fica guardado.',
          p3: 'Se já houver um PDF digitalizado num dossiê, podes aplicar-lhe o reconhecimento depois: «Reconhecer texto» lê o documento página a página. Se o papel for branco e limpo, as páginas ficam inalteradas e só se acrescenta o texto. Se estiverem cinzentas ou com iluminação irregular – como uma folha fotografada –, são também clareadas; a mensagem final diz-te o que aconteceu. Se o PDF já era pesquisável, não é tocado.',
        },
        dateien: {
          title: 'Gerir ficheiros',
          p1: 'No separador «Ficheiros» crias pastas e carregas ficheiros de qualquer tipo – um a um ou vários de uma vez. Pastas e ficheiros podem mudar de nome e ser movidos, e os ficheiros transferidos de novo; as pastas podem também ser partilhadas em projetos e transferidas como ZIP. O que podes fazer com uma pasta ou um processo – transferir, partilhar, mudar o nome, mover, eliminar – está na pasta aberta, à direita da barra de caminho, em «Pasta» (num processo: «Dossiê»).',
          p2: 'Cada conta tem uma quota de armazenamento; a ocupação atual vês no mosaico da página inicial e nas definições, em «Espaço de armazenamento».',
        },
        suche: {
          title: 'Pesquisar',
          p1: 'A lupa em Ficheiros e Documentos abre um campo de pesquisa que percorre todas as pastas ou dossiês de uma vez, não só o que está aberto. Um clique num resultado abre o ficheiro; ao lado vês onde está, e um clique nessa indicação leva-te à pasta.',
          p2: 'A pesquisa não encontra só pelo nome, mas também no texto: em PDF, textos Word e OpenDocument e ficheiros de texto, CSV e Markdown até 50 MB. O Openany lê esse texto aos poucos em segundo plano enquanto o Disco está aberto; até ter lido tudo, a pesquisa diz-te quantos ficheiros por enquanto só se encontram pelo nome. Num resultado no texto, a lista mostra a passagem com a palavra pesquisada realçada. PDF digitalizados sem camada de texto só aparecem depois de lhes aplicares «Reconhecer texto».',
        },
        pdf: {
          title: 'Ver e editar PDF',
          p1: 'Um clique num PDF abre-o diretamente no Openany, sem transferir. Folheias as páginas, aumentas ou reduzes, ajustas a vista à largura e pesquisas no documento com a lupa – os resultados ficam realçados na página. Se um PDF estiver protegido por palavra-passe, o Openany pede-a primeiro.',
          p2: 'Os formulários do PDF preenches diretamente. Com «Editar» aparece ainda uma barra de ferramentas: «Realçar» marca texto a cores, com a «Caneta» desenhas livremente na página e com «Texto» colocas as tuas próprias linhas na folha – cor, espessura do traço e tamanho de letra escolhes tu, e cada passo pode ser anulado. Os campos de cálculo dos formulários, porém, não calculam, porque o Openany não executa scripts de ficheiros alheios.',
          p3: 'Aos realces e traços podes juntar comentários: escolhe «Comentários» e toca no sítio; a mesma vista lista todos os comentários do documento. Tudo é guardado como anotações PDF normais, que outros programas também mostram. Ao guardar pela primeira vez, o Openany põe a versão anterior na reciclagem, para poderes voltar ao original; se saíres do PDF com alterações por guardar, pergunta antes.',
        },
        galerie: {
          title: 'Galeria de fotografias e álbuns',
          p1: 'No separador «Galeria» estão as tuas fotografias e vídeos – em álbuns com miniaturas ou, sem álbum, por baixo, em «Imagens». Os álbuns podem aninhar-se – por exemplo um álbum por viagem com um subálbum por dia – e mover-se a qualquer momento. O visualizador percorre todas as imagens de um álbum. Para transferir, partilhar, mover ou eliminar um álbum, abre-o e usa «Álbum» à direita da barra de caminho.',
          p2: 'A captura mais recente fica em cima, agrupada por meses com um título para cada mês. Conta a data de captura dos dados da fotografia; se faltar – por exemplo em capturas de ecrã ou imagens editadas –, conta o dia do carregamento.',
          p3: 'Álbuns inteiros transferem-se como ZIP, imagens individuais como ficheiro original. Os álbuns também podem ser partilhados em projetos.',
        },
        videos: {
          title: 'Vídeos e câmara',
          p1: 'Além de fotografias, a Galeria aceita também vídeos. Aparecem como miniatura com uma imagem fixa e reproduzem-se no visualizador. Alguns telemóveis gravam em formato HEVC, que nem todos os navegadores conseguem reproduzir; nesse caso, transfere o vídeo e abre-o com outro programa.',
          p2: 'No telemóvel e no tablet, a Galeria e cada álbum têm dois botões de câmara: um para fotografia, outro para vídeo. A gravação vai diretamente para o sítio que está aberto. No computador não há estes botões; carregas como de costume.',
        },
        karte: {
          title: 'Data e local de captura',
          p1: 'Para cada imagem, o visualizador mostra a data de captura e – se a fotografia o contiver – o local; «Ver no mapa» abre o sítio no OpenStreetMap. Se carregares fotografias de um telemóvel Android, escolhe-as pela app Ficheiros e não pela Galeria: caso contrário, o Android remove a localização antes de a imagem chegar ao Openany.',
        },
        papierkorbSpeicher: {
          title: 'Reciclagem',
          p1: 'Os conteúdos eliminados – documentos, ficheiros, fotografias, notas, contactos, mas também projetos, calendários e compromissos – vão primeiro para a reciclagem comum e podem lá ser restaurados. Após 30 dias, a reciclagem esvazia-se automaticamente; só então o espaço fica definitivamente livre.',
        },
        dateienVerschluesselt: {
          title: 'Encriptado de ponta a ponta',
          p1: 'Os teus ficheiros, processos, fotos e álbuns só são guardados encriptados no openany.de: nomes, pastas, álbuns, datas de captura e local estão num cabeçalho encriptado, e os conteúdos são encriptados com uma chave própria para cada ficheiro. Só os teus dispositivos autorizados os conseguem ler. Na primeira sincronização com esta versão, a app vai buscar todos os originais, encripta-os e volta a carregá-los – demora um pouco e é melhor fazê-lo em Wi-Fi. O que estiver no lixo nessa altura é apagado definitivamente, e as partilhas de álbuns e pastas em projetos terminam.',
        },
      },
    },
    calendar: {
      title: 'Calendário',
      subs: {
        verwalten: {
          title: 'Vários calendários',
          p1: 'Podes ter quantos calendários quiseres lado a lado – p. ex. pessoal, família, associação –, cada um com a sua cor. Mostras ou ocultas cada calendário com um clique. Em cima alternas entre vista mensal e semanal; a semana mostra os compromissos pela hora, e os compromissos passados aparecem discretamente esbatidos.',
        },
        termine: {
          title: 'Criar compromissos',
          p1: 'Crias compromissos com um clique num dia: com título, descrição e hora, ou como evento de dia inteiro ou de vários dias. Também são possíveis repetições – diárias, semanais, mensais ou anuais. Os compromissos existentes editas ou eliminas diretamente na vista. As horas valem no fuso horário definido no teu perfil.',
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
      },
    },
    projects: {
      title: 'Projetos',
      subs: {
        grundlagen: {
          title: 'Projetos e membros',
          p1: 'Um projeto é um espaço de trabalho partilhado: convidas outros utilizadores como membros e os convidados aceitam ou recusam o convite. O proprietário do projeto gere os membros e pode também passar o projeto a outro membro.',
          p2: 'Cada projeto tem quatro separadores: Chat, Planeamento, Membros e Partilhas. Os números nos separadores mostram mensagens de chat por ler ou quanto contêm.',
        },
        chat: {
          title: 'Chat do projeto',
          p1: 'Cada projeto tem um chat comum em tempo real: as mensagens aparecem de imediato para todos os membros, sem recarregar a página. O chat está cifrado de ponta a ponta: só os membros o leem nos seus dispositivos, o openany.de não. Se alguém sair, a chave muda sozinha – deixa de ler o que é novo.',
          p2: 'Qualquer membro pode afixar mensagens importantes – as mensagens afixadas encontram-se rapidamente na barra de afixadas por cima do histórico e também podem lá ser desafixadas.',
          p3: 'Com dois parênteses retos remetes, a meio da frase, para conteúdos do projeto: notas e pastas, ficheiros, álbuns e imagens partilhados. Ao escrever, uma lista de sugestões mostra o que serve e insere a ligação pronta. O botão com o símbolo de corrente ao lado do botão de enviar faz o mesmo – no telemóvel costuma ser mais prático do que procurar dois parênteses.',
          p4: 'Um clique numa ligação leva diretamente ao destino: à nota, à pasta com o ficheiro realçado ou à imagem na vista ampliada. Só podes remeter para o que já está visível no projeto – o que ninguém partilhou não aparece na lista de sugestões nem se torna uma ligação clicável.',
        },
        planung: {
          title: 'O separador Planeamento',
          p1: 'No separador «Planeamento» o projeto reúne as suas votações e listas. Com o botão «Novo» escolhes o tipo certo: marcação de data, inquérito, escala de turnos, lista do que cada um traz ou lista de convidados. Qualquer membro pode criar e votar; apagar só pode quem criou ou o proprietário do projeto.',
          p2: 'As votações são encriptadas de ponta a ponta – incluindo os votos. Os dispositivos dos membros contam sozinhos e respeitam as regras: com lugares limitados, conta quem se inscreveu primeiro. Só quem criou a votação a pode encerrar; numa marcação de data, o teu dispositivo pode adicionar a data escolhida ao teu calendário.',
        },
        terminfindung: {
          title: 'Marcar data',
          p1: 'Ao marcar uma data propões várias opções e todos votam sim, não ou talvez – assim combinam um encontro sem grandes voltas.',
        },
        umfragen: {
          title: 'Inquéritos',
          p1: 'Um inquérito faz uma pergunta com respostas fixas – sim/não ou próprias, anónimo se quiseres. Com opções assinaláveis serve também de lista de tarefas comum; as votações podem ser duplicadas e encerradas.',
        },
        listen: {
          title: 'Escala de turnos, lista do que levar, lista de convidados',
          p1: 'Três tipos de lista poupam trabalho de organização: na escala de turnos crias turnos com vagas e os membros inscrevem-se. A lista do que levar esclarece quem leva o quê – as entradas podem ser assumidas e completadas. A lista de convidados acompanha os convidados (convidados, aceitaram, recusaram), incluindo quem não tem conta no Openany.',
        },
        geteilteKalender: {
          title: 'Partilhar calendários',
          p1: 'Em «Planeamento» partilhas um dos teus calendários com o projeto: os membros veem lá os próximos eventos e todos no seu próprio calendário, em «Aus Projekten» – só leitura. Só tu os podes alterar; as alterações chegam na próxima sincronização. Tudo é encriptado com a chave do projeto. O caixote do lixo junto ao calendário termina a partilha; o teu calendário fica.',
        },
        freigaben: {
          title: 'Partilhar conteúdos',
          p1: 'No separador «Partilhas», «Partilhar» partilha com o projeto uma pasta de notas, um álbum ou uma pasta (ou processo): primeiro escolhes o tipo, depois qual. Tudo continua cifrado ponta a ponta – com uma chave própria por partilha, que muda quando deixas de partilhar ou alguém sai do projeto. O conteúdo fica contigo; quando a partilha termina, guardas tudo.',
          p2: 'Os álbuns e pastas são partilhados só para ver: os membros abrem-nos no projeto, veem subálbuns e subpastas, abrem imagens e PDF e, com «Copiar para mim», guardam uma cópia no seu próprio armazenamento em «De projetos». Só tu os podes alterar.',
          p3: 'As pastas de notas partilham-se com o nível «ver» ou «editar». Os membros abrem-nas no editor de notas habitual e veem também as imagens; com «editar» alteram notas e criam novas. Por agora, só quem é dono da pasta pode inserir imagens.',
        },
        lokal: {
          title: 'Projetos só presenciais',
          p1: 'Na app, «+ Projeto» cria um projeto só neste dispositivo – sem conta e sem servidor; tem a marca «Só presencial». Convidas outras pessoas em «Convidar membro por perto»: no outro dispositivo o Openany tem de estar aberto, na mesma rede Wi-Fi ou hotspot, e ambos os dispositivos mostram os mesmos seis algarismos, que comparam e confirmam. O projeto é gerido pelos dispositivos do proprietário; se houver só um, a app avisa – emparelha antes um segundo dispositivo teu.',
          p2: 'Os membros partilham as suas próprias pastas de notas, pastas e álbuns no projeto em «Partilhar algo», de início só para leitura. As pastas de notas também se podem partilhar para edição: aí só uma pessoa edita uma nota de cada vez, e guarda-se no dispositivo da pessoa a quem a pasta pertence – se esse não estiver por perto, a nota fica só de leitura. O chat chega logo a todos os que estão por perto, e aos restantes na sincronização seguinte. O planeamento – votações e listas – viaja da mesma forma.',
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
        hinweise: {
          title: 'Avisos',
          p1: 'Na app web, Mensagens mostra apenas avisos: notificações dos teus projetos, o aviso de que alguém te adicionou a um projeto, respostas ao formulário de contacto e avisos sobre os teus dispositivos. A pesquisa por cima da lista encontra avisos por texto e remetente.',
          p2: 'As mensagens diretas – também por Matrix, e-mail e por perto – só existem na app. São encriptadas de ponta a ponta e as chaves estão nos teus dispositivos, não no openany.de. Por isso a app web não as pode mostrar nem escrever.',
        },
        suche: {
          title: 'Pesquisar e filtrar',
          p1: 'Por cima do histórico há um campo de pesquisa. Encontra mensagens pelo texto, pelo nome do interlocutor e por identificadores Matrix, e realça os resultados. Por baixo, interruptores restringem a lista: às mensagens não lidas, às que têm anexo e – se usares mais de uma via – a vias concretas como Openany ou Matrix. Os interruptores podem combinar-se.',
          p2: 'Ao lado do campo de pesquisa alternas entre duas vistas: «Histórico» mostra todas as mensagens por ordem temporal, «Por contacto» reúne-as numa linha por pessoa, com a última mensagem e o número de não lidas. Um clique numa linha mostra o histórico com essa pessoa; «Fechar conversa» leva-te de volta a todas.',
        },
        anhaenge: {
          title: 'Anexos',
          p1: 'Pelo Matrix envias também ficheiros: o clipe no campo de escrita anexa um ficheiro até 10 MB, e no telemóvel e no tablet o botão de câmara ao lado tira logo uma fotografia. No histórico, as imagens aparecem em pré-visualização e os outros ficheiros com nome e tamanho; um clique abre imagens e PDF diretamente no Openany, o resto transferes. Pela via Openany vão ficheiros até 15 MB, encriptados de ponta a ponta como o texto. Contam para o teu armazenamento e para o do destinatário até cada um apagar a mensagem.',
        },
        matrixApp: {
          title: 'Mensagens pelo Matrix',
          p1: 'Na app, o teu próprio dispositivo é o dispositivo Matrix. Inicias sessão com uma conta existente nas definições, em «Contas Matrix», e as tuas mensagens são cifradas ponta a ponta entre este dispositivo e o interlocutor – o openany.de não as lê. A verificação do dispositivo por comparação de emojis ainda não é possível na app; por isso, outros programas Matrix mostram-na como dispositivo não verificado, mas a cifragem funciona na mesma.',
          p2: 'Podes ligar várias contas Matrix. As mensagens de todas partilham o histórico; as novas saem da conta predefinida, as respostas da conta onde a mensagem chegou, e «Tornar predefinido» torna outra conta a predefinida. Se apagares uma mensagem Matrix, desaparece só neste dispositivo; na sala continua lá.',
        },
        email: {
          title: 'E-mail pela tua caixa de correio',
          p1: 'Na app, o e-mail junta-se como mais uma via – não um programa de correio à parte, mas a tua caixa existente, por exemplo no Posteo, mailbox.org ou GMX. Nas definições, em «Caixas de correio», introduzes endereço e palavra-passe; normalmente é uma palavra-passe de aplicação que crias antes no fornecedor. A app procura os servidores sozinha; se não encontrar, introduzes em «Servidores à mão». A palavra-passe fica guardada a sete chaves neste dispositivo, e é o próprio dispositivo que fala com o servidor de correio – o openany.de não vê nenhum e-mail.',
          p2: 'Os e-mails aparecem com o assunto no histórico comum das Mensagens; «Buscar agora» vai buscar os novos de imediato. Se ligares várias caixas, os e-mails novos saem da caixa predefinida e as respostas da caixa onde o e-mail chegou; ao escrever escolhes em «De». Ao apagar decides se um e-mail desaparece só aqui ou vai também para a pasta de reciclagem do servidor. Se o fornecedor classificou algo como spam, aparece um aviso por cima do histórico; aí, «Não é spam» devolve o e-mail à caixa de entrada.',
          p3: 'Por e-mail, os anexos podem ter até 15 MB. Os anexos recebidos guardas nos teus ficheiros com «Guardar» ou na pasta de transferências com «No dispositivo». Se um anexo chegou cifrado, a app pergunta antes: em Ficheiros fica sem cifra e é sincronizado com o openany.de.',
        },
        pgp: {
          title: 'E-mail cifrado com OpenPGP',
          p1: 'Podes cifrar e-mails ponta a ponta com OpenPGP. Para isso, cada caixa precisa de uma chave própria: nas definições, junto à caixa, crias com «Criar chave» – ou, se já usas a caixa com PGP, por exemplo no Thunderbird, importas a existente com «Importar chave», para que os dois programas leiam os mesmos e-mails. A chave secreta está só neste dispositivo. «Guardar cópia de segurança» guarda-a, fechada com uma frase-passe, na pasta de transferências; «Guardar chave pública» dá-te o ficheiro que envias aos outros.',
          p2: 'As chaves públicas dos teus contactos, a app reúne-as sozinha em «Chaves dos contactos», a partir dos e-mails que as trazem. Se faltar alguma, procuras pelo endereço – se quiseres, também em keys.openpgp.org, que fica então a saber por quem perguntas – ou importas de um ficheiro. O melhor é comparar uma vez a impressão digital com o interlocutor; se uma chave mudar, a app avisa.',
          p3: 'Ao escrever ativas «Enviar cifrado»; por baixo vês se a chave do destinatário é conhecida. Se responderes a um e-mail cifrado, o interruptor já está ligado. No histórico, os e-mails têm a marca «cifrado» e, se estiverem assinados, «assinado» – ou um aviso se a assinatura não bater certo.',
        },
        vorOrt: {
          title: 'Mensagens por perto',
          p1: 'Com a via «Por perto» escreves diretamente a dispositivos próximos, sem servidor nenhum. No outro dispositivo o Openany tem de estar aberto, e ambos têm de estar na mesma rede Wi-Fi. Podem escrever-se assim que se conhecerem: os teus próprios dispositivos, membros de um projeto comum – ou pessoas que confirmaram presencialmente. Para isso escolhes «Confirmar presencialmente», os dois dispositivos mostram os mesmos 6 algarismos e ambos tocam em «Coincide». Se o outro dispositivo não estiver lá, a mensagem espera e segue no próximo encontro.',
          p2: 'De desconhecidos, de início, nada chega. Se nas definições, em «Dispositivos por perto», ligares «Permitir pedidos de dispositivos por perto», podem enviar-te até 3 mensagens de 500 caracteres cada; estas aparecem em «Pedidos» por cima do histórico, não no próprio histórico. Só podes responder depois da confirmação com os 6 algarismos. Quem não deve enviar-te mais nada, travas com «Bloquear». Se uma mensagem tua foi recusada, mostra «não entregue», e o motivo aparece quando apontas para ela.',
        },
        systemnachrichten: {
          title: 'Notificações e respostas',
          p1: 'Os eventos dos teus projetos – como novos inquéritos ou partilhas – chegam-te como aviso se o remetente optar por informar os membros; tal como o aviso de que alguém te adicionou a um projeto. Os avisos não são encriptados e aparecem na app e na app web. Aqui também encontras a resposta a uma mensagem enviada pelo formulário de contacto.',
        },
      },
    },
    contacts: {
      title: 'Contactos',
      subs: {
        adressbuch: {
          title: 'Contactos e as suas vias',
          p1: 'Os contactos abrem-se no topo da página de mensagens; o botão ao lado cria diretamente um novo contacto. Um contacto regista como chegar a alguém: nome, imagem, telefone e e-mail, e ainda quantas vias quiseres – Matrix, Meshtastic, nome no Openany, morada, empresa, aniversário ou outro, cada uma com etiqueta própria como pessoal ou trabalho. O campo de pesquisa encontra contactos pelo nome.',
          p2: 'As vias são clicáveis: um número de telefone liga, um endereço de e-mail abre o programa de correio, um nome no Openany ou um identificador Matrix abre o campo de escrita com o destinatário já preenchido. Os contactos eliminados vão para a reciclagem.',
        },
        verschluesselt: {
          title: 'Cifrado de ponta a ponta',
          p1: 'O teu livro de endereços está cifrado de ponta a ponta, com a mesma chave das tuas mensagens diretas. Só os teus dispositivos aprovados o podem ler; o openany.de guarda-o e sincroniza-o, mas não vê nomes, números nem imagens. Um dispositivo novo só mostra o livro de endereços depois de o aprovares noutro dispositivo. Por isso a aplicação web já não tem livro de endereços.',
        },
        vcard: {
          title: 'Guardar e importar vCard',
          p1: 'Em Definições → Contactos guardas todos os contactos como ficheiro vCard ou importas um de outro programa. Importar só acrescenta: quem já existe é reconhecido pelo identificador ou pelo nome e mantém o que tem. O ficheiro em si não está cifrado – guarda-o em conformidade.',
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
        profil: {
          title: 'Perfil, fuso horário e armazenamento',
          p1: 'O teu nome no Openany é fixo e não pode ser alterado. O endereço de e-mail é opcional e serve apenas fins da conta, como repor a palavra-passe, nunca publicidade; só o podes alterar com a tua palavra-passe. O fuso horário define como se entendem as horas dos teus compromissos – um botão usa o fuso do dispositivo. Por baixo, «Espaço de armazenamento» mostra quanto ocupas.',
        },
        module: {
          title: 'Módulos na página inicial',
          p1: 'Aqui escolhes que módulos aparecem como mosaico na página inicial – Notas, Calendário, Disco, Projetos, Mensagens e Contactos. A escolha não esconde nada; todos os módulos continuam acessíveis. Sem escolha, a página inicial mostra a apresentação de boas-vindas.',
        },
        sicherheit: {
          title: 'Segurança: palavra-passe e segundo fator',
          p1: 'A palavra-passe, o segundo fator e os teus dispositivos estão no anyid, não no Openany. O cartão «Conta e segurança» nas definições leva-te lá diretamente com três ligações: «Mudar a palavra-passe», «Segundo fator» e «Os teus dispositivos», onde vês os dispositivos associados e os podes remover. As alterações valem ao mesmo tempo para Openany, anyitem e anytail.',
          p2: 'O segundo fator é um código de uma app de autenticação, pedido depois da palavra-passe; sem ele não é possível criar palavras-passe de aplicação. Ao configurá-lo recebes códigos de recuperação; guarda-os – sem eles e sem a app não voltas a entrar. Se tiveres um endereço de e-mail guardado, podes repor tu mesmo uma palavra-passe esquecida.',
        },
        spracheDesign: {
          title: 'Aspeto e idioma',
          p1: 'Em «Aspeto» escolhes entre dois designs: «Nítido», em petróleo com cantos mais marcados e superfície calma, ou «Lilás», com cantos arredondados e fundo com padrão. Ambos existem em claro e escuro – isso mudas à parte com o interruptor no cabeçalho ou no menu do telemóvel. Em «Idioma» escolhes em que idioma o Openany fala contigo: alemão, inglês, espanhol, francês, português, polaco ou ucraniano. Enquanto não escolheres, segue o idioma do teu dispositivo ou navegador; se não estiver disponível, o Openany fala inglês.',
        },
        zugriff: {
          title: 'Acesso externo: palavras-passe de aplicação',
          p1: 'Os programas no teu computador não iniciam sessão com a palavra-passe da tua conta, mas com uma palavra-passe de aplicação própria. Ao criá-la, dás-lhe um nome e escolhes a finalidade «Sincronização» – para programas que espelham a tua conta e escrevem de volta. As notas só são guardadas cifradas no servidor, por isso já não há acesso por WebDAV. Só podes criá-la se tiveres iniciado sessão com um segundo fator.',
          p2: 'A nova palavra-passe é mostrada exatamente uma vez – copia-a de imediato. No programa introduzes o teu nome no Openany como nome de utilizador. A lista mostra quando cada palavra-passe foi usada pela última vez; se revogares uma, o programa perde o acesso de imediato.',
        },
        abgleich: {
          title: 'Sincronizar com o Openany',
          p1: 'Em «Sincronização» ligas a app à tua conta com «Ligar ao openany.de». A app mostra um código que escreves e confirmas na página do anyid no navegador. Essa confirmação no navegador não é um desvio, mas a prova de que és mesmo tu a acrescentar o dispositivo. Ligar é opcional: sem ligação, tudo fica neste dispositivo, e «Desligar» desfaz a ligação a qualquer momento sem perder dados aqui.',
          p2: 'Depois, «Sincronizar» sincroniza nos dois sentidos e diz no fim o que foi buscado, enviado ou ignorado. «Voltar a buscar tudo» volta a descarregar tudo em vez de só o que é novo; nada é apagado.',
          p3: 'Com «Atualizar em segundo plano» a app sincroniza sozinha mais ou menos de hora a hora – só com ligação e não com bateria fraca. Em dados móveis só vem a lista, ficheiros e imagens só em Wi-Fi. Os dispositivos por perto ficam de fora; para eles a app tem de estar aberta. No computador, o Openany sincroniza de hora a hora enquanto estiver aberto.',
        },
        geraeteNah: {
          title: 'Dispositivos por perto',
          p1: 'Os teus próprios dispositivos também se sincronizam diretamente, sem servidor nenhum – desde que estejam na mesma rede Wi-Fi ou hotspot e o Openany esteja aberto em ambos. Em «Dispositivos por perto» encontras-os com «Procurar» e ligas-os com «Emparelhar»: os dois mostram o mesmo código, e em ambos confirmas que coincide. A partir daí sincronizam-se entre si em cada «Sincronizar»; «Esquecer» desfaz o emparelhamento.',
          p2: 'Em «O teu nome» defines como apareces aos outros presencialmente. É com esse nome que te convidam para projetos – a ti como pessoa, não a um dispositivo; vale para todos os teus dispositivos emparelhados.',
        },
        speicherGeraet: {
          title: 'Armazenamento neste dispositivo',
          p1: 'Em «Armazenamento neste dispositivo» decides quanto dos teus ficheiros e imagens a app guarda à mão. «Quando preciso» mostra todos os ficheiros na lista, mas só vai buscar o conteúdo ao abrir. Com «Pastas e álbuns escolhidos» fica sempre no dispositivo o que marcaste no Disco com «Manter neste dispositivo», subpastas incluídas. «Manter tudo» vai buscar após cada sincronização o que falta, enquanto houver espaço.',
        },
        sofort: {
          title: 'Notificar de imediato',
          p1: 'Com «Notificar de imediato» a app mantém ligações económicas ao Openany e às tuas caixas de correio, para que novas mensagens e e-mails cheguem logo – sem Google. As caixas de correio não precisam do Openany para isso. O Android mostra por isso um aviso permanente, que podes ocultar nas definições do sistema. No computador, o Openany vigia enquanto estiver aberto e avisa com uma notificação do sistema.',
        },
        sicherung: {
          title: 'Cópia de segurança num ficheiro',
          p1: 'Em «Cópia de segurança», «Criar cópia» gera um ficheiro cifrado com tudo o que o Openany tem neste dispositivo: notas, calendário, contactos, ficheiros, galeria, projetos, mensagens e as tuas caixas de correio com palavra-passe e a tua própria chave OpenPGP. A app sugere uma frase-passe de seis palavras; aponta-a e guarda-a separada do ficheiro. Sem ela o ficheiro não pode ser aberto – nem por nós.',
          p2: 'Com «Repor cópia» abres um ficheiro assim noutro dispositivo. Substitui tudo o que lá estava; depois a ligação ao openany.de fica removida e voltas a emparelhar os dispositivos próximos. O que identifica um dispositivo não é incluído.',
        },
        tresor: {
          title: 'Credenciais e chaves no cofre',
          p1: 'Aquilo com que este dispositivo se identifica perante o Openany e outros dispositivos está guardado a sete chaves no dispositivo; a chave fica no porta-chaves do sistema – no Android, o Keystore; no computador, o Porta-chaves do macOS, o Gestor de Credenciais do Windows ou o Secret Service no Linux. Lá estão também as palavras-passe das tuas caixas de correio e as tuas chaves OpenPGP secretas. «Desligar» remove as credenciais só aqui – a revogação definitiva faz-se na lista de dispositivos do anyid e nas palavras-passe de aplicação do Openany.',
        },
      },
    },
    contact: {
      title: 'Ajuda e contacto',
      subs: {
        kontakt: {
          title: 'Contactar os operadores',
          p1: 'Dúvidas, problemas ou ideias? A ligação «Contacto» no rodapé leva-te ao formulário de contacto. A tua mensagem chega aos administradores como aviso, e encontras a resposta mais tarde em Mensagens.',
        },
        bedingungen: {
          title: 'Termos de utilização',
          p1: 'Os termos de utilização da beta e a informação legal estão sempre disponíveis nas ligações do rodapé. Em resumo: durante a beta, o Openany é gratuito e sem publicidade, e os teus dados estão em servidores na Alemanha. Só sai o que tu próprio envias – por exemplo mensagens por Matrix ou e-mail.',
        },
      },
    },
  },
};
