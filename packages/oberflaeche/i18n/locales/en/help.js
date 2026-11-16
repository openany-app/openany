// Help page (/hilfe): manual covering all modules and features.
// The outline (sections, subsections, paragraph counts) lives in
// shared/helpOutline.js and is checked against both languages – this file
// only holds the texts.
// vue-i18n caveat: no raw @, | or { in the texts.
export default {
  title: 'Help',
  intro: 'Here you will find descriptions and instructions for every module and feature of Openany. Use the table of contents on the left to jump straight to the right section.',
  version: 'Version {version}',
  aktualisieren: 'Please update',
  aktualisierenTitel: 'openany.de needs at least version {version}.',
  toc: 'Contents',
  sections: {
    start: {
      title: 'Getting started',
      subs: {
        ohneKonto: {
          title: 'The app without an account',
          p1: 'The app is a complete program that also works without any account or server: notes, calendar, storage, address book and messages then live only on this device. Whatever is not set up here or does not work, the app hides instead of showing it empty – the Openany channel for messages, for instance, only appears once the device is connected to openany.de.',
          p2: 'If you want to sync your data with openany.de, create an account in the settings under "Sync" or connect to an existing one; you pair with your other devices under "Nearby devices". After that, the header and the menu show a "Sync" button that spins while syncing.',
        },
        kontoAnlegen: {
          title: 'Creating an account',
          p1: 'You create an account on openany.de right in the app: in the settings under "Sync", tap "Create new account" and choose a name. Your device then solves a small computing puzzle that protects against accounts being created in bulk; it usually takes less than a minute. You need neither an email address nor a password.',
          p2: 'Then you accept the terms of use – only then is storage on openany.de available to you, 100 MB to start with. You connect further devices through a device that is already connected, using 6 digits (see “My devices”).',
          p3: 'Because your account has no password, a backup file is the only way back if you lose your only device. The app offers it to you right after creating the account; create it and keep it safe together with its passphrase (see “Backup in one file”).',
        },
        startseite: {
          title: 'The home page',
          p1: 'The home page shows compact tiles of the modules you selected: today’s events, the address book with search, the three most recently edited notes, the three latest files with the storage used, and your hubs. Each tile leads into its module; which ones appear is configured in the settings under "Modules on the homepage".',
          p2: 'At first all tiles are selected. If you deselect all of them, the welcome overview appears instead: it introduces all features and lets you assemble your tiles right away. The selection hides nothing – every area remains reachable through the bar and the menu.',
        },
        navigation: {
          title: 'Menu and layout',
          p1: 'On a computer there is a header at the top: on the left the round logo takes you to the home page, next to it are Notes, Calendar, Storage and Hubs. On the right follow the light/dark toggle, the envelope for messages with its unread counter, and the pill with your name – it opens Settings, Help, Trash and Log out.',
          p2: 'On the phone everything moves to the bottom: a fixed bar shows the modules as icons, with the menu button and its unread-message counter at the far left. It opens a panel with Messages, Settings, Help, Trash, the light/dark toggle and Log out. The toggle switches between light, dark and automatic – automatic follows your device\'s setting. Tapping the module that is already open takes you to its starting point.',
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
          p1: 'Notes live in folders that can be nested as deeply as you like. Notes and folders can be renamed, moved and rearranged – over time this grows into your own knowledge base. “Find” at the top (magnifier) opens a search field that searches all notes at once, title and content; opened inside a folder, “this folder only” limits it to that folder. What you can do with the open folder is under “…” on the right of the path bar.',
          p2: 'You go up through the path or with the back button, which goes one level higher; the cross in the path bar closes an open note. Next to each folder you see when a note in it last changed, including its subfolders. A folder without notes shows no date.',
        },
        wikilinks: {
          title: 'Wikilinks and backlinks',
          p1: 'Two square brackets link notes to each other: type the brackets and the title of the target note and the link is created automatically – if the note does not exist yet, the link becomes active as soon as it does. If you want different wording in the text than the title, add your own display text after a vertical bar; the link still points to the right note.',
          p2: 'Every note shows its backlinks: a list of all notes linking to it. This way you rediscover connections without keeping track yourself.',
        },
        tags: {
          title: 'Tags',
          p1: 'You assign tags right in the text with a hash in front of the word; a suggestion list of existing tags helps as you type. A slash lets you nest tags – for example Hub/Subtopic; filtering on the parent then also finds all subtopics. In the graph view the tags can be shown as their own nodes, and clicking a tag there filters the note list to it.',
        },
        einfuegen: {
          title: 'Inserting images and files',
          p1: 'The paperclip button in the toolbar inserts a file from your device into a note. Images appear as a compact preview right in the note (the full resolution is in the media library), other files as a link to open.',
          p2: 'Inserted content is filed automatically: images into a media library album, text and office files into a document dossier, everything else into a file folder – each in a dedicated area for notes.',
        },
        zitate: {
          title: 'Citations and references',
          p1: 'For academic work every folder keeps a source list: open the folder and choose “Sources” under “…”, then enter books, articles, websites, court decisions or statutes. Each source gets a short name. The list is a CSL-JSON file in Files, so it also works with Zotero and Zettlr.',
          p2: 'In notes of this folder the editor suggests matching sources as you cite and puts the citation into the text. That way citations come about as you write, without typing out the details each time. How they appear in the PDF is described under “Print version”.',
        },
        graph: {
          title: 'Graph view',
          p1: 'The graph view draws your notes as a network: every note is a dot, every wikilink a connection. The dots are coloured by top-level folder, so related areas are recognisable by colour alone; hollow dots are notes without any connection, and the more links a note has, the bigger its dot.',
          p2: 'Each top-level folder is a level. If there are several, you pick one or more in the legend below the graph – "All levels" takes them all, "None" clears the selection. Instead of a level, a tag can be the way in: the bar below lists the tags with their count, and a selected tag shows all notes carrying it, with the tag as a grey diamond. Once levels are selected, the bar only lists the tags that occur in them. Until you choose something the graph stays empty, so large collections are not built all at once; "Show all tags" brings in all tags of the bar at once.',
          p3: 'On a computer you zoom with the plus/minus buttons in the top right or the mouse wheel and pan by holding the mouse button; on the phone you zoom with two fingers and pan with one. Another button resets the view. Clicking a dot (without dragging) opens the note. Clicking a tag filters the note list by it.',
        },
        export: {
          title: 'PDF export',
          p1: 'The PDF button in the open note offers four versions: “Standard” is the note with title, timestamp and bibliography. “Clean” is the text only, without title, timestamp, page numbers or bibliography; citations stay as you wrote them. “Letter” sets the note in DIN 5008 layout for window envelopes, with the title as subject. “Print version” outputs just this note with a title page. The PDF goes to the “Notizen” dossier under Storage → Documents (earlier versions are kept alongside), the print version to the “Druckfassungen” dossier; “Open” in the message shows it right away.',
        },
        druckfassung: {
          title: 'Print version',
          p1: 'A folder becomes a formal document to hand in or print: open the folder and choose “Print version” under “…”, enter title, subtitle, name, institution, date and additional details for the title page, and decide whether to include a table of contents. Each note in the folder becomes a chapter, alphabetically by title – prefixes such as “01 Introduction” set the order; the document then shows just “Introduction”. Subfolders are included when “Include subfolders” is ticked: a subfolder becomes a heading and its notes move beneath it (“2 Basics”, “2.1 Terms”); a note named like its subfolder appears as text directly under that heading.',
          p2: 'It is typeset in a Times-like font with sans-serif headings, 2.5 cm margins, 1.5 line spacing, justified with hyphenation; page numbers start after the title page. The details stay saved with the folder. The PDF is created on your device – the first time takes a few seconds – and goes to the “Druckfassungen” file under Storage → Documents; every version is kept.',
          p3: "Citations from the folder’s source list: in the text you write [{'@'}shortname] or [{'@'}shortname, p. 12], and ^[Text] for a footnote. In the PDF – note or print version – these become Harvard citations such as “(Miller, 2020, p. 12)” with a bibliography at the end.The editor toolbar has a button for footnotes (superscript number): it inserts the brackets, and selected text becomes the footnote.",
        },
        papierkorb: {
          title: 'Trash',
          p1: 'Deleted notes go to the trash (in the menu under your name, on the phone behind the menu button) and can be restored from there or deleted permanently. When deleting a folder you decide whether its notes are deleted along with it or moved to the parent folder.',
        },
        verschluesselt: {
          title: 'End-to-end encrypted',
          p1: 'Your notes, folders, print-version details and source lists are stored on openany.de only in encrypted form: only your approved devices can read them. The server cannot see your folder names either. Search, backlinks, tags and graph are computed on your device.',
        },
        ordner: {
          title: 'Notes as files (Zettlr)',
          p1: 'In the app on your computer you can connect one of your folders to a directory: open the folder, choose “Connect to folder …” in the “…” menu and pick the directory. Each note lives there as a file “Id Title.md”, subfolders as subdirectories – the way Zettlr or Obsidian expect it. Whatever you change, rename, move or create there is picked up by the app while it is running; if both sides changed, a conflict copy is created. If you delete a file, the note goes to the trash. The directory is stored unencrypted on your computer; “Disconnect folder” leaves the files in place. Images in notes sit next to the note, under the name used in the text; a new image there that a note links to is taken into the app (media library, album “Notizen: …”).',
        },
      },
    },
    files: {
      title: 'Storage',
      subs: {
        aufbau: {
          title: 'Media library, files, documents',
          p1: 'Storage is split into three tabs at the top of the page: the media library for photos and videos, Files for anything at all, and Documents for paperwork in dossiers. When you switch tabs, the other area stays as you left it – open folders and loaded lists are not lost.',
          p2: 'Which tab opens first when you open storage is set in the settings under "Storage start"; as long as you have not chosen, it is Documents.',
        },
        dokumente: {
          title: 'Document filing',
          p1: 'In the "Documents" tab you organise PDFs, office files and text files into dossiers. Upload them with the button, or simply drag them into the open dossier. On phones and tablets there is a camera button next to it: it photographs a piece of paper and files it straight into the dossier as a document.',
          p2: 'Dossiers can be nested as deeply as you like. Dossiers and individual documents can be renamed and moved; whole dossiers can also be shared with a hub under “Shared”. Next to each dossier you see when something in it last changed, including its sub-dossiers.',
        },
        texterkennung: {
          title: 'Searchable documents',
          p1: 'Photograph a piece of paper and file it in a dossier – with the camera button or as a photo from your picture collection – and it becomes a PDF with an invisible text layer automatically. The document looks like your photo, but can be searched, selected, copied and read aloud. Recognition runs on your own device – the image is not sent anywhere for it.',
          p2: 'If you file several photos at once, you are asked whether they should become one multi-page document – the pages of a letter, say – or separate ones. The photo itself is not kept as well; if you want the image, put it in the media library. The first run takes longer because text recognition is downloaded once and then kept.',
          p3: 'If a scanned PDF is already sitting in a dossier, you can apply recognition to it afterwards: "Recognise text" reads the document page by page. If the paper is cleanly white the pages stay unchanged and only the text is added. If they are grey or unevenly lit – as with a photographed sheet – they are brightened as well; the message at the end tells you which happened. If the PDF was already searchable, it is left alone.',
        },
        dateien: {
          title: 'Managing files',
          p1: 'In the "Files" tab you create folders and upload files of any kind – one at a time or several at once. Tapping a file opens it with a suitable app on your device. The download icon saves a copy to the Download folder, or on the computer to a place of your choice; for PDFs it is in the viewer. Folders and files can be renamed and moved; folders can also be shared with a hub under “Shared”. What you can do with an open folder or dossier is under “…” on the right of the path bar. You go up through the path or with the back button, which goes one level higher.',
          p2: 'You can see how much you use on the home page tile. How much of it stays on this device is set in the settings under “Storage on this device”.',
        },
        suche: {
          title: 'Searching',
          p1: 'The magnifying glass in Files and Documents opens a search field that searches all folders or dossiers at once, not just the one currently open. Clicking a result opens the file; next to it you see where it lives, and clicking that takes you to the folder.',
          p2: 'Search finds files not only by name but also by their text: in PDFs, Word and OpenDocument texts, and text, CSV and Markdown files up to 50 MB. Openany reads this text bit by bit in the background while storage is open; until everything has been read, the search tells you how many files can for now only be found by name. For a match in the text, the list shows the passage with the search word highlighted. Scanned PDFs without a text layer only turn up once you have applied "Recognise text" to them.',
        },
        pdf: {
          title: 'Viewing and editing PDFs',
          p1: 'Clicking a PDF opens it right inside Openany. You page through it, zoom in or out, fit it to the width and search the document with the magnifying glass – matches are highlighted on the page. “Open with another app” hands it to another app. If a PDF is password-protected, Openany asks for the password first.',
          p2: 'You fill in PDF forms directly. "Edit" also brings up a toolbar: "Highlight" marks text in colour, the "Pen" lets you draw freely on the page, and "Text" places your own lines on the sheet – you choose colour, stroke width and font size, and every step can be undone. Calculated fields in forms do not calculate, though, because Openany does not run scripts from other people\'s files.',
          p3: 'You can attach comments to highlights and strokes: choose "Comments" and tap the spot; the same view lists all comments in the document. Everything is saved as ordinary PDF annotations that other programs show too. On the first save Openany moves the previous version to the trash, so you can go back to the original; if you leave the PDF with unsaved changes, it asks first.',
        },
        galerie: {
          title: 'Media library and albums',
          p1: 'The "Media library" tab holds your photos and videos – in albums with preview tiles or, without an album, below them under "Media". Albums can be nested – for instance one album per trip with a sub-album per day – and moved at any time. The lightbox flips through all pictures of an album. You move a photo in the large view; press and hold one in the grid to select several and move or delete them together. What you can do with the open album is under “…” on the right of the path bar.',
          p2: 'The newest shot comes first, grouped by month with a heading for each month. What counts is the capture date from the photo data; if there is none – screenshots or edited pictures, say – the upload day counts.',
          p3: 'In the large view you download a photo or video as the original file: to the Download folder on the phone, to a place of your choice on the computer. Albums can be shared with a hub under “Shared”. The originals stay with you. Use "Actions" in an album to upload your own cover image; it does not count as an item.',
        },
        fotosVomGeraet: {
          title: "Photos from this device",
          p1: "On Android, in the media library, “Add” → “Add photos from this device” brings in photos and videos already on the device: all of them, only certain device albums, or from a date onwards. Device albums become albums of the same name in openany; beforehand you see whether there is enough space, and what was already added is skipped. If the device is connected to openany.de, you choose once how the photos come in: on openany.de, so only the preview stays on the device and nothing needs space twice – or as a copy on this device.",
          p2: "With “Add new photos automatically” – in the media library under “Add” or under Settings → Storage and backup – whatever the camera takes from now on goes into the media library by itself, even when openany isn’t open: from the device albums you choose, starting when you turn it on. Optionally only on Wi-Fi or only while charging. Android throttles background work, so the photos arrive with some delay. On openany.de they count towards your storage. If you use Google Photos, turn off its backup – otherwise both upload.",
        },
        videos: {
          title: 'Videos and camera',
          p1: 'Besides photos, the media library also takes videos. They appear as a tile with a still image; tapping one opens it with your device’s video app.',
          p2: 'On phones and tablets the media library and every album have two camera buttons: one for a photo, one for a video. The recording lands directly wherever is currently open. On a computer the buttons are not there; you upload as usual.',
        },
        ton: {
          title: 'Audio recordings and music',
          p1: 'The media library also holds audio files such as MP3 or M4A. They appear as a tile with name and duration; a tap plays them right in the app. Audio files coming from outside are filed here too.',
          p2: 'The microphone button records directly, with pause, discard and save. The recording goes into the open album, otherwise into the album "Recordings". On Android it keeps running with the screen off, as the notification shows. There is no such button on Linux.',
          p3: 'On Android (version 10 and later) and Windows you can choose system audio instead of the microphone. The system then asks for your consent each time. Some apps block capturing, and phone calls never work. Only record conversations if everyone involved agrees.',
        },
        karte: {
          title: 'Capture date and location',
          p1: 'For every picture the lightbox shows the capture date and – if the photo contains it – the capture location; "Show on map" opens the spot on OpenStreetMap. When uploading photos from an Android phone, pick them via the Files app rather than the gallery: otherwise Android strips the location before the picture reaches Openany.',
        },
        papierkorbSpeicher: {
          title: 'Trash',
          p1: 'Deleted content – documents, files, photos, notes, contacts, but also hubs, calendars and events – first moves to the shared trash and can be restored there. After 30 days the trash empties automatically; only then is the storage space finally freed.',
        },
        dateienVerschluesselt: {
          title: 'End-to-end encrypted',
          p1: 'Your files, dossiers, photos and albums are stored on openany.de only in encrypted form: names, folders, albums, capture dates and location are in an encrypted header, and the contents themselves are encrypted with a separate key per file. Only your approved devices can read them.',
        },
      },
    },
    calendar: {
      title: 'Calendar',
      subs: {
        verwalten: {
          title: 'Multiple calendars',
          p1: 'You can keep as many calendars side by side as you like – say private, family, club – each with its own colour. A click shows or hides a calendar. At the top you switch between month, week and day; week and day show events by time, and past events are subtly greyed out. On phone and tablet you page by swiping: up and down in the month, sideways in week and day. On the computer there are arrows for this.',
        },
        termine: {
          title: 'Creating events',
          p1: 'You create events by clicking a day: with title, description, time, or as an all-day or multi-day event. Repetitions are possible too – daily, weekly, monthly or yearly. Clicking an event shows its details, where you edit or delete it.',
          p2: "A reminder alerts you at the chosen time before the start. If an event comes from another program with a more precise repetition – every two weeks, only three times, on Tuesdays and Thursdays, every first Monday – it is kept and shown in words on the event. If you pick a different repetition yourself, it replaces the old one. A single moved event of a series stays an event of its own, and the series leaves its old place free.",
        },
        abos: {
          title: 'Calendar subscriptions',
          p1: 'External calendars – such as public holidays or your club\'s fixtures – are added via a subscription URL in ICS format. Subscribed calendars refresh automatically. Their events are read-only in Openany: changes are made where the calendar comes from, because every refresh replaces the content with the original again.',
        },
        verschluesselt: {
          title: 'End-to-end encrypted',
          p1: 'Your calendars and events are stored on openany.de only in encrypted form: only your approved devices can read them. The address of a subscription is encrypted too; each device fetches it itself, openany.de fetches nothing.',
        },
        importExport: {
          title: 'Import and export',
          p1: 'Use “Import” to read an ICS file into one of your calendars. If you import the same file again, the events are updated instead of duplicated. The share icon next to a calendar saves it as an ICS file – to take along into any other calendar app.',
        },
        imTelefon: {
          title: "In the phone’s calendar",
          p1: "On Android, Settings → Calendar → “Show in the phone’s calendar” shows your calendars in every calendar app on the phone, under the account “openany” – including ones shared in hubs, and subscriptions if you add them. What you change, create or delete there comes back into openany. Reminders are then shown by the phone itself.",
          p2: "Google Calendar only shows other accounts after one step from you: there, Settings → Manage accounts → turn on “openany”. The events are decrypted only on this device. Turning this off removes them from the phone’s calendar; everything stays in openany.",
        },
        andereProgramme: {
          title: "For other programs on the computer",
          p1: "On a computer, Settings → Calendar → “Offer to other programs on this computer” offers your calendars and your address book via CalDAV and CardDAV – for Thunderbird, Apple Calendar, Apple Contacts and others. The tile shows address, user and password to copy; with them you set up an account in the program. In Thunderbird: New calendar → On the network, or New CardDAV address book. On Apple: Other CalDAV or CardDAV account → Manual. If several user accounts share the computer, only switch this on while you need it: when Openany is not running, another account could answer in its place and receive the password your program sends.",
          p2: "The service is reachable only on this computer and only while openany is running; when the app is closed, the program catches up later. Changes made there come back into openany; calendars shared in hubs are read-only there. “New password” locks out programs already set up until they get the new one.",
        },
      },
    },
    projects: {
      title: 'Hubs',
      subs: {
        grundlagen: {
          title: 'Hubs and members',
          p1: 'A hub is a shared workspace: you invite other users as members, and the invitees accept or decline the invitation. The hub owner manages the members and can also hand the hub over to another member.',
          p2: 'Every hub has four tabs: Shared, Board, Members and Chat. It opens on Shared. The numbers on the tabs show unread chat messages or how much each tab contains.',
        },
        chat: {
          title: 'Hub chat',
          p1: 'Every hub has a shared real-time chat: messages appear instantly for all members without reloading the page. The chat is end-to-end encrypted: only members can read it on their devices, openany.de cannot. When someone leaves, the key changes by itself – they cannot read anything new.',
          p2: 'Any member can pin important messages – pinned messages are easy to find for everyone in the pin bar above the history, where they can also be unpinned again.',
          p3: 'Two square brackets let you point at hub content in mid-sentence: at notes and at shared folders, files, albums and images. A suggestion list shows what is available as you type and inserts the finished link. The chain-symbol button next to the send button does the same – usually handier on a phone than hunting for two brackets.',
          p4: 'Clicking a link takes you straight to its target: into the note, into the folder with the file highlighted, or to the image in the lightbox. You can only point at what is visible in the hub anyway – anything nobody shared appears neither in the suggestion list nor as a clickable link.',
        },
        board: {
          title: 'Board',
          p1: 'Every hub has exactly one board – a shared canvas created together with the hub. Tapping “Board” opens it; “Back” returns to the tab you came from. All members draw, write and arrange on it; saving happens by itself shortly after every change. With “Embed” you place shared images, notes and files of the hub on the board as a card.',
          p2: 'If several people draw at the same time, the app merges their changes: each element takes its newest version, and deleted stays deleted. With members nearby you draw together live and see each other’s pointers; otherwise the changes arrive with the next sync. The board is end-to-end encrypted.',
        },
        geteilteKalender: {
          title: 'Sharing calendars',
          p1: 'Under “Shared”, “Share” (nearby: “Share something”) also shares one of your calendars with the hub. It then appears in the list under “Calendars”; tapping it opens it in the calendar – for members under “From hubs”. With “view” they can only read it; with “edit” they can add, change and delete events. Only you can change the calendar itself. If two people change the same event at the same time, the second version is kept next to it as a copy, with “Conflict”, date and time in the title. Everything is encrypted with a key of its own. The X next to the calendar stops sharing; your calendar stays. Without openany.de, members get it once they sync nearby – read only there; subscribed calendars can only be shared that way.',
        },
        freigaben: {
          title: 'Sharing content',
          p1: 'In the “Shared” tab, “Share” shares a notes folder, an album or a folder (or record) with the hub: pick the kind first, then which one. Everything stays end-to-end encrypted – with its own key per share, which changes when you stop sharing or someone leaves the hub. The content stays with you; when sharing ends, you keep everything.',
          p2: 'Albums and folders are shared for viewing only: members open them in the hub, see sub-albums and subfolders, open images and PDFs, and use “Take over” to put a copy into their own storage under “From hubs”. Only you can change them.',
          p3: 'Notes folders are shared with the level “view” or “edit”. Members open them in the usual note editor and also see the images in them; with “edit” they change notes and add new ones. For now only the owner of the folder can insert images.',
        },
        lokal: {
          title: 'On openany.de or nearby only',
          p1: '"+ Hub" creates a hub on this device. If the device is connected to openany.de, it goes there right away too – the list then shows a small cloud next to it. Whoever founded a hub switches this on and off under Settings → Sync → "Hubs on openany.de". Switched off, it syncs nearby only; on openany.de the hub, chat, planning and shares are deleted. With the cloud, you invite members by their openany name (owners only, and only people who have synced the app with openany.de at least once). Without it, you invite under "Invite a member nearby": Openany must be open on the other device, on the same Wi-Fi or hotspot, and both devices show the same six digits, which you compare and confirm. The hub is managed by the owner\'s devices; if there is only one, the app warns you – pair a second device of your own beforehand. When you delete a hub, this is first signed into its member list; only from that do the other members’ devices know it is gone. If openany.de reports a hub as deleted without this record, it stays on your device and keeps syncing nearby.',
          p2: 'Members share their own notes folders, folders and albums into the hub under "Share something", read-only at first. Notes folders can also be shared for editing: then only one person edits a note at a time, and it is saved on the device of the person who owns the folder – if that device is not nearby, the note stays read-only. The chat reaches everyone nearby at once, the others at the next sync. The board travels along the same way.',
          p3: 'You sync the rest with "Sync" as soon as a member is nearby. The owner can remove members; anyone who wants to leave does so with "Leave", which only works when another member is nearby to learn of it. What departed members had shared disappears for the others; what they already had stays on their devices.',
        },
      },
    },
    messages: {
      title: 'Messages',
      subs: {
        direkt: {
          title: 'Direct messages',
          p1: 'You open your messages from the menu. With "Message" at the top right you write to any other user directly – their Openany name is all you need. Links in messages are clickable.',
          p2: 'Under every received message, "Reply" opens the editor with the sender already filled in, on the same channel the message came through. Long messages are collapsed; "Read more" shows them in full. The unread counter on the menu button shows whether something new is waiting.',
          p3: 'Direct messages are end-to-end encrypted: openany.de only stores unreadable data, and only your devices can read them. All your devices share one key for this. A new device shows 6 digits; you type them in on one that is already set up, and it is signed in and has the key. The key is also in your backup file. If you lose a device, revoke it in the settings under "My devices"; the key changes when you do. You can write to anyone who has opened the app at least once.',
        },
        suche: {
          title: 'Searching and filtering',
          p1: 'Above the timeline sits a search field. It finds messages by their text, by the other person\'s name and by Matrix IDs, and highlights the matches. Below it, toggles narrow the list down: to unread messages, to those with an attachment and – if you use more than one channel – to single channels such as Openany or Matrix. The toggles can be combined.',
          p2: 'Next to the search field you switch between two views: "Timeline" shows all messages in order of time, "By contact" sums them up in one row per person, with the latest message and the number of unread ones. Clicking a row shows the timeline with exactly that person; "Close conversation" takes you back to everyone.',
        },
        anhaenge: {
          title: 'Attachments',
          p1: 'Over Matrix you can send files too: the paper clip in the compose field attaches a file of up to 10 MB, and on phones and tablets the camera button next to it takes a photo straight away. In the timeline pictures appear as a preview, other files with name and size; a click opens pictures and PDFs right inside Openany, everything else you download. The Openany channel carries files of up to 10 MB, end-to-end encrypted like the text. They count towards your storage and the recipient\'s until each of you deletes the message.',
        },
        matrixApp: {
          title: 'Messages via Matrix',
          p1: 'In the app your device itself is the Matrix device. You sign in with an existing account in the settings under "Matrix accounts", and your messages are end-to-end encrypted between this device and the other person – openany.de does not read along. So that other Matrix apps such as Element trust this device, verify it there at the account: with the recovery key from Element, or – if your account has no backup yet – openany sets one up and shows you the new key once.',
          p2: 'You can connect several Matrix accounts. Their messages share one timeline; new ones go from the default account, replies from the account the message arrived at, and "Make default" makes another account the default. If you delete a Matrix message, it disappears only on this device; it stays in the room.',
        },
        email: {
          title: 'Email via your mailbox',
          p1: 'In the app, email joins as a further channel – not a mail program of its own, but your existing mailbox, at Posteo, mailbox.org or GMX, say. In the settings under "Email mailboxes" you enter address and password; usually that is an app password you first create with your provider. The app looks up the servers itself; if it finds none, you enter them under "Servers by hand". The password stays locked away on this device, and the device talks to the mail server itself – openany.de never sees a mail.',
          p2: 'Mails appear with their subject in the shared timeline under Messages; "Fetch now" fetches new ones immediately. If you connect several mailboxes, new mails go from the default mailbox and replies from the one the mail arrived at; when writing you choose under "From". When deleting you decide whether a mail only disappears here or also moves to the trash folder on the mail server. If your provider has filed something as spam, a notice appears above the timeline; there "Not spam" brings a mail back to the inbox.',
          p3: 'Email attachments may be up to 20 MB. Received attachments go into your files with "Save" or into the download folder with "To this device". If an attachment arrived encrypted, the app asks first: in Files it lies unencrypted and is synced with openany.de.',
        },
        pgp: {
          title: 'Encrypted email with OpenPGP',
          p1: 'You can encrypt mails end to end with OpenPGP. Each mailbox needs a key of its own: in the settings at the mailbox you create it with "Create key" – or, if you already use the mailbox with PGP, in Thunderbird for instance, you import the existing one with "Import key" so both programs read the same mails. The secret key lives only on this device. "Save backup copy" puts it, locked with a passphrase, into the download folder; "Save public key" gives you the file you send to others.',
          p2: 'The app collects your contacts\' public keys under "Contacts\' keys" by itself from mails that carry them. If one is missing, you search for it by address – optionally also at keys.openpgp.org, which then learns whom you are asking about – or import it from a file. Best compare the fingerprint once with the other person; if a key changes, the app points it out.',
          p3: 'When writing you switch on "Send encrypted"; below it you see whether the recipient\'s key is known. When you reply to an encrypted mail, the switch is already on. In the timeline mails carry the mark "encrypted" and, if they are signed, "signed" – or a warning if the signature does not match.',
        },
        vorOrt: {
          title: 'Messages nearby',
          p1: 'With the "Nearby" channel you write to devices close by directly, entirely without a server. Openany must be open on the other device, and both must be on the same Wi-Fi. You can write to each other once you know each other: your own devices, members of a shared hub – or people you have confirmed nearby. For that you choose "Confirm nearby", both devices show the same 6 digits, and both tap "Matches". If the other device is not there right now, a message waits and goes across at the next meeting.',
          p2: 'At first nothing arrives from strangers. If you switch on "Allow requests from nearby devices" in the settings under "Nearby devices", they can send you up to 3 messages of 500 characters each; these appear under "Requests" above the timeline, not in the timeline itself. You can only reply after confirming with the 6 digits. Anyone who should not send you anything more, you stop with "Block". If one of your own messages was refused, it shows "not delivered", and the reason appears when you point at it.',
        },
        systemnachrichten: {
          title: 'Notifications and replies',
          p1: 'Events from your hubs – such as new shares – reach you as a notice if the sender lets the members know; so does the message that someone added you to a hub, and messages from openany.de about your account, such as the request to confirm it. Notices are not end-to-end encrypted. You also find the reply to a message sent through the support form here.',
        },
      },
    },
    contacts: {
      title: 'Address book',
      subs: {
        adressbuch: {
          title: 'Contacts and how to reach them',
          p1: 'You open the address book through its tile on the home page or at the top of the messages page; the button next to it creates a new contact right away. A contact records how someone can be reached: name, picture, phone and email plus any number of further routes – Matrix, Meshtastic, openany name, postal address, employer, birthday or other, each with its own label such as home or work. The search field finds contacts by name.',
          p2: 'The routes are clickable: a phone number places a call, an email address opens your mail program, an openany name or a Matrix ID opens the compose form with the recipient already filled in. Deleted contacts go to the trash.',
        },
        verschluesselt: {
          title: 'End-to-end encrypted',
          p1: 'Your address book is end-to-end encrypted with the same key as your direct messages. Only your approved devices can read it; openany.de stores and syncs it but sees neither names nor numbers nor pictures. A new device only shows the address book once you have approved it on another device.',
        },
        vcard: {
          title: 'Save and read vCard files',
          p1: 'Under Settings → Address book you save all contacts as a vCard file or read one from another program. Reading only adds: anyone already there is recognised by their identifier or name and keeps what they have. The file itself is not encrypted – keep it accordingly.',
        },
        imTelefon: {
          title: "In the phone’s contacts",
          p1: "On Android, Settings → Address book → “Show in the phone’s contacts” shows your contacts in the phone’s address book, under the account “openany” – when someone calls, their name appears. Name, phone, email, address, company, birthday, website, nickname and photo go across; what you change, create or delete there comes back. The phone doesn’t know Matrix, Meshtastic or Openany IDs – they stay only in openany. A photo changed there is reset by openany. Turning this off removes the contacts there.",
        },
        andereProgramme: {
          title: "For other programs on the computer",
          p1: "On a computer, Thunderbird or Apple Contacts, for example, read and write your address book via CardDAV – turned on under Settings → Calendar, with the same details as for the calendars (see there). All ways of reaching someone travel along, including Matrix, Meshtastic and the Openany name.",
        },
        sicherheitsnummer: {
          title: 'Safety number',
          p1: 'The safety number lets you check that nobody is reading along in between – not even openany.de. Open it in the contact under “Security” or in your messages after opening a conversation from “By contact”. Compare the twelve groups of digits on both of your devices; if they match, tap “It matches”. Messages from this person then carry a check mark next to the lock. If their key changes later, for example after a lost device, the check mark disappears and a notice asks you to compare again.',
        },
      },
    },
    settings: {
      title: 'Settings and account',
      subs: {
        module: {
          title: 'Modules on the homepage',
          p1: 'Here you choose which modules appear as tiles on the home page – Notes, Calendar, Storage, Hubs and Address book. The selection hides nothing; all modules remain reachable. Without a selection, the home page shows the welcome overview.',
        },
        spracheDesign: {
          title: 'Appearance and language',
          p1: 'Under "Appearance" you choose between two designs: "Clear" in teal with squarer corners and a calm surface, or "Lilac" with round corners and a patterned background. Both come in light and dark – you switch that independently with the toggle in the header or in the phone menu. Under "Language" you set which language Openany speaks to you in: German, English, Spanish, French, Portuguese, Polish or Ukrainian. Until you choose, it follows the language of your device or browser; if that one is not available, Openany speaks English.',
        },
        abgleich: {
          title: 'Syncing with Openany',
          p1: 'Under "Sync" you connect the app to openany.de. If you have no account yet, create one with "Create new account" (see “Creating an account”). If you already have a connected device, you approve the new one on it with 6 digits (see “My devices”). An older account with a password is connected with "Connect to openany.de": the app shows a code that you enter and confirm on the anyid page in the browser. Connecting is optional: without a connection everything stays on this device, and "Disconnect" undoes it at any time without losing any data here.',
          p2: 'After that, "Sync" syncs in both directions and reports afterwards what was fetched, pushed or skipped. "Fetch everything again" downloads the whole stock again instead of just what is new; nothing is deleted in the process.',
          p3: 'With "Refresh in the background" the app syncs by itself roughly every hour – only with a connection and not on a low battery. On mobile data only the list comes, files and pictures only on Wi-Fi. On Wi-Fi it also tries paired nearby devices if Openany is running on them. On a computer Openany syncs every hour while it is running; with "Keep running in the background" it starts when you log in and stays as an icon in the menu bar when you close the window.',
        },
        geraeteNah: {
          title: 'Nearby devices',
          p1: 'Your own devices also sync with each other directly, without any server – as long as both are on the same Wi-Fi or hotspot and Openany is running on both. Under "Nearby devices" you find them with "Search" and connect them with "Pair": both devices show the same code, and on both you confirm that it matches. From then on they sync by themselves while Openany is running on both, and on every "Sync"; "Forget" undoes the pairing.',
          p2: 'Under "Your name" you set how you appear to others nearby. That is the name under which people invite you into hubs – you as a person, not a single device; it applies to all your paired devices.',
        },
        meineGeraete: {
          title: 'My devices',
          p1: 'What you encrypt end-to-end can only be read by your devices; they share a key for this. A new device shows 6 digits, and you enter them here on a device that is already set up. If a device gets lost, you revoke it here; the key changes in the process. The key is also in your backup file.',
          p2: 'At the very bottom of “My devices” you delete your account on openany.de yourself – immediately and with everything stored there. To be safe, you type in your username. Hubs that still have other members you hand over to someone first.',
        },
        fristen: {
          title: 'Confirming your account and staying active',
          p1: 'So that no unused accounts pile up, openany.de asks you two days after creating your account to confirm it once more; a strip with "Keep account" appears at the top. If you do not confirm within 14 days, the account is deleted.',
          p2: 'If you do not open the app for four months, the same request comes again, with a final reminder two weeks before deletion. One tap on "Keep account" is enough. When an account is deleted, only the content on openany.de goes; what is on your devices stays.',
        },
        speicherGeraet: {
          title: 'Storage on this device',
          p1: 'Under "Storage on this device" you decide what is kept here. "Keep everything" fetches everything from your albums, dossiers and folders while there is room; it is meant for computers and tablets. With "Selected albums, dossiers and folders" only what you choose in Storage with "Keep on this device" is kept here; of everything else you only see the name. New camera photos go into the album "Camera". Anything outside an album, dossier or folder stays on its device and is never synced.',
          p2: 'Notes, calendars, contacts, messages and the names of albums, dossiers and folders go to the server. Photos and files only go there if they are shared in a hub, attached to a note, or if you choose "Put on server", individually or for a whole album, dossier or folder. Everything else is synced directly between your devices. "Remove from server" deletes nothing: your devices keep their copies, and the last copy always stays.',
          p3: 'Under "Storage on this device" you also see how much is on the server, split into messages, notes, media library, files and documents.',
        },
        sofort: {
          title: 'Notify immediately',
          p1: 'With "Notify immediately" the app keeps economical connections to Openany and to your mailboxes so new messages and mails arrive at once – entirely without Google. The mailboxes do not need Openany for this. Android shows a permanent notice for it, which you can hide in the system settings. On a computer Openany keeps watch while it is running and shows a system notification.',
        },
        sicherung: {
          title: 'Backup in one file',
          p1: 'Under "Backup", "Create backup" makes an encrypted file with everything Openany has on this device: notes, calendar, contacts, files, media library, hubs, messages and your mailboxes including password and your own OpenPGP key. The app suggests a passphrase of six words; write it down and keep it apart from the file. Without it the file cannot be opened – not even by us.',
          p2: '"Restore backup" opens such a file on another device. It replaces everything that was there; afterwards the connection to openany.de is removed, and you pair nearby devices again. What identifies a device is not included.',
        },
        tresor: {
          title: 'Credentials and keys in the vault',
          p1: 'What this device uses to identify itself to Openany and other devices is kept locked on the device; the key to it stays in the system keychain – the Keystore on Android, and on the computer the macOS Keychain, Windows Credential Manager or the Secret Service on Linux. Your mailbox passwords and your secret OpenPGP keys are kept there too. "Disconnect" only removes the credentials here – a device is revoked for good under “My devices”.',
        },
      },
    },
    contact: {
      title: 'Reporting, rules and contact',
      subs: {
        kontakt: {
          title: 'Contacting us',
          p1: 'Send questions, problems or wishes through the support form at the very bottom of this help; it goes end-to-end encrypted to the administrators, and you find the reply under Messages. Without an account you reach us through the form on openany.de/report under “Question or request”. We reply in German or English.',
        },
        bedingungen: {
          title: 'Terms of use and privacy',
          p1: 'You find the terms of use and the privacy policy under “About openany” below the support form, and the legal notice at the very end of this help; all three are also on openany.de. In short: openany is free and ad-free, your content is encrypted on your devices, and the servers are in Germany.',
        },
        melden: {
          title: 'Reporting content',
          p1: 'You report content or accounts that break the law or the terms of use through the form on openany.de/report – even without an account. "Open form" above the support form takes you there in the language of the app. The report is encrypted in your browser; only the administrators can read it.',
          p2: 'After sending you get an access code. With it you see the status and our reply on the same page and can follow up. Keep it safe – without it you can no longer reach your report. It is deleted three months after the last message.',
        },
        entscheidungen: {
          title: 'If we suspend or delete your account',
          p1: 'If an account breaks the terms of use, the administrators can suspend or delete it. If your account is suspended, a red strip appears at the top; tapping it shows since when, what the decision is based on and why. Syncing with openany.de is paused then; what is on your devices stays. When the suspension is lifted, everything continues as before.',
          p2: 'If your account is deleted, the app shows the reason once at the next sync and then disconnects; the app keeps working without openany.de. If you disagree with a decision, tap "Object": it opens the form on openany.de/report with “Objection to a decision”, and we review the decision again.',
        },
      },
    },
  },
};
