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
          p1: 'As notas ficam em pastas que se podem aninhar sem limite. Podes mudar o nome, mover e reordenar notas e pastas – assim cresce com o tempo a tua própria coleção de conhecimento. O campo de pesquisa por cima da lista pesquisa em todas as notas de uma vez, título e conteúdo, independentemente da pasta aberta.',
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
          p1: 'Para trabalho académico, podes atribuir a uma pasta um ficheiro bibliográfico em formato CSL-JSON – por exemplo uma exportação do Zotero que carregaste antes para o Disco. O botão «Associar biblioteca» da pasta faz a ligação; as subpastas herdam a biblioteca automaticamente.',
          p2: 'Nas notas dessa pasta, o editor sugere fontes adequadas ao citar e insere uma referência curta no texto. Assim as referências surgem enquanto escreves, sem teres de digitar os dados da fonte de cada vez.',
        },
        graph: {
          title: 'Vista de grafo',
          p1: 'A vista de grafo desenha as tuas notas como uma rede: cada nota é um ponto, cada wikilink uma ligação. Os pontos são coloridos pela pasta de nível superior, de modo que as áreas relacionadas se reconhecem pela cor; pontos ocos são notas sem qualquer ligação, e quanto mais ligações uma nota tiver, maior é o seu ponto.',
          p2: 'Cada pasta de primeiro nível é um nível. Se houver vários, escolhes um ou mais na legenda por baixo do grafo – «Todos os níveis» escolhe todos, «Nenhum» limpa a seleção. Em vez de um nível, também uma etiqueta pode ser a entrada: a barra por baixo lista as etiquetas com o seu número, e uma etiqueta escolhida mostra todas as notas que a têm, com a etiqueta como losango cinzento. Com níveis escolhidos, a barra só lista as etiquetas que neles aparecem. Até escolheres algo, o grafo fica vazio, para que coleções grandes não sejam construídas de uma vez; «Mostrar todas as etiquetas» traz todas as etiquetas da barra de uma vez.',
          p3: 'No computador fazes zoom com os botões mais/menos em cima à direita ou com a roda do rato e deslocas a vista com o botão premido; no telemóvel fazes zoom com dois dedos e deslocas com um. Outro botão repõe a vista. Um clique num ponto (sem arrastar) abre a nota. Um clique numa etiqueta filtra a lista de notas por ela.',
        },
        uebersicht: {
          title: 'Índice automático',
          p1: 'Além do grafo há um índice automático: lista todos os títulos de notas de A a Z e mostra por baixo uma árvore expansível das tuas palavras-chave (as etiquetas aninhadas aparecem como ramos). Um clique num título abre a nota, um clique numa palavra-chave filtra a lista – sem teres de manter tu um índice.',
        },
        export: {
          title: 'Exportar para PDF',
          p1: 'Cada nota pode ser transferida em PDF – para imprimir, arquivar ou partilhar. A formatação do editor mantém-se.',
        },
        papierkorb: {
          title: 'Reciclagem',
          p1: 'As notas eliminadas vão para a reciclagem (no menu por baixo do teu nome; no telemóvel, no botão de menu) e podem lá ser restauradas ou eliminadas definitivamente. Ao eliminar uma pasta decides se as notas que contém são eliminadas também ou movidas para a pasta superior.',
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
          p1: 'No separador «Ficheiros» crias pastas e carregas ficheiros de qualquer tipo – um a um ou vários de uma vez. Pastas e ficheiros podem mudar de nome e ser movidos, e os ficheiros transferidos de novo; as pastas podem também ser partilhadas em projetos e transferidas como ZIP.',
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
          p1: 'No separador «Galeria» estão as tuas fotografias e vídeos – em álbuns com miniaturas ou, sem álbum, por baixo, em «Imagens». Os álbuns podem aninhar-se – por exemplo um álbum por viagem com um subálbum por dia – e mover-se a qualquer momento. O visualizador percorre todas as imagens de um álbum.',
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
          p1: 'Os conteúdos eliminados – documentos, ficheiros, fotografias, notas, contactos, mas também projetos, quadros, calendários e compromissos – vão primeiro para a reciclagem comum e podem lá ser restaurados. Após 30 dias, a reciclagem esvazia-se automaticamente; só então o espaço fica definitivamente livre.',
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
        tagesansicht: {
          title: 'Vista do dia',
          p1: 'Um clique no número de um dia abre a vista do dia – uma revisão em vez de uma grelha, pensada para a pergunta da véspera: de que precisa a criança amanhã? Logo em cima, o plano de guarda diz com quem está («contigo» ou com quem mais). Por baixo seguem por ordem as aulas do horário, cada uma com o que há para ela; testes e outros prazos dos próximos dias aparecem já antecipadamente com «daqui a … dias». Com as setas passas para o dia anterior ou seguinte.',
          p2: 'Em cada aula anotas rapidamente um trabalho com «Trabalho de casa»; vai parar como cartão com a disciplina ao quadro do projeto. «Caderno» abre a pasta da disciplina. O que tem prazo mas não pertence a nenhuma aula aparece em «Sem tempo letivo». Um clique numa entrada leva-te ao seu lugar no projeto.',
        },
        ausProjekten: {
          title: 'Dos projetos',
          p1: 'Em «Dos projetos», na lista de calendários, mostras o que os teus projetos acrescentam ao calendário: horário escolar, guarda e prazos – cartões e marcos com data limite. Cada fonte pode mostrar-se ou ocultar-se à parte, e os planos novos aparecem sozinhos. As entradas são só de leitura; um clique mostra os pormenores, e «Abrir no projeto» leva-te ao sítio onde as alteras.',
        },
        abos: {
          title: 'Subscrições de calendário',
          p1: 'Calendários externos – por exemplo feriados ou o calendário de jogos do clube – integras através de um URL de subscrição em formato ICS. Os calendários subscritos atualizam-se automaticamente. Os seus compromissos são só de leitura no Openany: alteram-se onde o calendário tem origem, porque cada atualização volta a substituir o conteúdo pelo original.',
          p2: 'Ao contrário, podes partilhar os teus calendários com uma ligação de subscrição: o ícone de feed na lista de calendários cria um URL secreto que pode ser subscrito no Google, Apple ou Outlook – as alterações chegam lá automaticamente. Trata a ligação como uma palavra-passe; com «Renovar ligação» a antiga deixa de valer de imediato.',
        },
        importExport: {
          title: 'Importar e exportar',
          p1: 'Calendários existentes importas como ficheiro ICS, e da mesma forma exportas os teus calendários – portabilidade total nos dois sentidos, sem ficar preso ao Openany.',
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
          p1: 'Cada projeto tem um chat comum em tempo real: as mensagens aparecem de imediato para todos os membros, sem recarregar a página.',
          p2: 'Qualquer membro pode afixar mensagens importantes – as mensagens afixadas encontram-se rapidamente na barra de afixadas por cima do histórico e também podem lá ser desafixadas.',
          p3: 'Com dois parênteses retos fazes referência, a meio de uma frase, a conteúdos do projeto: notas, quadros, roteiros e coleções de locais com os respetivos cartões, marcos e locais, bem como pastas, ficheiros, álbuns e imagens partilhados. Ao escrever, uma lista de sugestões mostra o que encaixa e insere a referência pronta. O botão com o ícone de corrente ao lado do botão de enviar faz o mesmo – no telemóvel costuma ser mais cómodo do que procurar dois parênteses.',
          p4: 'Um clique numa referência leva diretamente ao destino: à nota, ao quadro com o cartão aberto, à pasta com o ficheiro destacado ou à imagem no visualizador. Só podes fazer referência ao que já é visível no projeto – o que ninguém partilhou não aparece na lista de sugestões nem fica clicável.',
        },
        planung: {
          title: 'O separador Planeamento',
          p1: 'No separador «Planeamento» o projeto reúne as suas ferramentas de planeamento. Com o botão «Novo» escolhes o tipo adequado: Marcar data, Inquérito, Escala de turnos, Lista do que levar, Lista de convidados, Quadro, Roteiro, Locais, Férias escolares, Horário escolar ou Plano de guarda. O modelo «Predefinição: escola» cria de uma vez férias escolares, horário, testes e um quadro de trabalhos de casa. Ao fazê-lo escolhes o teu estado federado, para que as férias fiquem logo lá, e, se quiseres, o horário de toques da tua escola. Qualquer membro pode criar e editar; eliminar pode quem criou ou o proprietário do projeto.',
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
        boards: {
          title: 'Quadros Kanban',
          p1: 'Nos quadros Kanban organizam tarefas em colunas e cartões livres – por exemplo «Pendente», «Em curso», «Feito». Os cartões movem-se arrastando; as alterações aparecem em direto para todos os membros. Um cartão pode ainda ter um local das coleções de locais do projeto e uma disciplina – assim um trabalho de casa fica ligado à aula certa.',
        },
        roadmap: {
          title: 'Roteiro',
          p1: 'O roteiro mostra marcos com data e estado numa linha temporal – assim toda a equipa vê o que deve estar pronto e quando. Os marcos podem ser marcados como alcançados; os que estão em atraso ficam assinalados. Tal como os cartões, os marcos podem ter um local e uma disciplina, por exemplo um teste.',
        },
        orte: {
          title: 'Locais',
          p1: 'Em Locais reúnem pontos no OpenStreetMap – pontos de encontro, moradas, estacionamentos. Colocas o ponto com um clique no mapa, com nome e nota opcionais; um clique no nome de um mosaico de local abre o ponto diretamente no OpenStreetMap. Depois de reunidos, escolhes os locais noutros sítios sem mais: em cartões, marcos, disciplinas e entradas do horário escolar ou do plano de guarda.',
        },
        schuljahr: {
          title: 'Férias escolares',
          p1: 'As férias escolares registam um ano letivo: o período e os dias livres. As férias do teu estado federado (Alemanha) podem ser importadas como ficheiro ICS; dias de formação e feriados avulsos acrescentas à mão. A diferença conta: nas férias costuma valer outro acordo de guarda, num dia livre avulso continua o ritmo habitual. A importação só acrescenta e não substitui nada do que introduziste.',
        },
        wochenplan: {
          title: 'Horário escolar e plano de guarda',
          p1: 'Ambos são a mesma peça: uma grelha semanal de segunda a domingo que se repete semana após semana. O horário escolar tem horas e disciplinas, o plano de guarda dias inteiros e uma pessoa – «quem fica com a criança e quando». Se lhe associares férias escolares, as aulas ficam de fora nas férias automaticamente; um plano de guarda pode limitar-se ao tempo de aulas ou às férias. Se o vosso ritmo muda de semana para semana, passas para semanas A/B: ou de duas em duas semanas a partir da segunda-feira da semana A, ou por semanas do calendário pares e ímpares, se o vosso acordo assim o diz – nesse caso, porém, o ritmo salta nos anos com 53 semanas.',
          p2: 'Por baixo da grelha estão os períodos. Substituem-na no seu intervalo e cobrem o que não é ritmo: as férias de verão divididas ao meio, um fim de semana trocado, uma visita de estudo. O fuso horário pertence ao plano e não a quem o vê – assim o horário mostra a mesma hora a ambos os pais, mesmo que um viva no estrangeiro.',
        },
        faecher: {
          title: 'Disciplinas',
          p1: 'As disciplinas de um projeto geres em «Disciplinas» no horário escolar: nome, abreviatura, professor, cor e, se quiseres, um local. Pertencem ao projeto e não a um plano concreto, por isso sobrevivem à mudança de semestre. Uma disciplina define o que uma aula mostra no horário e liga cartões de trabalhos de casa e testes à aula certa. Se apagares uma disciplina, cartões, marcos e aulas perdem apenas a referência – eles próprios ficam.',
        },
        hefte: {
          title: 'Cadernos das disciplinas',
          p1: 'Para cada disciplina, «Criar pasta e partilhar» cria uma pasta de notas – o caderno – e partilha-a logo no projeto. A pasta é tua, não do projeto, e conta para o teu espaço; por isso a disciplina indica de quem é a pasta. Um projeto não possui conteúdos próprios: o que lá está, foram membros que partilharam.',
        },
        planAbo: {
          title: 'Subscrever o horário e a guarda',
          p1: 'Com «Subscrição» crias uma ligação com a qual o horário escolar ou o plano de guarda podem ser subscritos no Google, Apple ou Thunderbird – sem abrir o Openany. A ligação vale para o tipo, não para um plano concreto: se tiverem a guarda das férias num segundo plano, ela está na mesma subscrição. Os locais só vão incluídos se o pedires expressamente – a ligação é pública e não exige sessão, e com locais as vossas moradas estariam por trás. Os programas de calendário só vão buscar a subscrição de poucas em poucas horas; «Renovar» invalida logo a ligação antiga, «Revogar» desativa-a.',
        },
        freigaben: {
          title: 'Partilhar conteúdos',
          p1: 'No separador «Partilhas» está tudo o que os membros disponibilizaram ao projeto: pastas de notas, álbuns e pastas e dossiês do Disco. Partilha-se onde o conteúdo está – nas Notas, na Galeria ou no Disco –, com o nível «Só leitura» ou «Editar». Uma partilha vale para todos os membros do projeto, é herdada por subpastas e subálbuns e pode ser removida a qualquer momento. Os conteúdos continuam a ser do proprietário; quando a partilha termina, fica com tudo.',
          p2: 'Um clique numa partilha abre-a no projeto: notas no espaço de trabalho habitual com pesquisa, retroligações e um grafo de todas as notas partilhadas do projeto, álbuns como grelha de fotografias com visualizador, pastas como lista de ficheiros. Com «Editar», os membros podem também contribuir – escrever notas, carregar fotografias, guardar ficheiros e criar subpastas; «Só leitura» permite ver e transferir.',
        },
        projektNotizen: {
          title: 'Notas de projeto partilhadas',
          p1: 'Numa pasta de notas partilhada com «Editar», os membros podem criar, editar e eliminar notas diretamente no projeto e criar subpastas; as imagens e ficheiros incorporados são vistos por todos os membros, e quem pode editar pode também inserir os seus. Tudo o que for criado – incluindo imagens e ficheiros carregados – pertence ao proprietário da pasta e conta para o armazenamento dele; as notas eliminadas vão para a reciclagem dele, por isso nada se perde definitivamente.',
        },
        ki: {
          title: 'IA no projeto',
          p1: 'Um projeto pode ter uma IA ligada. Quem a configura é o proprietário do projeto, no separador «Membros», por baixo da lista de membros: fornecedor, modelo e uma chave de API própria. A chave é guardada cifrada e depois não é mostrada a ninguém, nem ao proprietário; os custos ficam a cargo da conta a que a chave pertence. No mesmo sítio, todos os membros veem se há uma IA ligada, a que fornecedor e quem a configurou.',
          p2: 'Pergunta-se a partir de conteúdos partilhados: por baixo de uma nota com «Perguntar à IA», em documentos de dossiês e pastas partilhados (PDF, Word, OpenDocument e ficheiros de texto) e – se o modelo entender imagens – em fotografias no visualizador de um álbum partilhado. Escreves um pedido, como «Resume-o em três frases», e recebes a resposta para copiar; quem pode editar a nota pode também inseri-la diretamente no fim.',
          p3: 'Importante: o que perguntas sai do Openany. Por cima do campo de entrada diz sempre o que vai para que fornecedor – todo o texto da nota, o texto do documento ou a fotografia, reduzida e sem local de captura. PDF digitalizados sem camada de texto precisam antes do reconhecimento de texto; documentos muito longos são recusados.',
        },
        benachrichtigungen: {
          title: 'Avisar os membros',
          p1: 'Com novos inquéritos, marcações de data ou partilhas podes, se quiseres, avisar os membros por mensagem direta – assim ninguém perde as novidades do projeto.',
        },
        lokal: {
          title: 'Projetos só presenciais',
          p1: 'Na app, «+ Projeto» cria um projeto só neste dispositivo – sem conta e sem servidor; tem a marca «Só presencial». Convidas outras pessoas em «Convidar membro por perto»: no outro dispositivo o Openany tem de estar aberto, na mesma rede Wi-Fi ou hotspot, e ambos os dispositivos mostram os mesmos seis algarismos, que comparam e confirmam. O projeto é gerido pelos dispositivos do proprietário; se houver só um, a app avisa – emparelha antes um segundo dispositivo teu.',
          p2: 'Os membros partilham as suas próprias pastas de notas, pastas e álbuns no projeto em «Partilhar algo», de início só para leitura. As pastas de notas também se podem partilhar para edição: aí só uma pessoa edita uma nota de cada vez, e guarda-se no dispositivo da pessoa a quem a pasta pertence – se esse não estiver por perto, a nota fica só de leitura. O chat chega logo a todos os que estão por perto, e aos restantes na sincronização seguinte. O planeamento – quadros, roteiros, locais e o planeamento escolar – viaja da mesma forma.',
          p3: 'O resto sincronizas com «Sincronizar» assim que houver um membro por perto. O proprietário pode remover membros; quem quiser sair fá-lo com «Sair», o que só funciona se outro membro estiver por perto para ficar a saber. O que quem saiu tinha partilhado desaparece para os outros; o que já tinham fica nos seus dispositivos.',
        },
      },
    },
    messages: {
      title: 'Mensagens',
      subs: {
        direkt: {
          title: 'Mensagens diretas',
          p1: 'No computador chegas às mensagens pelo envelope no cabeçalho; no telemóvel, pelo botão de menu em baixo à esquerda. Com «Mensagem» em cima à direita escreves diretamente a qualquer outro utilizador – como destinatário basta o nome dele no Openany. As mensagens novas aparecem de imediato, sem recarregar a página, e as ligações nelas são clicáveis.',
          p2: 'Por baixo de cada mensagem recebida, «Responder» abre o campo de escrita com o remetente já preenchido, pela mesma via por onde a mensagem chegou. As mensagens longas aparecem recolhidas; «Ler mais» mostra-as inteiras. O contador de não lidas mostra a qualquer momento se há algo novo – no computador, no envelope; no telemóvel, no botão de menu da barra inferior.',
        },
        suche: {
          title: 'Pesquisar e filtrar',
          p1: 'Por cima do histórico há um campo de pesquisa. Encontra mensagens pelo texto, pelo nome do interlocutor e por identificadores Matrix, e realça os resultados. Por baixo, interruptores restringem a lista: às mensagens não lidas, às que têm anexo e – se usares mais de uma via – a vias concretas como Openany ou Matrix. Os interruptores podem combinar-se.',
          p2: 'Ao lado do campo de pesquisa alternas entre duas vistas: «Histórico» mostra todas as mensagens por ordem temporal, «Por contacto» reúne-as numa linha por pessoa, com a última mensagem e o número de não lidas. Um clique numa linha mostra o histórico com essa pessoa; «Fechar conversa» leva-te de volta a todas.',
        },
        anhaenge: {
          title: 'Anexos',
          p1: 'Pelo Matrix envias também ficheiros: o clipe no campo de escrita anexa um ficheiro até 10 MB, e no telemóvel e no tablet o botão de câmara ao lado tira logo uma fotografia. No histórico, as imagens aparecem em pré-visualização e os outros ficheiros com nome e tamanho; um clique abre imagens e PDF diretamente no Openany, o resto transferes. Pela via Openany só vão textos.',
        },
        matrix: {
          title: 'Mensagens por Matrix',
          p1: 'As pessoas sem conta no Openany alcanças por Matrix. Para isso ligas nas definições, em «Conta Matrix», uma conta existente do matrix.org ou de outro homeserver – tens de a criar lá. A palavra-passe só é usada para iniciar sessão e não é guardada. O Openany inicia sessão como dispositivo próprio e a partir daí pode ler todas as mensagens novas dessa conta, também as que escreves noutro programa Matrix; para fora, tudo continua cifrado.',
          p2: 'Com uma conta ligada, ao escrever escolhes a via: Openany ou Matrix, e nesse caso o identificador Matrix como destinatário. As respostas chegam ao mesmo histórico e vêm marcadas com «Matrix». Se apagares uma mensagem Matrix, desaparece só no Openany; na sala continua lá. Com «Desligar» terminas a sessão do dispositivo; as mensagens já recebidas mantêm-se.',
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
          p1: 'Os eventos dos teus projetos – por exemplo novos inquéritos ou partilhas – chegam-te como mensagem direta automática, desde que o remetente opte por avisar os membros. Aqui encontras também a resposta a uma mensagem enviada pelo formulário de contacto.',
        },
      },
    },
    contacts: {
      title: 'Contactos',
      subs: {
        adressbuch: {
          title: 'Contactos e as suas vias',
          p1: 'Os contactos abrem-se no topo da página de mensagens; o botão ao lado cria diretamente um novo contacto. Um contacto regista como chegar a alguém: nome, imagem, telefone e e-mail, e ainda quantas vias quiseres – Matrix, Meshtastic, nome no Openany, morada, empresa, aniversário ou outro, cada uma com etiqueta própria como pessoal ou trabalho. O campo de pesquisa encontra contactos pelo nome.',
          p2: 'As vias são clicáveis: um número de telefone liga, um endereço de e-mail abre o programa de correio, um nome no Openany ou um identificador Matrix abre o campo de escrita com o destinatário já preenchido. Um contacto pode estar associado a uma conta Openany – é só uma referência e não dá acesso a projetos nem a conteúdos. Os contactos eliminados vão para a reciclagem.',
        },
        adressbuchDatei: {
          title: 'Guardar e trazer contactos',
          p1: 'Nas definições, em «Contactos», transferes todos os contactos como ficheiro vCard ou importas um ficheiro vCard de outro programa. Importar só acrescenta: os contactos existentes são reconhecidos pelo identificador ou pelo nome e mantêm o que têm. Todos os dados viajam em campos padrão; no máximo perdem-se etiquetas próprias, porque muitos livros de endereços só conhecem «pessoal» e «trabalho».',
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
          p1: 'Os programas no teu computador não iniciam sessão com a palavra-passe da tua conta, mas com uma palavra-passe de aplicação própria. Ao criá-la dás-lhe um nome e escolhes a finalidade: «Programas» para WebDAV, Zettlr ou um gestor de ficheiros que leem e escrevem as tuas notas, ou «Sincronização» para programas que espelham a tua conta e escrevem de volta. Só é possível criar se tiveres iniciado sessão com segundo fator.',
          p2: 'A nova palavra-passe é mostrada exatamente uma vez – copia-a de imediato. No programa introduzes o teu nome no Openany como nome de utilizador; o endereço WebDAV está na indicação por baixo da palavra-passe. A lista mostra quando cada palavra-passe foi usada pela última vez; se revogares uma, o programa perde o acesso de imediato.',
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
          p1: 'Perguntas, problemas ou sugestões? Pela ligação «Contacto» no rodapé chegas ao formulário de contacto. A tua mensagem vai como mensagem direta para os administradores, e a resposta encontras mais tarde em Mensagens.',
        },
        bedingungen: {
          title: 'Termos de utilização',
          p1: 'Os termos de utilização da beta e a informação legal estão sempre disponíveis nas ligações do rodapé. Em resumo: durante a beta, o Openany é gratuito e sem publicidade, e os teus dados estão em servidores na Alemanha. Só sai o que tu próprio envias – por exemplo mensagens por Matrix ou pedidos à IA de um projeto.',
        },
      },
    },
  },
};
