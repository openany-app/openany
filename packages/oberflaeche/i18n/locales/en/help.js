// Help page (/hilfe): manual covering all modules and features.
// The outline (sections, subsections, paragraph counts) lives in
// shared/helpOutline.js and is checked against both languages – this file
// only holds the texts.
// vue-i18n caveat: no raw @, | or { in the texts.
export default {
  title: 'Help',
  intro: 'Here you will find descriptions and instructions for every module and feature of Openany. Use the table of contents on the left to jump straight to the right section.',
  version: 'Version {version}',
  toc: 'Contents',
  sections: {
    start: {
      title: 'Getting started',
      subs: {
        konto: {
          title: 'Account and sign-in',
          p1: 'Openany accounts are created by personal invitation only – there is no open registration. An email address is optional: Openany works entirely without one, but with an email you can reset a forgotten password yourself.',
          p2: 'You sign in with your username and password through anyid, the shared sign-in service. The same sign-in covers Openany, anyitem and anytail; that is why your password and second factor are managed there rather than in Openany itself (see "Security" under the settings).',
        },
        ohneKonto: {
          title: 'The app without an account',
          p1: 'The app is a complete program that also works without any account or server: notes, calendar, storage, address book and messages then live only on this device. Whatever is not set up here or does not work, the app hides instead of showing it empty – the Openany channel for messages, for instance, only appears once the device is connected to openany.de.',
          p2: 'If you want to sync your data with Openany on the web, tap "Connect to openany.de" in the settings under "Sync"; you pair with your other devices under "Nearby devices". After that, a "Sync" button appears in the header and the menu, and it spins while syncing.',
        },
        startseite: {
          title: 'The home page',
          p1: 'The home page shows compact tiles of the modules you selected for it: your upcoming events, recently edited notes, the latest files with a storage bar, your projects with unread counters, the newest messages in your inbox and the address book. Each tile leads into its module; which ones appear is configured in the settings under "Modules on the homepage".',
          p2: 'If you have not selected any modules for the home page, the welcome overview appears instead: it introduces all features and lets you assemble your tiles right away. The selection hides nothing – every area remains reachable through the bar and the menu.',
        },
        navigation: {
          title: 'Menu and layout',
          p1: 'On a computer there is a header at the top: on the left the round logo takes you to the home page, next to it are Notes, Calendar, Storage and Projects. On the right follow the light/dark toggle, the envelope for messages with its unread counter, and the pill with your name – it opens Settings, Help, Trash and Log out.',
          p2: 'On the phone everything moves to the bottom: a fixed bar shows the modules as icons, with the menu button and its unread-message counter at the far left. It opens a panel with Messages, Settings, Help, Trash, the light/dark toggle and Log out. The toggle switches between light, dark and automatic – automatic follows your device\'s setting.',
        },
      },
    },
    notes: {
      title: 'Notes',
      subs: {
        editor: {
          title: 'Writing in the editor',
          p1: 'The note editor works like a modern word processor: headings, lists, quotes, code blocks, bold and italics are available from the toolbar. Alternatively type markdown shortcuts – for example a hash followed by a space for a heading or a dash for a list – and they turn into formatting as you write.',
          p2: 'Saving is automatic: shortly after you stop typing, Openany saves the note and shows the save status in the toolbar. The created and last-modified dates are also shown under each note. On the phone the toolbar appears as a single scrollable row right above the keyboard as soon as you type in the text – so every tool is within reach without getting in the way while reading.',
        },
        mappen: {
          title: 'Folders and structure',
          p1: 'Notes live in folders that can be nested as deeply as you like. Notes and folders can be renamed, moved and rearranged – over time this grows into your own knowledge base. The search field above the list searches all notes at once, title and content, regardless of the folder currently open.',
          p2: 'You can download an entire folder including its subfolders as a ZIP archive; the notes inside are markdown files and stay readable everywhere.',
        },
        wikilinks: {
          title: 'Wikilinks and backlinks',
          p1: 'Two square brackets link notes to each other: type the brackets and the title of the target note and the link is created automatically – if the note does not exist yet, the link becomes active as soon as it does. If you want different wording in the text than the title, add your own display text after a vertical bar; the link still points to the right note.',
          p2: 'Every note shows its backlinks: a list of all notes linking to it. This way you rediscover connections without keeping track yourself.',
        },
        tags: {
          title: 'Tags',
          p1: 'You assign tags right in the text with a hash in front of the word; a suggestion list of existing tags helps as you type. A slash lets you nest tags – for example Project/Subtopic; filtering on the parent then also finds all subtopics. In the graph view the tags can be shown as their own nodes, and clicking a tag there filters the note list to it.',
        },
        einfuegen: {
          title: 'Inserting images and files',
          p1: 'The paperclip button in the toolbar inserts content into a note – either freshly uploaded or as a reference to content already stored in Openany. Images appear as a space-saving preview right inside the note (the full resolution lives in the gallery), other files as a clickable download link.',
          p2: 'Inserted content is filed automatically: images into a gallery album, text and office files into a document dossier, everything else into a file folder – each in a dedicated area for notes. As long as the reference sits in at least one note, the file cannot be moved to the trash from Storage or Gallery; remove the reference from the text and it becomes freely deletable again.',
        },
        zitate: {
          title: 'Citations and references',
          p1: 'For academic work you can assign a reference file in CSL-JSON format to a folder – for example an export from Zotero that you uploaded to Storage beforehand. The "Link library" button on the folder makes the connection; subfolders inherit the library automatically.',
          p2: 'In notes within that folder the editor suggests matching sources as you cite and inserts a short reference into the text. This way citations appear while writing, without retyping the source details every time.',
        },
        graph: {
          title: 'Graph view',
          p1: 'The graph view draws your notes as a network: every note is a dot, every wikilink a connection. The dots are coloured by top-level folder, so related areas are recognisable by colour alone; hollow dots are notes without any connection, and the more links a note has, the bigger its dot.',
          p2: 'Each top-level folder is a level. If there are several, you pick one or more in the legend below the graph – "All levels" takes them all, "None" clears the selection. Instead of a level, a tag can be the way in: the bar below lists the tags with their count, and a selected tag shows all notes carrying it, with the tag as a grey diamond. Once levels are selected, the bar only lists the tags that occur in them. Until you choose something the graph stays empty, so large collections are not built all at once; "Show all tags" brings in all tags of the bar at once.',
          p3: 'On a computer you zoom with the plus/minus buttons in the top right or the mouse wheel and pan by holding the mouse button; on the phone you zoom with two fingers and pan with one. Another button resets the view. Clicking a dot (without dragging) opens the note. Clicking a tag filters the note list by it.',
        },
        uebersicht: {
          title: 'Automatic overview',
          p1: 'Alongside the graph there is an automatic overview: it lists all note titles from A to Z and below shows a collapsible tree of your tags (nested tags appear as branches). Clicking a title opens the note, clicking a tag filters the list – all without you having to maintain a table of contents yourself.',
        },
        export: {
          title: 'PDF export',
          p1: 'Every note can be downloaded as a PDF – for printing, archiving or passing on. The formatting from the editor is preserved.',
        },
        papierkorb: {
          title: 'Trash',
          p1: 'Deleted notes go to the trash (in the menu under your name, on the phone behind the menu button) and can be restored from there or deleted permanently. When deleting a folder you decide whether its notes are deleted along with it or moved to the parent folder.',
        },
      },
    },
    files: {
      title: 'Storage',
      subs: {
        aufbau: {
          title: 'Gallery, files, documents',
          p1: 'Storage is split into three tabs at the top of the page: the gallery for photos and videos, Files for anything at all, and Documents for paperwork in dossiers. When you switch tabs, the other area stays as you left it – open folders and loaded lists are not lost.',
          p2: 'Which tab opens first when you open storage is set in the settings under "Storage start"; as long as you have not chosen, it is Documents.',
        },
        dokumente: {
          title: 'Document filing',
          p1: 'In the "Documents" tab you organise PDFs, office files and text files into dossiers. Upload them with the button, or simply drag them into the open dossier. On phones and tablets there is a camera button next to it: it photographs a piece of paper and files it straight into the dossier as a document.',
          p2: 'Dossiers can be nested as deeply as you like. Both dossiers and individual documents can be renamed and moved; whole dossiers can also be shared into projects and downloaded as a ZIP.',
        },
        texterkennung: {
          title: 'Searchable documents',
          p1: 'Photograph a piece of paper and file it in a dossier – with the camera button or as a photo from your picture collection – and it becomes a PDF with an invisible text layer automatically. The document looks like your photo, but can be searched, selected, copied and read aloud. Recognition runs on your own device – the image is not sent anywhere for it.',
          p2: 'If you file several photos at once, you are asked whether they should become one multi-page document – the pages of a letter, say – or separate ones. The photo itself is not kept as well; if you want the image, put it in the gallery. The first run takes longer because text recognition is downloaded once and then kept.',
          p3: 'If a scanned PDF is already sitting in a dossier, you can apply recognition to it afterwards: "Recognise text" reads the document page by page. If the paper is cleanly white the pages stay unchanged and only the text is added. If they are grey or unevenly lit – as with a photographed sheet – they are brightened as well; the message at the end tells you which happened. If the PDF was already searchable, it is left alone.',
        },
        dateien: {
          title: 'Managing files',
          p1: 'In the "Files" tab you create folders and upload files of any kind – individually or several at once. Both folders and files can be renamed and moved, files downloaded again; folders can also be shared into projects and downloaded as a ZIP.',
          p2: 'Every account has a storage quota; the current usage is shown on the home page tile and in the settings under "Storage".',
        },
        suche: {
          title: 'Searching',
          p1: 'The magnifying glass in Files and Documents opens a search field that searches all folders or dossiers at once, not just the one currently open. Clicking a result opens the file; next to it you see where it lives, and clicking that takes you to the folder.',
          p2: 'Search finds files not only by name but also by their text: in PDFs, Word and OpenDocument texts, and text, CSV and Markdown files up to 50 MB. Openany reads this text bit by bit in the background while storage is open; until everything has been read, the search tells you how many files can for now only be found by name. For a match in the text, the list shows the passage with the search word highlighted. Scanned PDFs without a text layer only turn up once you have applied "Recognise text" to them.',
        },
        pdf: {
          title: 'Viewing and editing PDFs',
          p1: 'Clicking a PDF opens it right inside Openany, without downloading. You page through it, zoom in or out, fit it to the width and search the document with the magnifying glass – matches are highlighted on the page. If a PDF is password-protected, Openany asks for the password first.',
          p2: 'You fill in PDF forms directly. "Edit" also brings up a toolbar: "Highlight" marks text in colour, the "Pen" lets you draw freely on the page, and "Text" places your own lines on the sheet – you choose colour, stroke width and font size, and every step can be undone. Calculated fields in forms do not calculate, though, because Openany does not run scripts from other people\'s files.',
          p3: 'You can attach comments to highlights and strokes: choose "Comments" and tap the spot; the same view lists all comments in the document. Everything is saved as ordinary PDF annotations that other programs show too. On the first save Openany moves the previous version to the trash, so you can go back to the original; if you leave the PDF with unsaved changes, it asks first.',
        },
        galerie: {
          title: 'Photo gallery and albums',
          p1: 'The "Gallery" tab holds your photos and videos – in albums with preview tiles or, without an album, below them under "Photos". Albums can be nested – for instance one album per trip with a sub-album per day – and moved at any time. The lightbox flips through all pictures of an album.',
          p2: 'The newest shot comes first, grouped by month with a heading for each month. What counts is the capture date from the photo data; if there is none – screenshots or edited pictures, say – the upload day counts.',
          p3: 'Whole albums can be downloaded as ZIP, single pictures as the original file. Albums can be shared into projects as well.',
        },
        videos: {
          title: 'Videos and camera',
          p1: 'Besides photos, the gallery also takes videos. They appear as a tile with a still frame and play in the lightbox. Some phones record in HEVC, which not every browser can play; in that case download the video and open it with another program.',
          p2: 'On phones and tablets the gallery and every album have two camera buttons: one for a photo, one for a video. The recording lands directly wherever is currently open. On a computer the buttons are not there; you upload as usual.',
        },
        karte: {
          title: 'Capture date and location',
          p1: 'For every picture the lightbox shows the capture date and – if the photo contains it – the capture location; "Show on map" opens the spot on OpenStreetMap. When uploading photos from an Android phone, pick them via the Files app rather than the gallery: otherwise Android strips the location before the picture reaches Openany.',
        },
        papierkorbSpeicher: {
          title: 'Trash',
          p1: 'Deleted content – documents, files, photos, notes, contacts, but also projects, boards, calendars and events – first moves to the shared trash and can be restored there. After 30 days the trash empties automatically; only then is the storage space finally freed.',
        },
      },
    },
    calendar: {
      title: 'Calendar',
      subs: {
        verwalten: {
          title: 'Multiple calendars',
          p1: 'You can keep any number of calendars side by side – for example private, family, club – each with its own colour. Individual calendars can be shown and hidden with a click. At the top you switch between month and week view; the week shows events by time of day, and past events are subtly greyed out.',
        },
        termine: {
          title: 'Creating events',
          p1: 'You create events by clicking a day: with title, description, time, or as an all-day or multi-day event. Recurring events are supported too – daily, weekly, monthly or yearly. Existing events are edited or deleted right from the view. Times apply in the time zone set in your profile.',
        },
        tagesansicht: {
          title: 'Day view',
          p1: 'Clicking the number of a day opens the day view – a run-through rather than a grid, meant for the question on the evening before: what does the child need tomorrow? At the very top the care plan says who the child is with ("with you" or with whom else). Below come the lessons from the timetable in order, each with what is due for it; tests and other deadlines of the coming days already appear in advance with "in … days". The arrows take you to the previous or next day.',
          p2: 'At every lesson you can quickly add one with "Homework"; it lands as a card with the subject on the project\'s board. "Notebook" opens the folder for the subject. Whatever is due but belongs to no lesson is listed under "No lesson". One click on an entry takes you to its place in the project.',
        },
        ausProjekten: {
          title: 'From projects',
          p1: 'Under "From projects" in the calendar list you show what your projects contribute to the calendar: timetable, care and due dates – cards and milestones with a deadline. Each source can be shown and hidden on its own, and new plans appear by themselves. The entries are read-only; a click shows the details, and "Open in project" takes you to where you change them.',
        },
        abos: {
          title: 'Calendar subscriptions',
          p1: 'External calendars – such as public holidays or your club\'s fixtures – are added via a subscription URL in ICS format. Subscribed calendars refresh automatically. Their events are read-only in Openany: changes are made where the calendar comes from, because every refresh replaces the content with the original again.',
          p2: 'The other way round you can share your own calendars via a subscription link: the feed icon in the calendar list creates a secret URL that Google, Apple or Outlook can subscribe to – changes arrive there automatically. Treat the link like a password; "Renew link" invalidates the old one immediately.',
        },
        importExport: {
          title: 'Import and export',
          p1: 'Existing calendars are imported as ICS files, and you can export your calendars the same way – full data portability in both directions, with no lock-in to Openany.',
        },
      },
    },
    projects: {
      title: 'Projects',
      subs: {
        grundlagen: {
          title: 'Projects and members',
          p1: 'A project is a shared workspace: you invite other users as members, and the invitees accept or decline the invitation. The project owner manages the members and can also hand the project over to another member.',
          p2: 'Every project has four tabs: Chat, Planning, Members and Shared. The numbers on the tabs show unread chat messages or how much each tab contains.',
        },
        chat: {
          title: 'Project chat',
          p1: 'Every project has a shared real-time chat: messages appear instantly for all members without reloading the page.',
          p2: 'Any member can pin important messages – pinned messages are easy to find for everyone in the pin bar above the history, where they can also be unpinned again.',
          p3: 'Two square brackets let you point at project content in mid-sentence: at notes, at boards, roadmaps and place collections including the individual cards, milestones and places inside them, and at shared folders, files, albums and images. A suggestion list shows what is available as you type and inserts the finished link. The chain-symbol button next to the send button does the same – usually handier on a phone than hunting for two brackets.',
          p4: 'Clicking a link takes you straight to its target: into the note, onto the board with the card opened, into the folder with the file highlighted, or to the image in the lightbox. You can only point at what is visible in the project anyway – anything nobody shared appears neither in the suggestion list nor as a clickable link.',
        },
        planung: {
          title: 'The planning tab',
          p1: 'The "Planning" tab collects the project\'s planning tools. Via the "New" button you pick the right type: scheduling, survey, shift plan, bring list, guest list, board, roadmap, places, school holidays, timetable or care plan. The "Preset: School" template creates school holidays, a timetable, tests and a homework board in one go. While doing so you pick your federal state, so the holidays are in right away, and optionally your school\'s bell schedule. Any member may create and edit; deleting is reserved for the respective creator or the project owner.',
        },
        terminfindung: {
          title: 'Scheduling',
          p1: 'With scheduling you propose several date options and everyone votes yes, no or maybe – so you agree on a meeting without endless back and forth.',
        },
        umfragen: {
          title: 'Surveys',
          p1: 'A survey asks one question with fixed answers – yes/no or your own, anonymous if desired. With checkable options it also serves as a shared task list; polls can be duplicated and closed.',
        },
        listen: {
          title: 'Shift plan, bring list, guest list',
          p1: 'Three list types take organisational work off your hands: with the shift plan you create shifts with slots and members sign up. The bring list clarifies who brings what – entries can be claimed and added. The guest list keeps track of guests (invited, accepted, declined), including guests without an Openany account.',
        },
        boards: {
          title: 'Kanban boards',
          p1: 'On kanban boards you organise tasks in freely arranged columns and cards – for example "Open", "In progress", "Done". Cards are moved by dragging; changes appear live for all members. A card can also carry a place from the project\'s place collections and a subject – so a homework task belongs to the matching lesson.',
        },
        roadmap: {
          title: 'Roadmap',
          p1: 'The roadmap shows milestones with date and status on a timeline – so the whole team sees what should be done by when. Milestones can be marked as reached, overdue ones are flagged. Like cards, milestones can also carry a place and a subject, a test for instance.',
        },
        orte: {
          title: 'Places',
          p1: 'Under places you collect map points on OpenStreetMap – meeting points, addresses, parking spots. You drop the point by clicking the map, optionally with a name and note; clicking the name of a place tile opens the point directly on OpenStreetMap. Once collected, you simply pick the places elsewhere: on cards, milestones, subjects and entries in the timetable or care plan.',
        },
        schuljahr: {
          title: 'School holidays',
          p1: 'School holidays record a school year: its period and the days off within it. You can import your region\'s holidays as an ICS file and add individual training days and public holidays by hand. The difference matters: during holidays a different care arrangement often applies, while on a single day off it continues in the usual rhythm. The import only adds and replaces nothing you entered yourself.',
        },
        wochenplan: {
          title: 'Timetable and care plan',
          p1: 'Both are the same building block: a weekly grid from Monday to Sunday that repeats week after week. The timetable carries times and subjects, the care plan whole days and a person – "who has the child when". Link school holidays to it and lessons drop out during the holidays by themselves; a care plan can be limited to term time or to the holidays. If your rhythm alternates weekly, switch to A/B weeks: either every two weeks from the Monday of week A, or by even and odd calendar weeks if your arrangement says so in writing – the rhythm then jumps, though, in years with 53 calendar weeks.',
          p2: 'Below the grid are the periods. They override it within their date range and cover everything that is not a rhythm: the summer holidays split in half, a swapped weekend, a school trip. The time zone belongs to the plan, not to the viewer – so the timetable shows the same times for both parents, even if one of them lives abroad.',
        },
        faecher: {
          title: 'Subjects',
          p1: 'You maintain a project\'s subjects via "Subjects" in the timetable: name, abbreviation, teacher, colour and optionally a place. They belong to the project and not to a single plan, so they survive the change of term. A subject decides what a lesson in the timetable shows, and links homework cards and tests to the matching lesson. If you delete a subject, cards, milestones and lessons only lose the reference to it – they themselves stay.',
        },
        hefte: {
          title: 'Exercise books for subjects',
          p1: 'For every subject, "Create folder and share" creates a notes folder – the exercise book – and shares it into the project straight away. The folder belongs to you, not to the project, and counts against your storage; that is why the subject shows whose folder it is. A project owns no content itself: whatever is in it, members have shared.',
        },
        planAbo: {
          title: 'Subscribing to timetable and care',
          p1: '"Subscribe" creates a link with which the timetable or the care plan can be subscribed to in Google, Apple or Thunderbird – without opening Openany. The link covers the type, not a single plan: if you keep holiday care as a second plan, it is in the same subscription. Places are only included if you explicitly ask for it – the link is public and needs no sign-in, and with places your addresses would sit behind it. Calendar programs only fetch the subscription every few hours; "Renew" invalidates the old link at once, "Revoke" switches it off.',
        },
        freigaben: {
          title: 'Sharing content',
          p1: 'The "Shared" tab holds everything members have made available to the project: note folders, albums, and folders and dossiers from storage. You share each item where it lives – in Notes, in the Gallery or in Storage – either with the level "Read only" or "Edit". A share applies to all project members, is inherited by subfolders and sub-albums, and can be removed at any time. The content itself stays with its owner; when a share ends, everything remains theirs.',
          p2: 'Clicking a share opens it inside the project: notes in the familiar workspace with search, backlinks and a graph across all shared notes of the project; albums as a photo grid with lightbox; folders as a file list. With "Edit" members can contribute themselves – write notes, upload photos, drop files and create subfolders; "Read only" allows viewing and downloading.',
        },
        projektNotizen: {
          title: 'Shared project notes',
          p1: 'In a note folder shared with "Edit", members can create, edit and delete notes and create subfolders right inside the project; embedded images and files are visible to all members, and anyone allowed to edit can insert their own. Everything created – including uploaded images and files – belongs to the folder\'s owner and counts against their storage; deleted notes land in their trash, so nothing is lost for good.',
        },
        ki: {
          title: 'AI in a project',
          p1: 'A project can have an AI connected. The project owner sets it up in the "Members" tab below the member list: provider, model and their own API key. The key is stored encrypted and never shown to anyone again, not even the owner; costs are borne by the account the key belongs to. All members see in the same place whether an AI is connected, to which provider, and who set it up.',
          p2: 'You ask from within shared content: below a note via "Ask AI", on documents in shared dossiers and folders (PDF, Word, OpenDocument and text files) and – if the model understands images – on photos in the lightbox of a shared album. You write an instruction such as "Summarise this in three sentences" and get the answer to copy; anyone allowed to edit the note can also insert it at the end directly.',
          p3: 'Important: what you ask leaves Openany. Above the input field it always says what goes to which provider – the whole note text, the document\'s text, or the photo, scaled down and without its capture location. Scanned PDFs without a text layer need text recognition first; very long documents are rejected.',
        },
        benachrichtigungen: {
          title: 'Notifying members',
          p1: 'For new surveys, scheduling polls or shares you can optionally notify the members by direct message – so nobody misses news in the project.',
        },
        lokal: {
          title: 'Nearby-only projects',
          p1: 'In the app, "+ Project" creates a project on this device only – without an account and without a server; it carries the mark "Nearby only". You invite others under "Invite a member nearby": Openany must be open on the other device, on the same Wi-Fi or hotspot, and both devices show the same six digits, which you compare and confirm. The project is managed by the owner\'s devices; if there is only one, the app warns you – pair a second device of your own beforehand.',
          p2: 'Members share their own notes folders, folders and albums into the project under "Share something", read-only at first. Notes folders can also be shared for editing: then only one person edits a note at a time, and it is saved on the device of the person who owns the folder – if that device is not nearby, the note stays read-only. The chat reaches everyone nearby at once, the others at the next sync. The planning – boards, roadmaps, places and school planning – travels along the same way.',
          p3: 'You sync the rest with "Sync" as soon as a member is nearby. The owner can remove members; anyone who wants to leave does so with "Leave", which only works when another member is nearby to learn of it. What departed members had shared disappears for the others; what they already had stays on their devices.',
        },
      },
    },
    messages: {
      title: 'Messages',
      subs: {
        direkt: {
          title: 'Direct messages',
          p1: 'You reach the messages on a computer via the envelope in the header, on the phone via the menu button at the bottom left. "Message" at the top right lets you write directly to any other user – their Openany name is all you need as recipient. New messages appear instantly without reloading the page, and links in them are clickable.',
          p2: 'Under every message you receive, "Reply" opens the compose field with the sender already filled in, on the same channel the message came by. Long messages are collapsed; "Read more" shows them in full. The unread counter always shows whether something new is waiting – on a computer on the envelope, on a phone on the menu button in the bottom bar.',
        },
        suche: {
          title: 'Searching and filtering',
          p1: 'Above the timeline sits a search field. It finds messages by their text, by the other person\'s name and by Matrix IDs, and highlights the matches. Below it, toggles narrow the list down: to unread messages, to those with an attachment and – if you use more than one channel – to single channels such as Openany or Matrix. The toggles can be combined.',
          p2: 'Next to the search field you switch between two views: "Timeline" shows all messages in order of time, "By contact" sums them up in one row per person, with the latest message and the number of unread ones. Clicking a row shows the timeline with exactly that person; "Close conversation" takes you back to everyone.',
        },
        anhaenge: {
          title: 'Attachments',
          p1: 'Over Matrix you can send files too: the paper clip in the compose field attaches a file of up to 10 MB, and on phones and tablets the camera button next to it takes a photo straight away. In the timeline pictures appear as a preview, other files with name and size; a click opens pictures and PDFs right inside Openany, everything else you download. The Openany channel carries text only.',
        },
        matrix: {
          title: 'Messages via Matrix',
          p1: 'People without an Openany account can be reached via Matrix. To do so, connect an existing account at matrix.org or another homeserver in the settings under "Matrix account" – you create the account there. The password is used only to sign in and is not stored. Openany signs in as its own device and can read all new messages of that account from then on, including those you write in another Matrix app; everything stays encrypted towards the outside.',
          p2: 'Once an account is connected, you choose the channel when writing: Openany or Matrix, then with the Matrix ID as recipient. Replies arrive in the same timeline and are marked "Matrix". If you delete a Matrix message, it disappears only in Openany; it stays in the room. "Disconnect" signs the device out again; messages already received are kept.',
        },
        matrixApp: {
          title: 'Messages via Matrix',
          p1: 'In the app your device itself is the Matrix device. You sign in with an existing account in the settings under "Matrix accounts", and your messages are end-to-end encrypted between this device and the other person – openany.de does not read along. The app cannot yet do device verification by emoji comparison; other Matrix programs therefore list it as an unverified device, but it is encrypted all the same.',
          p2: 'You can connect several Matrix accounts. Their messages share one timeline; new ones go from the default account, replies from the account the message arrived at, and "Make default" makes another account the default. If you delete a Matrix message, it disappears only on this device; it stays in the room.',
        },
        email: {
          title: 'Email via your mailbox',
          p1: 'In the app, email joins as a further channel – not a mail program of its own, but your existing mailbox, at Posteo, mailbox.org or GMX, say. In the settings under "Email mailboxes" you enter address and password; usually that is an app password you first create with your provider. The app looks up the servers itself; if it finds none, you enter them under "Servers by hand". The password stays locked away on this device, and the device talks to the mail server itself – openany.de never sees a mail.',
          p2: 'Mails appear with their subject in the shared timeline under Messages; "Fetch now" fetches new ones immediately. If you connect several mailboxes, new mails go from the default mailbox and replies from the one the mail arrived at; when writing you choose under "From". When deleting you decide whether a mail only disappears here or also moves to the trash folder on the mail server. If your provider has filed something as spam, a notice appears above the timeline; there "Not spam" brings a mail back to the inbox.',
          p3: 'Email attachments may be up to 15 MB. Received attachments go into your files with "Save" or into the download folder with "To this device". If an attachment arrived encrypted, the app asks first: in Files it lies unencrypted and is synced with openany.de.',
        },
        pgp: {
          title: 'Encrypted email with OpenPGP',
          p1: 'You can encrypt mails end to end with OpenPGP. Each mailbox needs a key of its own: in the settings at the mailbox you create it with "Create key" – or, if you already use the mailbox with PGP, in Thunderbird for instance, you import the existing one with "Import key" so both programs read the same mails. The secret key lives only on this device. "Save backup copy" puts it, locked with a passphrase, into the download folder; "Save public key" gives you the file you send to others.',
          p2: 'The app collects your contacts\' public keys under "Contacts\' keys" by itself from mails that carry them. If one is missing, you search for it by address – optionally also at keys.openpgp.org, which then learns whom you are asking about – or import it from a file. Best compare the fingerprint once with the other person; if a key changes, the app points it out.',
          p3: 'When writing you switch on "Send encrypted"; below it you see whether the recipient\'s key is known. When you reply to an encrypted mail, the switch is already on. In the timeline mails carry the mark "encrypted" and, if they are signed, "signed" – or a warning if the signature does not match.',
        },
        vorOrt: {
          title: 'Messages nearby',
          p1: 'With the "Nearby" channel you write to devices close by directly, entirely without a server. Openany must be open on the other device, and both must be on the same Wi-Fi. You can write to each other once you know each other: your own devices, members of a shared project – or people you have confirmed nearby. For that you choose "Confirm nearby", both devices show the same 6 digits, and both tap "Matches". If the other device is not there right now, a message waits and goes across at the next meeting.',
          p2: 'At first nothing arrives from strangers. If you switch on "Allow requests from nearby devices" in the settings under "Nearby devices", they can send you up to 3 messages of 500 characters each; these appear under "Requests" above the timeline, not in the timeline itself. You can only reply after confirming with the 6 digits. Anyone who should not send you anything more, you stop with "Block". If one of your own messages was refused, it shows "not delivered", and the reason appears when you point at it.',
        },
        systemnachrichten: {
          title: 'Notifications and replies',
          p1: 'Events from your projects – such as new surveys or shares – reach you as automatic direct messages, provided the sender chose to notify the members. The reply to a message sent via the contact form also shows up here.',
        },
      },
    },
    contacts: {
      title: 'Address book',
      subs: {
        adressbuch: {
          title: 'Contacts and how to reach them',
          p1: 'You open the address book at the top of the messages page; the button next to it creates a new contact right away. A contact records how someone can be reached: name, picture, phone and email plus any number of further routes – Matrix, Meshtastic, openany name, postal address, employer, birthday or other, each with its own label such as home or work. The search field finds contacts by name.',
          p2: 'The routes are clickable: a phone number places a call, an email address opens your mail program, an openany name or a Matrix ID opens the compose form with the recipient already filled in. A contact can be linked to an Openany account – that is only a reference and grants no access to projects or content. Deleted contacts go to the trash.',
        },
        adressbuchDatei: {
          title: 'Backing up and importing the address book',
          p1: 'In the settings under "Address book" you download all contacts as a vCard file or read in a vCard file from another program. Reading a file only adds: existing contacts are recognised by their identifier or their name and keep what they have. Everything travels in standard fields; at most your own labels can get lost, because many address books only know "home" and "work".',
        },
      },
    },
    settings: {
      title: 'Settings and account',
      subs: {
        profil: {
          title: 'Profile, time zone and storage',
          p1: 'Your Openany name is fixed and cannot be changed. The email address is optional and serves only account purposes such as password reset, never advertising; you can change it only with your password. The time zone determines how the times of your events are meant – a button adopts the device\'s zone. Below, "Storage" shows how much space you use.',
        },
        module: {
          title: 'Modules on the homepage',
          p1: 'Here you choose which modules appear as tiles on the home page – Notes, Calendar, Storage, Projects, Messages and Address book. The selection hides nothing; all modules remain reachable. Without a selection, the home page shows the welcome overview.',
        },
        sicherheit: {
          title: 'Security: password and second factor',
          p1: 'Password, second factor and your devices live with anyid, not with Openany. The "Account & security" card in the settings takes you straight there with three links: "Change password", "Second factor" and "Your devices", where you view paired devices and throw them out again. Changes apply to Openany, anyitem and anytail at once.',
          p2: 'The second factor is a code from an authenticator app that is asked for after the password; without it no app password can be created. When setting it up you receive recovery codes; keep them safe – without them and without the app you will not get back in. If you have stored an email address, you can reset a forgotten password yourself.',
        },
        spracheDesign: {
          title: 'Appearance and language',
          p1: 'Under "Appearance" you choose between two designs: "Clear" in teal with squarer corners and a calm surface, or "Lilac" with round corners and a patterned background. Both come in light and dark – you switch that independently with the toggle in the header or in the phone menu. Under "Language" you set which language Openany speaks to you in: German, English, Spanish, French, Portuguese, Polish or Ukrainian. Until you choose, it follows the language of your device or browser; if that one is not available, Openany speaks English.',
        },
        zugriff: {
          title: 'External access: app passwords',
          p1: 'Programs on your computer do not sign in with your account password but with their own app password. When creating one you give it a name and choose its purpose: "Programs" for WebDAV, Zettlr or a file manager that read and write your notes, or "Sync" for programs that mirror your account and write back. Creating one only works if you signed in with a second factor.',
          p2: 'The new password is shown exactly once – copy it right away. In the program you enter your Openany name as username; the WebDAV address is in the hint below the password. The list shows when each password was last used; revoke one and the program loses access immediately.',
        },
        abgleich: {
          title: 'Syncing with Openany',
          p1: 'Under "Sync" you connect the app to your account with "Connect to openany.de". The app shows a code that you type in and confirm on anyid\'s page in the browser. This confirmation in the browser is not a detour but the proof that it really is you adding the device. Connecting is optional: without a connection everything stays on this device, and "Disconnect" undoes it at any time without losing data here.',
          p2: 'After that, "Sync" syncs in both directions and reports afterwards what was fetched, pushed or skipped. "Fetch everything again" downloads the whole stock again instead of just what is new; nothing is deleted in the process.',
          p3: 'With "Refresh in the background" the app syncs by itself roughly every hour – only with a connection and not on a low battery. On mobile data only the list comes, files and pictures only on Wi-Fi. Nearby devices are not included; for them the app has to be open.',
        },
        geraeteNah: {
          title: 'Nearby devices',
          p1: 'Your own devices also sync with each other directly, without any server – as long as both are on the same Wi-Fi or hotspot and Openany is open on both. Under "Nearby devices" you find them with "Search" and connect them with "Pair": both devices show the same code, and on both you confirm that it matches. From then on they sync with each other on every "Sync"; "Forget" undoes the pairing.',
          p2: 'Under "Your name" you set how you appear to others nearby. That is the name under which people invite you into projects – you as a person, not a single device; it applies to all your paired devices.',
        },
        speicherGeraet: {
          title: 'Storage on this device',
          p1: 'Under "Storage on this device" you decide how much of your files and pictures the app keeps at hand. "On demand" shows all files in the list but only fetches the content when you open it. With "Selected folders and albums", whatever you marked in storage with "Keep on this device" always stays on the device, sub-folders included. "Keep everything" fetches what is missing after every sync, as long as there is room.',
        },
        sofort: {
          title: 'Notify immediately',
          p1: 'With "Notify immediately" the app keeps economical connections to Openany and to your mailboxes so new messages and mails arrive at once – entirely without Google. The mailboxes do not need Openany for this. Android shows a permanent notice for it, which you can hide in the system settings.',
        },
        sicherung: {
          title: 'Backup in one file',
          p1: 'Under "Backup", "Create backup" makes an encrypted file with everything Openany has on this device: notes, calendar, contacts, files, gallery, projects, messages and your mailboxes including password and your own OpenPGP key. The app suggests a passphrase of six words; write it down and keep it apart from the file. Without it the file cannot be opened – not even by us.',
          p2: '"Restore backup" opens such a file on another device. It replaces everything that was there; afterwards the connection to openany.de is removed, and you pair nearby devices again. What identifies a device is not included.',
        },
        tresor: {
          title: 'Credentials and keys in the vault',
          p1: 'What this device uses to identify itself to Openany and other devices lies locked on the device; the key to it stays in the Android Keystore. Your mailbox passwords and your secret OpenPGP keys are kept there too. "Disconnect" removes the credentials only here – they are finally revoked in anyid\'s device list and under Openany\'s app passwords.',
        },
      },
    },
    contact: {
      title: 'Help and contact',
      subs: {
        kontakt: {
          title: 'Contacting the operators',
          p1: 'Questions, problems or wishes? The "Contact" link in the footer opens the contact form. Your message goes to the admins as a direct message, and you will find the reply under Messages later.',
        },
        bedingungen: {
          title: 'Terms of use',
          p1: 'The beta terms of use and the legal notice are always available via the links in the footer. In short: during the beta Openany is free and ad-free, and your data is stored on servers in Germany. Only what you send out yourself leaves – such as messages via Matrix or requests to a project\'s AI.',
        },
      },
    },
  },
};
