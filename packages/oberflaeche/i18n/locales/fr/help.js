// Attention vue-i18n : pas de @, | ou { bruts dans les textes.
export default {
  title: 'Aide',
  intro: 'Tu trouveras ici des descriptions et des guides pour tous les modules et fonctions d’Openany. Le sommaire à gauche te mène directement à la bonne section.',
  version: 'Version {version}',
  toc: 'Sommaire',
  sections: {
    start: {
      title: 'Premiers pas',
      subs: {
        konto: {
          title: 'Compte et connexion',
          p1: 'Les comptes Openany sont créés exclusivement sur invitation personnelle – il n’y a pas d’inscription ouverte. L’adresse e-mail est facultative : Openany fonctionne aussi sans, mais avec un e-mail tu peux réinitialiser toi-même un mot de passe oublié.',
          p2: 'La connexion se fait avec nom d’utilisateur et mot de passe via anyid, le service de connexion commun. La même connexion vaut pour Openany, anyitem et anytail ; le mot de passe et le deuxième facteur sont donc gérés là-bas et non dans Openany (voir « Sécurité » dans les paramètres).',
        },
        ohneKonto: {
          title: 'L’application sans compte',
          p1: 'L’application est un programme complet qui fonctionne aussi sans compte ni serveur : notes, agenda, stockage, carnet d’adresses et messages restent alors uniquement sur cet appareil. Ce qui n’est pas configuré ici ou ne fonctionne pas, l’application le masque au lieu de l’afficher vide – la voie Openany des messages, par exemple, n’apparaît qu’une fois l’appareil connecté à openany.de.',
          p2: 'Si tu veux synchroniser tes données avec Openany en ligne, touche « Se connecter à openany.de » dans les paramètres, sous « Synchronisation » ; tu t’associes à tes autres appareils sous « Appareils à proximité ». Ensuite, un bouton « Synchroniser » apparaît dans l’en-tête et dans le menu ; il tourne pendant la synchronisation.',
        },
        startseite: {
          title: 'L’accueil',
          p1: 'L’accueil affiche des tuiles compactes des modules que tu as choisis : les prochains rendez-vous, les notes modifiées récemment, les derniers fichiers avec la barre de stockage, tes projets avec compteur de non-lus, les messages les plus récents de la boîte de réception et le carnet d’adresses. Chaque tuile mène à son module ; tu choisis lesquelles apparaissent dans les paramètres, sous « Modules sur l’accueil ».',
          p2: 'Si tu n’as choisi aucun module pour l’accueil, la page de bienvenue apparaît à la place : elle présente toutes les fonctions et te permet de composer directement les tuiles. La sélection ne masque rien – tous les espaces restent accessibles via la barre et le menu.',
        },
        navigation: {
          title: 'Menu et structure',
          p1: 'Sur ordinateur, un en-tête se trouve en haut : à gauche, le logo rond mène à l’accueil, à côté se trouvent Notes, Agenda, Stockage et Projets. À droite suivent le bouton clair/sombre, l’enveloppe des messages avec compteur de non-lus et le champ avec ton nom – il ouvre Paramètres, Aide, Corbeille et Déconnexion.',
          p2: 'Sur mobile, tout passe en bas : une barre fixe affiche les modules sous forme d’icônes, et tout à gauche se trouve le bouton de menu avec le compteur de messages non lus. Il ouvre un panneau avec Messages, Paramètres, Aide, Corbeille, le bouton clair/sombre et la déconnexion. Le bouton alterne entre clair, sombre et automatique – automatique suit le réglage de ton appareil.',
        },
      },
    },
    notes: {
      title: 'Notes',
      subs: {
        editor: {
          title: 'Écrire dans l’éditeur',
          p1: 'L’éditeur de notes fonctionne comme un traitement de texte moderne : titres, listes, citations, blocs de code, gras et italique se choisissent dans la barre d’outils. Tu peux aussi taper des raccourcis Markdown – par exemple un dièse suivi d’un espace pour un titre ou un tiret pour une liste – qui se transforment en mise en forme pendant la frappe.',
          p2: 'L’enregistrement est automatique : peu après que tu as cessé de taper, Openany sauvegarde la note et affiche l’état dans la barre d’outils. Sous chaque note figurent aussi la date de création et celle de la dernière modification. Sur mobile, la barre d’outils apparaît sur une seule ligne juste au-dessus du clavier dès que tu touches le texte, et défile latéralement – tous les outils sont à portée de main sans gêner la lecture.',
        },
        mappen: {
          title: 'Classeurs et structure',
          p1: 'Les notes sont rangées dans des classeurs imbricables à volonté. Tu peux renommer, déplacer et réorganiser notes et classeurs – ainsi se construit peu à peu ta propre base de connaissances. La loupe en haut ouvre un champ de recherche qui parcourt toutes les notes à la fois, titre et contenu ; ouverte dans un classeur, « uniquement dans ce classeur » la limite à celui-ci. Ce que tu peux faire avec le classeur ouvert se trouve sous « Classeur », à droite de la barre de chemin.',
          p2: 'Un classeur entier avec ses sous-classeurs se télécharge en archive ZIP ; les notes y sont des fichiers Markdown et restent lisibles partout.',
        },
        wikilinks: {
          title: 'Wikiliens et rétroliens',
          p1: 'Avec deux crochets, tu relies des notes entre elles : tape les crochets et le titre de la note cible, et le lien se crée automatiquement – si la note n’existe pas encore, le lien devient actif dès qu’elle existe. Si le texte doit afficher autre chose que le titre, tu indiques un texte affiché personnalisé après une barre verticale ; le lien pointe toujours vers la bonne note.',
          p2: 'Chaque note affiche ses rétroliens : la liste de toutes les notes qui pointent vers elle. Tu retrouves ainsi les liens entre les idées sans devoir tenir de registre.',
        },
        tags: {
          title: 'Tags',
          p1: 'Les mots-clés s’attribuent directement dans le texte avec un dièse devant le mot ; pendant la frappe, une liste propose les tags déjà utilisés. Avec une barre oblique, tu imbriques les mots-clés – par exemple Projet/Sous-thème ; un filtre sur le thème principal trouve alors aussi tous les sous-thèmes. Dans la vue graphe, les mots-clés peuvent s’afficher comme nœuds distincts, et un clic sur un mot-clé y filtre la liste des notes.',
        },
        einfuegen: {
          title: 'Insérer des images et des fichiers',
          p1: 'Le bouton trombone de la barre d’outils permet d’insérer des contenus dans une note – fraîchement téléversés ou en référence à des contenus déjà stockés dans Openany. Les images apparaissent en aperçu compact directement dans la note (la pleine résolution est dans la Galerie), les autres fichiers sous forme de lien cliquable à télécharger.',
          p2: 'Les contenus insérés sont rangés automatiquement : les images dans un album de la Galerie, les fichiers texte et Office dans un classeur de Documents, tout le reste dans un dossier de fichiers – chaque fois dans un espace dédié aux notes. Tant que la référence figure dans au moins une note, le fichier ne peut pas être mis à la corbeille depuis le Stockage ou la Galerie ; si tu retires la référence du texte, il redevient supprimable.',
        },
        zitate: {
          title: 'Citations et bibliographie',
          p1: 'Pour le travail universitaire, tu peux associer à un classeur un fichier bibliographique au format CSL-JSON – par exemple un export Zotero préalablement téléversé dans le Stockage. Dans le classeur ouvert, « Classeur » → « Lier une bibliothèque » établit la connexion ; les sous-classeurs héritent automatiquement de la bibliothèque.',
          p2: 'Dans les notes de ce classeur, l’éditeur propose des sources adaptées lors d’une citation et insère une référence courte dans le texte. Les références naissent ainsi en écrivant, sans retaper les sources à chaque fois.',
        },
        graph: {
          title: 'Vue graphe',
          p1: 'La vue graphe dessine tes notes comme un réseau : chaque note est un point, chaque wikilien une connexion. Les points sont colorés selon le classeur de niveau supérieur, si bien que les ensembles liés se reconnaissent à la couleur ; les points creux représentent des notes sans aucun lien, et plus une note a de liens, plus son point est gros.',
          p2: 'Chaque classeur de premier niveau est un niveau. S’il y en a plusieurs, tu en choisis un ou plusieurs dans la légende sous le graphe – « Tous les niveaux » les prend tous, « Aucun » vide la sélection. Au lieu d’un niveau, un tag peut aussi servir d’entrée : la barre en dessous liste les tags avec leur nombre, et un tag choisi montre toutes les notes qui le portent, avec le tag en losange gris. Dès que des niveaux sont choisis, la barre ne liste plus que les tags qui y figurent. Tant que tu ne choisis rien, le graphe reste vide, afin que les grandes collections ne soient pas construites d’un coup ; « Afficher tous les tags » ajoute d’un coup tous les tags de la barre.',
          p3: 'Sur ordinateur, tu zoomes avec les boutons plus/moins en haut à droite ou la molette, et tu déplaces la vue en maintenant le bouton de la souris ; sur mobile, tu zoomes à deux doigts et déplaces avec un seul. Un autre bouton réinitialise la vue. Un clic sur un point (sans glisser) ouvre la note. Un clic sur un tag filtre la liste des notes en conséquence.',
        },
        export: {
          title: 'Export PDF',
          p1: 'Chaque note peut être exportée en PDF – pour l’imprimer, l’archiver ou la transmettre. La mise en forme de l’éditeur est conservée. Le PDF va dans le dossier « Notizen » sous Stockage → Documents (les versions précédentes sont conservées) ; « Ouvrir » dans le message l’affiche aussitôt.',
        },
        druckfassung: {
          title: 'Version imprimable',
          p1: 'Un dossier devient un document formel à rendre ou à imprimer : dans le dossier ouvert, choisis « Version imprimable » sous « Classeur », saisis titre, sous-titre, nom, établissement, date et informations complémentaires pour la page de titre, et décide d’ajouter ou non une table des matières. Chaque note du dossier devient un chapitre, par ordre alphabétique du titre ; des préfixes comme « 01 Introduction » fixent l’ordre, et le document n’affiche alors que « Introduction ». Les sous-classeurs sont inclus si « Inclure les sous-classeurs » est coché : un sous-classeur devient un titre et ses notes passent en dessous (« 2 Bases », « 2.1 Notions ») ; une note portant le nom de son sous-classeur apparaît comme texte juste sous ce titre.',
          p2: 'La composition utilise une police de type Times avec des titres sans empattement, des marges de 2,5 cm, un interligne de 1,5, un texte justifié avec césure ; la numérotation commence après la page de titre. Les informations restent enregistrées avec le dossier. Le PDF est créé sur ton appareil – la première fois prend quelques secondes – et va dans le dossier « Druckfassungen » sous Stockage → Documents ; chaque version est conservée.',
          p3: "Citations : dans le dossier ouvert, ouvre « Classeur » → « Sources » et saisis livres, articles, sites web, décisions de justice ou lois ; chacun reçoit un nom court. Dans le texte, écris [{'@'}nomcourt] ou [{'@'}nomcourt, p. 12], et ^[Texte] pour une note de bas de page. Dans le PDF – note ou version imprimable – cela devient des citations Harvard comme « (Müller, 2020, p. 12) » avec la bibliographie à la fin. La liste est un fichier CSL-JSON dans Fichiers, utilisable aussi avec Zotero et Zettlr. La barre d’outils de l’éditeur a un bouton pour les notes de bas de page (chiffre en exposant) : il insère les crochets, et le texte sélectionné devient la note.",
        },
        papierkorb: {
          title: 'Corbeille',
          p1: 'Les notes supprimées vont dans la corbeille (dans le menu sous ton nom, sur mobile dans le bouton de menu) et peuvent y être restaurées ou supprimées définitivement. Lorsque tu supprimes un classeur, tu décides si les notes qu’il contient sont supprimées aussi ou déplacées dans le classeur parent.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes notes, dossiers, indications de la version imprimée et listes de sources ne sont stockés sur openany.de que chiffrés : seuls tes appareils autorisés peuvent les lire. Le serveur ne voit pas non plus les noms de tes dossiers. La recherche, les rétroliens, les tags et le graphe sont calculés sur ton appareil.',
        },
        ordner: {
          title: 'Notes en fichiers (Zettlr)',
          p1: 'Dans l’app sur ton ordinateur, tu peux relier un de tes classeurs à un dossier : ouvre le classeur, choisis « Relier à un dossier … » dans le menu « … » et sélectionne le dossier. Chaque note y est un fichier « Id Titre.md », les sous-classeurs des sous-dossiers – comme Zettlr ou Obsidian l’attendent. Ce que tu y modifies, renommes, déplaces ou crées est repris par l’app tant qu’elle tourne ; si les deux côtés ont changé, une copie de conflit est créée. Si tu supprimes un fichier, la note va à la corbeille. Le dossier est stocké sans chiffrement sur ton ordinateur ; « Délier le dossier » laisse les fichiers en place. Les images des notes se trouvent à côté de la note, sous le nom utilisé dans le texte ; une nouvelle image placée là et citée par une note est reprise dans l’app (galerie, album « Notizen: … »).',
        },
      },
    },
    files: {
      title: 'Stockage',
      subs: {
        aufbau: {
          title: 'Galerie, fichiers, documents',
          p1: 'Le stockage se divise en trois onglets en haut de la page : la Galerie pour les photos et vidéos, Fichiers pour tout le reste et Documents pour les papiers rangés en classeurs. Quand tu changes d’onglet, l’autre espace reste tel que tu l’as laissé – dossiers ouverts et listes chargées ne sont pas perdus.',
          p2: 'L’onglet qui s’ouvre en premier quand tu ouvres le stockage se règle dans les paramètres, sous « Démarrage du stockage » ; tant que tu n’as rien choisi, ce sont les Documents.',
        },
        dokumente: {
          title: 'Archivage de documents',
          p1: 'Dans l’onglet « Documents », tu ranges fichiers PDF, Office et texte dans des classeurs. Le téléversement se fait par bouton ou en glissant simplement les fichiers dans le classeur ouvert. Sur téléphone et tablette, un bouton appareil photo se trouve à côté : il photographie un papier et le range aussitôt comme document dans le classeur.',
          p2: 'Les classeurs s’imbriquent à volonté. Tu peux renommer et déplacer classeurs et documents ; les classeurs entiers peuvent en plus être partagés dans des projets et téléchargés en ZIP.',
        },
        texterkennung: {
          title: 'Documents interrogeables',
          p1: 'Si tu photographies un document papier et le ranges dans un classeur – avec le bouton appareil photo ou comme photo de ta collection d’images –, il devient automatiquement un PDF doté d’une couche de texte invisible. Le document ressemble à ta photo, mais on peut y rechercher, surligner, copier et le faire lire à voix haute. La reconnaissance s’exécute sur ton propre appareil – l’image n’est envoyée nulle part.',
          p2: 'Si tu ranges plusieurs photos à la fois, on te demande s’il faut en faire un document de plusieurs pages – comme les pages d’une lettre – ou des documents séparés. La photo elle-même n’est pas conservée en plus ; pour garder l’image, range-la dans la Galerie. La première conversion est plus longue, car la reconnaissance de texte est chargée une seule fois puis conservée.',
          p3: 'Si un PDF scanné se trouve déjà dans un classeur, tu peux lui appliquer la reconnaissance après coup : « Reconnaître le texte » lit le document page par page. Si le papier est bien blanc, les pages restent inchangées et seul le texte est ajouté. Si elles sont grises ou éclairées de façon inégale – comme une feuille photographiée –, elles sont en plus éclaircies ; le message final t’indique ce qui s’est passé. Si le PDF était déjà interrogeable, il n’est pas modifié.',
        },
        dateien: {
          title: 'Gérer les fichiers',
          p1: 'Dans l’onglet « Fichiers », tu crées des dossiers et téléverses des fichiers de tout type – un par un ou plusieurs à la fois. Dossiers et fichiers peuvent être renommés et déplacés, les fichiers téléchargés à nouveau ; les dossiers peuvent aussi être partagés dans des projets et téléchargés en ZIP. Ce que tu peux faire avec un dossier ou un classeur – télécharger, partager, renommer, déplacer, supprimer – se trouve dans le dossier ouvert, à droite de la barre de chemin, sous « Dossier » (dans un classeur : « Classeur »).',
          p2: 'Chaque compte dispose d’un quota de stockage ; l’occupation actuelle est visible sur la tuile de l’accueil et dans les paramètres, sous « Espace de stockage ».',
        },
        suche: {
          title: 'Rechercher',
          p1: 'La loupe dans Fichiers et Documents ouvre un champ de recherche qui parcourt tous les dossiers ou classeurs à la fois, pas seulement celui qui est ouvert. Un clic sur un résultat ouvre le fichier ; à côté figure son emplacement, et un clic dessus t’emmène dans le dossier.',
          p2: 'La recherche ne trouve pas seulement par nom, mais aussi dans le texte : dans les PDF, les textes Word et OpenDocument ainsi que les fichiers texte, CSV et Markdown jusqu’à 50 Mo. Openany lit ce texte petit à petit en arrière-plan tant que le stockage est ouvert ; tant que tout n’est pas lu, la recherche t’indique combien de fichiers ne se trouvent pour l’instant que par leur nom. Pour un résultat dans le texte, la liste montre le passage avec le mot cherché mis en évidence. Les PDF scannés sans couche de texte n’apparaissent qu’après leur avoir appliqué « Reconnaître le texte ».',
        },
        pdf: {
          title: 'Afficher et modifier des PDF',
          p1: 'Un clic sur un PDF l’ouvre directement dans Openany, sans téléchargement. Tu feuillettes les pages, agrandis ou réduis, ajustes l’affichage à la largeur et recherches dans le document avec la loupe – les résultats sont mis en évidence sur la page. Si un PDF est protégé par mot de passe, Openany te le demande d’abord.',
          p2: 'Tu remplis directement les formulaires du PDF. « Modifier » fait en plus apparaître une barre d’outils : « Surligner » met du texte en couleur, le « Stylo » permet de dessiner librement sur la page et « Texte » pose tes propres lignes sur la feuille – couleur, épaisseur du trait et taille de police sont à ton choix, et chaque étape peut être annulée. Les champs de calcul des formulaires ne calculent toutefois pas, car Openany n’exécute pas de scripts provenant de fichiers tiers.',
          p3: 'Tu peux ajouter des commentaires aux surlignages et aux traits : choisis « Commentaires » et touche l’endroit ; la même vue liste tous les commentaires du document. Tout est enregistré sous forme d’annotations PDF ordinaires, que d’autres programmes affichent aussi. Au premier enregistrement, Openany place la version précédente dans la corbeille, pour que tu puisses revenir à l’original ; si tu quittes le PDF avec des modifications non enregistrées, il te le demande d’abord.',
        },
        galerie: {
          title: 'Galerie photo et albums',
          p1: 'L’onglet « Galerie » contient tes photos et vidéos – dans des albums avec vignettes ou, sans album, en dessous sous « Images ». Les albums s’imbriquent – par exemple un album par voyage avec un sous-album par jour – et se déplacent à tout moment. La visionneuse fait défiler toutes les images d’un album. Pour télécharger, partager, déplacer ou supprimer un album, ouvre-le et utilise « Album » à droite de la barre de chemin.',
          p2: 'La prise de vue la plus récente est en haut, regroupée par mois avec un titre pour chaque mois. C’est la date de prise de vue des données de la photo qui compte ; si elle manque – captures d’écran ou images retouchées, par exemple –, c’est le jour du téléversement.',
          p3: 'Les albums entiers se téléchargent en ZIP, les images individuelles en fichier original. Les albums peuvent aussi être partagés dans des projets.',
        },
        videos: {
          title: 'Vidéos et appareil photo',
          p1: 'En plus des photos, la Galerie accueille aussi les vidéos. Elles apparaissent comme vignette avec une image fixe et se lisent dans la visionneuse. Certains téléphones filment au format HEVC, que tous les navigateurs ne savent pas lire ; dans ce cas, télécharge la vidéo et ouvre-la avec un autre programme.',
          p2: 'Sur téléphone et tablette, la Galerie et chaque album ont deux boutons appareil photo : l’un pour une photo, l’autre pour une vidéo. L’enregistrement arrive directement là où tu te trouves. Sur ordinateur, ces boutons n’existent pas ; tu téléverses comme d’habitude.',
        },
        karte: {
          title: 'Date et lieu de prise de vue',
          p1: 'Pour chaque image, la visionneuse affiche la date de prise de vue et – si la photo le contient – le lieu ; « Voir sur la carte » ouvre l’endroit sur OpenStreetMap. Si tu téléverses des photos depuis un téléphone Android, choisis-les via l’application Fichiers plutôt que via la galerie : sinon Android supprime la position avant que l’image n’arrive dans Openany.',
        },
        papierkorbSpeicher: {
          title: 'Corbeille',
          p1: 'Les contenus supprimés – documents, fichiers, photos, notes, contacts, mais aussi projets, agendas et rendez-vous – passent d’abord dans la corbeille commune et peuvent y être restaurés. Au bout de 30 jours, la corbeille se vide automatiquement ; ce n’est qu’alors que l’espace est définitivement libéré.',
        },
        dateienVerschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes fichiers, classeurs, photos et albums ne sont stockés sur openany.de que chiffrés : noms, dossiers, albums, dates de prise de vue et lieu figurent dans un en-tête chiffré, et les contenus eux-mêmes sont chiffrés avec une clé propre à chaque fichier. Seuls tes appareils autorisés peuvent les lire. Lors de la première synchronisation avec cette version, l’app récupère tous les originaux, les chiffre et les téléverse à nouveau – cela prend un moment et se fait de préférence en Wi-Fi. Ce qui se trouve alors dans la corbeille est supprimé définitivement, et les partages d’albums et de dossiers dans les projets prennent fin.',
        },
      },
    },
    calendar: {
      title: 'Agenda',
      subs: {
        verwalten: {
          title: 'Plusieurs agendas',
          p1: 'Tu peux tenir autant d’agendas que tu veux côte à côte – perso, famille, association –, chacun avec sa couleur. Tu affiches ou masques chaque agenda d’un clic. En haut, tu passes de la vue mois à la vue semaine ; la semaine montre les rendez-vous selon l’heure, et les rendez-vous passés apparaissent discrètement grisés.',
        },
        termine: {
          title: 'Créer des rendez-vous',
          p1: 'Tu crées un rendez-vous en cliquant sur un jour : avec titre, description, heure, ou comme événement sur une journée entière ou sur plusieurs jours. Les répétitions sont aussi possibles – quotidiennes, hebdomadaires, mensuelles ou annuelles. Tu modifies ou supprimes les rendez-vous existants directement depuis la vue. Les heures s’entendent dans le fuseau horaire défini dans ton profil.',
        },
        abos: {
          title: 'Abonnements d’agenda',
          p1: 'Les agendas externes – par exemple les jours fériés ou le calendrier des matchs du club – s’intègrent via une URL d’abonnement au format ICS. Les agendas abonnés se mettent à jour automatiquement. Leurs rendez-vous sont en lecture seule dans Openany : on les modifie là d’où vient l’agenda, car chaque mise à jour remplace de nouveau le contenu par l’original.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Tes agendas et rendez-vous ne sont stockés sur openany.de que sous forme chiffrée : seuls tes appareils autorisés peuvent les lire. L’adresse d’un abonnement est chiffrée elle aussi ; chaque appareil la récupère lui-même, openany.de ne récupère rien.',
        },
        importExport: {
          title: 'Importer et exporter',
          p1: 'Avec « Importer », tu charges un fichier ICS dans l’un de tes agendas. Si tu importes à nouveau le même fichier, les rendez-vous sont mis à jour au lieu d’être dupliqués. L’icône de partage à côté d’un agenda l’enregistre en fichier ICS – à emporter dans n’importe quelle autre application d’agenda.',
        },
      },
    },
    projects: {
      title: 'Projets',
      subs: {
        grundlagen: {
          title: 'Projets et membres',
          p1: 'Un projet est un espace de travail commun : tu invites d’autres utilisateurs comme membres, qui acceptent ou refusent l’invitation. Le propriétaire du projet gère les membres et peut aussi transmettre le projet à un autre membre.',
          p2: 'Chaque projet a quatre onglets : Chat, Planification, Membres et Partages. Les chiffres sur les onglets indiquent les messages de chat non lus ou le nombre d’éléments qu’ils contiennent.',
        },
        chat: {
          title: 'Chat du projet',
          p1: 'Chaque projet dispose d’un chat commun en temps réel : les messages apparaissent immédiatement chez tous les membres, sans recharger la page. Le chat est chiffré de bout en bout : seuls les membres peuvent le lire sur leurs appareils, pas openany.de. Si quelqu’un part, la clé change d’elle-même – il ne lit plus rien de nouveau.',
          p2: 'Chaque membre peut épingler des messages importants – les messages épinglés se retrouvent vite dans la barre d’épingles au-dessus de l’historique et peuvent y être détachés.',
          p3: 'Avec deux crochets, tu renvoies en pleine phrase à des contenus du projet : aux notes ainsi qu’aux dossiers, fichiers, albums et images partagés. Pendant la saisie, une liste de suggestions montre ce qui convient et insère le lien terminé. Le bouton avec le symbole de chaîne à côté du bouton d’envoi fait la même chose – sur téléphone, souvent plus pratique que de chercher deux crochets.',
          p4: 'Un clic sur un lien mène directement à la cible : dans la note, dans le dossier avec le fichier mis en évidence ou vers l’image dans la visionneuse. Tu ne peux renvoyer qu’à ce qui est déjà visible dans le projet – ce que personne n’a partagé n’apparaît ni dans la liste de suggestions ni comme lien cliquable.',
        },
        planung: {
          title: 'L’onglet Planification',
          p1: 'L’onglet « Planification » rassemble les votes et les listes du projet. Avec le bouton « Nouveau », tu choisis le type qui convient : recherche de date, sondage, planning des tours, liste de ce que chacun apporte ou liste d’invités. Chaque membre peut créer et voter ; seul le créateur ou le propriétaire du projet peut supprimer.',
          p2: 'Les votes sont chiffrés de bout en bout – y compris les voix. Les appareils des membres comptent eux-mêmes et appliquent les règles : quand les places sont limitées, c’est le premier inscrit qui compte. Seule la personne qui a créé le vote peut le clôturer ; pour une recherche de date, ton appareil peut ajouter la date choisie à ton agenda.',
        },
        terminfindung: {
          title: 'Trouver une date',
          p1: 'Pour trouver une date, tu proposes plusieurs options et chacun vote oui, non ou peut-être – vous fixez ainsi une rencontre sans longs échanges.',
        },
        umfragen: {
          title: 'Sondages',
          p1: 'Un sondage pose une question avec des réponses fixes – oui/non ou personnalisées, anonyme si souhaité. Avec des options à cocher, il sert aussi de liste de tâches commune ; les votes peuvent être dupliqués et clôturés.',
        },
        listen: {
          title: 'Planning de créneaux, liste « qui apporte quoi », liste d’invités',
          p1: 'Trois types de listes allègent l’organisation : dans le planning de créneaux, tu crées des créneaux avec des places et les membres s’inscrivent. La liste « qui apporte quoi » clarifie qui apporte quoi – les entrées peuvent être prises en charge et complétées. La liste d’invités garde un œil sur les invités (invités, acceptés, refusés), y compris ceux sans compte Openany.',
        },
        geteilteKalender: {
          title: 'Partager des agendas',
          p1: 'Sous « Planification », tu partages l’un de tes agendas avec le projet : les membres y voient les prochains rendez-vous et tous dans leur propre agenda, sous « Aus Projekten » – en lecture seule. Toi seul peux les modifier ; les changements arrivent à la prochaine synchronisation. Tout est chiffré avec la clé du projet. La corbeille à côté de l’agenda arrête le partage ; ton agenda reste.',
        },
        freigaben: {
          title: 'Partager des contenus',
          p1: 'Dans l’onglet « Partages », « Partager » partage avec le projet un dossier de notes, un album ou un dossier (ou classeur) : choisis d’abord le type, puis l’élément. Tout reste chiffré de bout en bout – avec une clé propre à chaque partage, qui change quand tu arrêtes de partager ou que quelqu’un quitte le projet. Les contenus restent chez toi ; quand le partage prend fin, tu gardes tout.',
          p2: 'Les albums et dossiers sont partagés en consultation seulement : les membres les ouvrent dans le projet, voient sous-albums et sous-dossiers, ouvrent images et PDF et, avec « Reprendre », en mettent une copie dans leur propre stockage sous « Des projets ». Toi seul peux les modifier.',
          p3: 'Les dossiers de notes se partagent avec le niveau « consulter » ou « modifier ». Les membres les ouvrent dans l’éditeur de notes habituel et voient aussi les images qu’ils contiennent ; avec « modifier », ils changent les notes et en créent. Pour l’instant, seul le propriétaire du dossier peut insérer des images.',
        },
        lokal: {
          title: 'Projets uniquement sur place',
          p1: 'Dans l’application, « + Projet » crée un projet uniquement sur cet appareil – sans compte et sans serveur ; il porte la marque « Sur place seulement ». Tu invites d’autres personnes sous « Inviter un membre à proximité » : Openany doit être ouvert sur l’autre appareil, sur le même Wi-Fi ou point d’accès, et les deux appareils affichent les mêmes six chiffres, que vous comparez et confirmez. Le projet est géré par les appareils du propriétaire ; s’il n’y en a qu’un, l’application avertit – associe d’abord un second appareil à toi.',
          p2: 'Les membres partagent leurs propres classeurs de notes, dossiers et albums dans le projet sous « Partager quelque chose », d’abord en lecture seule. Les classeurs de notes peuvent aussi être partagés en modification : une seule personne modifie alors une note à la fois, et l’enregistrement se fait sur l’appareil de la personne à qui appartient le classeur – s’il n’est pas à proximité, la note reste en lecture seule. Le chat atteint aussitôt toutes les personnes à proximité, les autres à la prochaine synchronisation. La planification – votes et listes – voyage de la même manière.',
          p3: 'Tu synchronises le reste avec « Synchroniser » dès qu’un membre est à proximité. Le propriétaire peut retirer des membres ; qui veut partir le fait avec « Quitter », ce qui ne marche que si un autre membre est à proximité pour en être informé. Ce que les partants avaient partagé disparaît chez les autres ; ce qu’ils avaient déjà reste sur leurs appareils.',
        },
      },
    },
    messages: {
      title: 'Messages',
      subs: {
        direkt: {
          title: 'Messages directs',
          p1: 'Tu ouvres tes messages depuis le menu. Avec « Message » en haut à droite, tu écris directement à n\'importe quel autre utilisateur – son nom Openany suffit. Les liens dans les messages sont cliquables.',
          p2: 'Sous chaque message reçu, « Répondre » ouvre l\'éditeur avec l\'expéditeur déjà renseigné, par le même canal que le message. Les longs messages sont repliés ; « Lire la suite » les affiche en entier. Le compteur de non-lus sur le bouton de menu indique si quelque chose de nouveau t\'attend.',
          p3: 'Les messages directs sont chiffrés de bout en bout : openany.de ne stocke que des données illisibles, seuls tes appareils peuvent les lire. Tous tes appareils partagent une clé. Un nouvel appareil affiche 6 chiffres ; tu les saisis sur un appareil déjà configuré, et il est connecté et reçoit la clé. La clé se trouve aussi dans ton fichier de sauvegarde. Si tu perds un appareil, retire-le dans les réglages sous « Mes appareils » ; la clé change alors. Tu peux écrire à toute personne qui a ouvert l\'application au moins une fois.',
        },
        hinweise: {
          title: 'Avis',
          p1: 'Dans l\'application web, Messages n\'affiche que des avis : notifications de tes projets, l\'avis qu\'une personne t\'a ajouté à un projet, réponses au formulaire de contact et avis sur tes appareils. La recherche au-dessus de la liste trouve les avis par texte et par expéditeur.',
          p2: 'Les messages directs – y compris par Matrix, e-mail et à proximité – ne sont disponibles que dans l\'application. Ils sont chiffrés de bout en bout, et les clés se trouvent sur tes appareils, pas sur openany.de. C\'est pourquoi l\'application web ne peut ni les afficher ni les écrire.',
        },
        suche: {
          title: 'Rechercher et filtrer',
          p1: 'Au-dessus du fil se trouve un champ de recherche. Il trouve les messages par leur texte, par le nom de l’interlocuteur et par identifiant Matrix, et met les résultats en évidence. En dessous, des boutons restreignent la liste : aux messages non lus, à ceux avec pièce jointe et – si tu utilises plus d’une voie – à certaines voies comme Openany ou Matrix. Ces boutons se combinent.',
          p2: 'À côté du champ de recherche, tu passes d’une vue à l’autre : « Fil » montre tous les messages dans l’ordre chronologique, « Par contact » les regroupe en une ligne par interlocuteur, avec le dernier message et le nombre de non-lus. Un clic sur une ligne affiche le fil avec cette personne précise ; « Fermer la conversation » te ramène à tous.',
        },
        anhaenge: {
          title: 'Pièces jointes',
          p1: 'Par Matrix, tu envoies aussi des fichiers : le trombone du champ de saisie joint un fichier jusqu’à 10 Mo, et sur téléphone et tablette le bouton appareil photo à côté prend directement une photo. Dans le fil, les images apparaissent en aperçu, les autres fichiers avec nom et taille ; un clic ouvre images et PDF directement dans Openany, le reste se télécharge. Par la voie Openany passent des fichiers jusqu’à 15 Mo, chiffrés de bout en bout comme le texte. Ils comptent dans ton stockage et dans celui du destinataire jusqu’à ce que chacun supprime le message.',
        },
        matrixApp: {
          title: 'Messages via Matrix',
          p1: 'Dans l’application, ton appareil est lui-même l’appareil Matrix. Tu te connectes avec un compte existant dans les paramètres, sous « Comptes Matrix », et tes messages sont chiffrés de bout en bout entre cet appareil et ton interlocuteur – openany.de ne lit pas. La vérification de l’appareil par comparaison d’emojis n’est pas encore possible dans l’application ; les autres programmes Matrix l’affichent donc comme appareil non vérifié, mais le chiffrement est bien là.',
          p2: 'Tu peux connecter plusieurs comptes Matrix. Leurs messages partagent un même fil ; les nouveaux partent du compte par défaut, les réponses du compte sur lequel le message est arrivé, et « Définir par défaut » fait d’un autre compte le compte par défaut. Si tu supprimes un message Matrix, il disparaît seulement sur cet appareil ; il reste dans le salon.',
        },
        email: {
          title: 'E-mail via ta boîte',
          p1: 'Dans l’application, l’e-mail s’ajoute comme voie supplémentaire – pas un logiciel de messagerie à part, mais ta boîte existante, par exemple chez Posteo, mailbox.org ou GMX. Dans les paramètres, sous « Boîtes mail », tu saisis adresse et mot de passe ; le plus souvent un mot de passe d’application créé au préalable chez ton fournisseur. L’application trouve les serveurs elle-même ; si elle n’en trouve pas, tu les saisis sous « Serveurs à la main ». Le mot de passe reste sous clé sur cet appareil, et l’appareil parle lui-même au serveur de messagerie – openany.de ne voit aucun e-mail.',
          p2: 'Les e-mails apparaissent avec leur objet dans le fil commun des Messages ; « Relever maintenant » récupère aussitôt les nouveaux. Si tu connectes plusieurs boîtes, les nouveaux e-mails partent de la boîte par défaut et les réponses de celle qui a reçu l’e-mail ; en écrivant, tu choisis sous « De ». À la suppression, tu décides si un e-mail disparaît seulement ici ou passe aussi dans le dossier corbeille du serveur. Si ton fournisseur a classé quelque chose en spam, un avis apparaît au-dessus du fil ; là, « Pas un spam » renvoie l’e-mail dans la boîte de réception.',
          p3: 'Par e-mail, les pièces jointes peuvent aller jusqu’à 15 Mo. Les pièces jointes reçues vont dans tes fichiers avec « Enregistrer » ou dans le dossier de téléchargements avec « Sur l’appareil ». Si une pièce jointe est arrivée chiffrée, l’application demande d’abord : dans Fichiers, elle est stockée en clair et synchronisée avec openany.de.',
        },
        pgp: {
          title: 'E-mail chiffré avec OpenPGP',
          p1: 'Tu peux chiffrer tes e-mails de bout en bout avec OpenPGP. Pour cela, chaque boîte a besoin de sa propre clé : dans les paramètres, à la boîte, tu la crées avec « Créer une clé » – ou, si tu utilises déjà la boîte avec PGP, par exemple dans Thunderbird, tu importes la clé existante avec « Importer une clé », afin que les deux programmes lisent les mêmes e-mails. La clé secrète se trouve uniquement sur cet appareil. « Enregistrer une copie de sauvegarde » la dépose, verrouillée par une phrase secrète, dans le dossier de téléchargements ; « Enregistrer la clé publique » te donne le fichier à envoyer aux autres.',
          p2: 'Les clés publiques de tes contacts, l’application les rassemble d’elle-même sous « Clés des contacts », à partir des e-mails qui les joignent. S’il en manque une, tu la cherches par adresse – si tu le souhaites aussi sur keys.openpgp.org, qui apprend alors qui tu cherches – ou tu l’importes depuis un fichier. Compare de préférence une fois l’empreinte avec ton interlocuteur ; si une clé change, l’application te le signale.',
          p3: 'En écrivant, tu actives « Envoyer chiffré » ; en dessous, tu vois si la clé du destinataire est connue. Si tu réponds à un e-mail chiffré, l’option est déjà activée. Dans le fil, les e-mails portent la marque « chiffré » et, s’ils sont signés, « signé » – ou un avertissement si la signature ne correspond pas.',
        },
        vorOrt: {
          title: 'Messages à proximité',
          p1: 'Avec la voie « À proximité », tu écris directement aux appareils proches, sans aucun serveur. Openany doit être ouvert sur l’autre appareil, et les deux doivent être sur le même Wi-Fi. Vous pouvez vous écrire dès que vous vous connaissez : tes propres appareils, les membres d’un projet commun – ou les personnes que vous avez confirmées sur place. Pour cela, tu choisis « Confirmer sur place », les deux appareils affichent les mêmes 6 chiffres, et vous touchez tous deux « Correspond ». Si l’autre appareil n’est pas là, le message attend et part à la prochaine rencontre.',
          p2: 'Au début, rien n’arrive des inconnus. Si tu actives « Autoriser les demandes des appareils à proximité » dans les paramètres, sous « Appareils à proximité », ils peuvent t’envoyer jusqu’à 3 messages de 500 caractères chacun ; ceux-ci apparaissent sous « Demandes » au-dessus du fil, pas dans le fil lui-même. Tu ne peux répondre qu’après la confirmation avec les 6 chiffres. Qui ne doit plus rien t’envoyer, tu l’arrêtes avec « Bloquer ». Si l’un de tes messages a été refusé, il porte « non distribué », et la raison s’affiche quand tu le survoles.',
        },
        systemnachrichten: {
          title: 'Notifications et réponses',
          p1: 'Les événements de tes projets – par exemple de nouveaux sondages ou partages – te parviennent sous forme d\'avis si l\'expéditeur choisit d\'informer les membres ; de même l\'avis qu\'une personne t\'a ajouté à un projet. Les avis ne sont pas chiffrés et apparaissent dans l\'application et dans l\'application web. Tu trouves aussi ici la réponse à un message envoyé via le formulaire de contact.',
        },
      },
    },
    contacts: {
      title: 'Carnet d’adresses',
      subs: {
        adressbuch: {
          title: 'Contacts et leurs moyens',
          p1: 'Le carnet d’adresses s’ouvre en haut de la page Messages ; le bouton à côté crée directement un nouveau contact. Un contact indique comment joindre quelqu’un : nom, image, téléphone et e-mail ainsi que d’autres moyens à volonté – Matrix, Meshtastic, nom Openany, adresse postale, entreprise, anniversaire ou autre, chacun avec son libellé comme perso ou travail. Le champ de recherche trouve les contacts par nom.',
          p2: 'Les moyens sont cliquables : un numéro de téléphone appelle, une adresse e-mail ouvre le programme de messagerie, un nom Openany ou un identifiant Matrix ouvre le champ d’écriture avec le destinataire déjà rempli. Les contacts supprimés vont dans la corbeille.',
        },
        verschluesselt: {
          title: 'Chiffré de bout en bout',
          p1: 'Ton carnet d’adresses est chiffré de bout en bout, avec la même clé que tes messages directs. Seuls tes appareils approuvés peuvent le lire ; openany.de le conserve et le synchronise, mais ne voit ni noms, ni numéros, ni images. Un nouvel appareil n’affiche le carnet qu’une fois approuvé sur un autre appareil. C’est pourquoi l’application web n’a plus de carnet d’adresses.',
        },
        vcard: {
          title: 'Enregistrer et importer des vCard',
          p1: 'Dans Paramètres → Carnet d’adresses, tu enregistres tous les contacts dans un fichier vCard ou tu en importes un d’un autre programme. L’import ne fait qu’ajouter : les contacts existants sont reconnus par leur identifiant ou leur nom et gardent ce qu’ils ont. Le fichier lui-même n’est pas chiffré – conserve-le en conséquence.',
        },
        sicherheitsnummer: {
          title: 'Numéro de sécurité',
          p1: 'Le numéro de sécurité te permet de vérifier que personne ne lit entre vous – pas même openany.de. Ouvre-le dans le contact, sous « Sécurité », ou dans les messages après avoir ouvert une conversation depuis « Par contact ». Comparez les douze groupes de chiffres sur vos deux appareils ; s’ils correspondent, touche « Ça correspond ». Les messages de cette personne portent alors une coche à côté du cadenas. Si sa clé change plus tard, par exemple après la perte d’un appareil, la coche disparaît et un avis vous invite à comparer de nouveau.',
        },
      },
    },
    settings: {
      title: 'Paramètres et compte',
      subs: {
        profil: {
          title: 'Profil, fuseau horaire et espace de stockage',
          p1: 'Ton nom Openany est fixe et ne peut pas être modifié. L’adresse e-mail est facultative et ne sert qu’à des fins liées au compte, comme la réinitialisation du mot de passe, jamais à la publicité ; tu ne peux la modifier qu’avec ton mot de passe. Le fuseau horaire détermine comment s’entendent les heures de tes rendez-vous – un bouton reprend le fuseau de l’appareil. En dessous, « Espace de stockage » montre combien tu occupes.',
        },
        module: {
          title: 'Modules sur l’accueil',
          p1: 'Ici, tu choisis quels modules apparaissent en tuile sur l’accueil – Notes, Agenda, Stockage, Projets, Messages et Carnet d’adresses. La sélection ne masque rien ; tous les modules restent accessibles. Sans sélection, l’accueil affiche la page de bienvenue.',
        },
        sicherheit: {
          title: 'Sécurité : mot de passe et deuxième facteur',
          p1: 'Le mot de passe, le deuxième facteur et tes appareils sont chez anyid, pas chez Openany. La carte « Compte et sécurité » des paramètres t’y emmène directement par trois liens : « Changer le mot de passe », « Deuxième facteur » et « Tes appareils », où tu vois les appareils couplés et peux les retirer. Les modifications valent à la fois pour Openany, anyitem et anytail.',
          p2: 'Le deuxième facteur est un code issu d’une application d’authentification, demandé après le mot de passe ; sans lui, aucun mot de passe d’application ne peut être créé. Lors de la configuration, tu reçois des codes de récupération ; garde-les – sans eux et sans l’application, tu ne pourras plus entrer. Si tu as enregistré une adresse e-mail, tu peux réinitialiser toi-même un mot de passe oublié.',
        },
        spracheDesign: {
          title: 'Apparence et langue',
          p1: 'Sous « Apparence », tu choisis entre deux designs : « Net », bleu pétrole avec des angles plus marqués et une surface calme, ou « Lilas », avec des angles arrondis et un fond à motif. Les deux existent en clair et en sombre – ce que tu règles indépendamment avec le bouton de l’en-tête ou du menu mobile. Sous « Langue », tu choisis la langue dans laquelle Openany te parle : allemand, anglais, espagnol, français, portugais, polonais ou ukrainien. Tant que tu n’as pas choisi, il suit la langue de ton appareil ou de ton navigateur ; si elle n’est pas disponible, Openany parle anglais.',
        },
        zugriff: {
          title: 'Accès externe : mots de passe d’application',
          p1: 'Les programmes de ton ordinateur ne se connectent pas avec le mot de passe de ton compte, mais avec un mot de passe d’application qui leur est propre. En le créant, tu lui donnes un nom et choisis l’usage « Synchronisation » – pour les programmes qui reflètent ton compte et y réécrivent. Les notes ne sont plus stockées sur le serveur que chiffrées ; il n’y a donc plus d’accès WebDAV. La création n’est possible que si tu t’es connecté·e avec un second facteur.',
          p2: 'Le nouveau mot de passe s’affiche une seule fois – copie-le tout de suite. Dans le programme, saisis ton nom Openany comme nom d’utilisateur. La liste indique quand chaque mot de passe a été utilisé pour la dernière fois ; si tu en révoques un, le programme perd immédiatement l’accès.',
        },
        abgleich: {
          title: 'Synchroniser avec Openany',
          p1: 'Sous « Synchronisation », tu connectes l’application à ton compte avec « Se connecter à openany.de ». L’application affiche un code que tu saisis et confirmes sur la page d’anyid dans le navigateur. Cette confirmation dans le navigateur n’est pas un détour, mais la preuve que c’est bien toi qui ajoutes l’appareil. La connexion est facultative : sans elle, tout reste sur cet appareil, et « Déconnecter » l’annule à tout moment sans perte de données ici.',
          p2: 'Ensuite, « Synchroniser » synchronise dans les deux sens et indique ce qui a été récupéré, envoyé ou ignoré. « Tout récupérer à nouveau » recharge tout le contenu au lieu du seul nouveau ; rien n’est supprimé.',
          p3: 'Avec « Actualiser en arrière-plan », l’application se synchronise d’elle-même environ toutes les heures – seulement avec une connexion et pas avec une batterie faible. En données mobiles, seule la liste arrive ; fichiers et images seulement en Wi-Fi. Les appareils à proximité n’en font pas partie ; pour eux, l’application doit être ouverte. Sur ordinateur, Openany se synchronise toutes les heures tant qu’il est ouvert.',
        },
        geraeteNah: {
          title: 'Appareils à proximité',
          p1: 'Tes propres appareils se synchronisent aussi directement entre eux, sans serveur – tant qu’ils sont sur le même Wi-Fi ou point d’accès et qu’Openany est ouvert sur les deux. Sous « Appareils à proximité », tu les trouves avec « Rechercher » et les relies avec « Associer » : les deux appareils affichent le même code, et sur chacun tu confirmes qu’il correspond. Dès lors, ils se synchronisent entre eux à chaque « Synchroniser » ; « Oublier » annule l’association.',
          p2: 'Sous « Ton nom », tu choisis comment tu apparais aux autres sur place. C’est sous ce nom qu’on t’invite dans des projets – toi en tant que personne, pas un appareil précis ; il vaut pour tous tes appareils associés.',
        },
        speicherGeraet: {
          title: 'Stockage sur cet appareil',
          p1: 'Sous « Stockage sur cet appareil », tu décides quelle part de tes fichiers et images l’application garde sous la main. « À la demande » affiche tous les fichiers dans la liste, mais n’en récupère le contenu qu’à l’ouverture. Avec « Dossiers et albums choisis », ce que tu as marqué dans le stockage avec « Garder sur cet appareil » reste toujours sur l’appareil, sous-dossiers compris. « Tout garder » récupère après chaque synchronisation ce qui manque, tant qu’il y a de la place.',
        },
        sofort: {
          title: 'Notifier immédiatement',
          p1: 'Avec « Notifier immédiatement », l’application maintient des connexions économes vers Openany et vers tes boîtes mail, pour que les nouveaux messages et e-mails arrivent aussitôt – sans Google. Les boîtes mail n’ont pas besoin d’Openany pour cela. Android affiche pour cela une notification permanente, que tu peux masquer dans les paramètres du système. Sur ordinateur, Openany veille tant qu’il est ouvert et prévient par une notification du système.',
        },
        sicherung: {
          title: 'Sauvegarde dans un fichier',
          p1: 'Sous « Sauvegarde », « Créer une sauvegarde » produit un fichier chiffré avec tout ce qu’Openany a sur cet appareil : notes, calendrier, contacts, fichiers, galerie, projets, messages et tes boîtes mail avec mot de passe et ta propre clé OpenPGP. L’application propose une phrase secrète de six mots ; note-la et garde-la séparée du fichier. Sans elle, le fichier ne peut pas être ouvert – pas même par nous.',
          p2: '« Restaurer une sauvegarde » ouvre un tel fichier sur un autre appareil. Elle remplace tout ce qui s’y trouvait ; ensuite la connexion à openany.de est supprimée et tu associes de nouveau les appareils à proximité. Ce qui identifie un appareil n’est pas inclus.',
        },
        tresor: {
          title: 'Identifiants et clés dans le coffre',
          p1: 'Ce avec quoi cet appareil s’identifie auprès d’Openany et d’autres appareils est conservé sous clé sur l’appareil ; la clé reste dans le trousseau du système : le Keystore sous Android et, sur ordinateur, le trousseau de macOS, le Gestionnaire d’identification de Windows ou le Secret Service sous Linux. On y trouve aussi les mots de passe de tes boîtes mail et tes clés OpenPGP secrètes. « Déconnecter » supprime les identifiants seulement ici – ils sont révoqués définitivement dans la liste d’appareils d’anyid et dans les mots de passe d’application d’Openany.',
        },
      },
    },
    contact: {
      title: 'Aide et contact',
      subs: {
        kontakt: {
          title: 'Contacter les exploitants',
          p1: 'Des questions, des problèmes ou des idées ? Le lien « Contact » en bas de page mène au formulaire de contact. Ton message parvient aux admins sous forme d\'avis, et tu trouveras la réponse plus tard sous Messages.',
        },
        bedingungen: {
          title: 'Conditions d’utilisation',
          p1: 'Les conditions d’utilisation de la bêta et les mentions légales sont accessibles à tout moment via les liens du pied de page. En bref : pendant la bêta, Openany est gratuit et sans publicité, et tes données sont hébergées sur des serveurs en Allemagne. Seul sort ce que tu envoies toi-même – par exemple des messages via Matrix ou par e-mail.',
        },
      },
    },
  },
};
