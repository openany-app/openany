# Help

Here you will find descriptions and instructions for every module and feature of Openany. Use the table of contents on the left to jump straight to the right section.

- [Getting started](#getting-started)
- [Notes](#notes)
- [Storage](#storage)
- [Calendar](#calendar)
- [Projects](#projects)
- [Messages](#messages)
- [Address book](#address-book)
- [Settings and account](#settings-and-account)

## Getting started

### The app without an account

The app is a complete program that also works without any account or server: notes, calendar, storage, address book and messages then live only on this device. Whatever is not set up here or does not work, the app hides instead of showing it empty – the Openany channel for messages, for instance, only appears once the device is connected to openany.de.

If you want to sync your data with Openany on the web, tap "Connect to openany.de" in the settings under "Sync"; you pair with your other devices under "Nearby devices". After that, a "Sync" button appears in the header and the menu, and it spins while syncing.

### The home page

The home page shows compact tiles of the modules you selected for it: your upcoming events, recently edited notes, the latest files with a storage bar, your projects with unread counters, the newest messages in your inbox and the address book. Each tile leads into its module; which ones appear is configured in the settings under "Modules on the homepage".

If you have not selected any modules for the home page, the welcome overview appears instead: it introduces all features and lets you assemble your tiles right away. The selection hides nothing – every area remains reachable through the bar and the menu.

### Menu and layout

On a computer there is a header at the top: on the left the round logo takes you to the home page, next to it are Notes, Calendar, Storage and Projects. On the right follow the light/dark toggle, the envelope for messages with its unread counter, and the pill with your name – it opens Settings, Help, Trash and Log out.

On the phone everything moves to the bottom: a fixed bar shows the modules as icons, with the menu button and its unread-message counter at the far left. It opens a panel with Messages, Settings, Help, Trash, the light/dark toggle and Log out. The toggle switches between light, dark and automatic – automatic follows your device's setting.

## Notes

### Writing in the editor

The note editor works like a modern word processor: headings, lists, quotes, code blocks, bold and italics are available from the toolbar. Alternatively type markdown shortcuts – for example a hash followed by a space for a heading or a dash for a list – and they turn into formatting as you write.

Saving is automatic: shortly after you stop typing, Openany saves the note and shows the save status in the toolbar. The created and last-modified dates are also shown under each note. On the phone the toolbar appears as a single scrollable row right above the keyboard as soon as you type in the text – so every tool is within reach without getting in the way while reading.

### Folders and structure

Notes live in folders that can be nested as deeply as you like. Notes and folders can be renamed, moved and rearranged – over time this grows into your own knowledge base. The search field above the list searches all notes at once, title and content, regardless of the folder currently open.

You can download an entire folder including its subfolders as a ZIP archive; the notes inside are markdown files and stay readable everywhere.

### Wikilinks and backlinks

Two square brackets link notes to each other: type the brackets and the title of the target note and the link is created automatically – if the note does not exist yet, the link becomes active as soon as it does. If you want different wording in the text than the title, add your own display text after a vertical bar; the link still points to the right note.

Every note shows its backlinks: a list of all notes linking to it. This way you rediscover connections without keeping track yourself.

### Tags

You assign tags right in the text with a hash in front of the word; a suggestion list of existing tags helps as you type. A slash lets you nest tags – for example Project/Subtopic; filtering on the parent then also finds all subtopics. In the graph view the tags can be shown as their own nodes, and clicking a tag there filters the note list to it.

### Inserting images and files

The paperclip button in the toolbar inserts content into a note – either freshly uploaded or as a reference to content already stored in Openany. Images appear as a space-saving preview right inside the note (the full resolution lives in the gallery), other files as a clickable download link.

Inserted content is filed automatically: images into a gallery album, text and office files into a document dossier, everything else into a file folder – each in a dedicated area for notes. As long as the reference sits in at least one note, the file cannot be moved to the trash from Storage or Gallery; remove the reference from the text and it becomes freely deletable again.

### Citations and references

For academic work you can assign a reference file in CSL-JSON format to a folder – for example an export from Zotero that you uploaded to Storage beforehand. The "Link library" button on the folder makes the connection; subfolders inherit the library automatically.

In notes within that folder the editor suggests matching sources as you cite and inserts a short reference into the text. This way citations appear while writing, without retyping the source details every time.

### Graph view

The graph view draws your notes as a network: every note is a dot, every wikilink a connection. The dots are coloured by top-level folder, so related areas are recognisable by colour alone; hollow dots are notes without any connection, and the more links a note has, the bigger its dot.

Each top-level folder is a level. If there are several, you pick one or more in the legend below the graph – "All levels" takes them all, "None" clears the selection. Instead of a level, a tag can be the way in: the bar below lists the tags with their count, and a selected tag shows all notes carrying it, with the tag as a grey diamond. Once levels are selected, the bar only lists the tags that occur in them. Until you choose something the graph stays empty, so large collections are not built all at once; "Show all tags" brings in all tags of the bar at once.

On a computer you zoom with the plus/minus buttons in the top right or the mouse wheel and pan by holding the mouse button; on the phone you zoom with two fingers and pan with one. Another button resets the view. Clicking a dot (without dragging) opens the note. Clicking a tag filters the note list by it.

### Automatic overview

Alongside the graph there is an automatic overview: it lists all note titles from A to Z and below shows a collapsible tree of your tags (nested tags appear as branches). Clicking a title opens the note, clicking a tag filters the list – all without you having to maintain a table of contents yourself.

### PDF export

Every note can be downloaded as a PDF – for printing, archiving or passing on. The formatting from the editor is preserved.

### Trash

Deleted notes go to the trash (in the menu under your name, on the phone behind the menu button) and can be restored from there or deleted permanently. When deleting a folder you decide whether its notes are deleted along with it or moved to the parent folder.

## Storage

### Gallery, files, documents

Storage is split into three tabs at the top of the page: the gallery for photos and videos, Files for anything at all, and Documents for paperwork in dossiers. When you switch tabs, the other area stays as you left it – open folders and loaded lists are not lost.

Which tab opens first when you open storage is set in the settings under "Storage start"; as long as you have not chosen, it is Documents.

### Document filing

In the "Documents" tab you organise PDFs, office files and text files into dossiers. Upload them with the button, or simply drag them into the open dossier. On phones and tablets there is a camera button next to it: it photographs a piece of paper and files it straight into the dossier as a document.

Dossiers can be nested as deeply as you like. Both dossiers and individual documents can be renamed and moved; whole dossiers can also be shared into projects and downloaded as a ZIP.

### Searchable documents

Photograph a piece of paper and file it in a dossier – with the camera button or as a photo from your picture collection – and it becomes a PDF with an invisible text layer automatically. The document looks like your photo, but can be searched, selected, copied and read aloud. Recognition runs on your own device – the image is not sent anywhere for it.

If you file several photos at once, you are asked whether they should become one multi-page document – the pages of a letter, say – or separate ones. The photo itself is not kept as well; if you want the image, put it in the gallery. The first run takes longer because text recognition is downloaded once and then kept.

If a scanned PDF is already sitting in a dossier, you can apply recognition to it afterwards: "Recognise text" reads the document page by page. If the paper is cleanly white the pages stay unchanged and only the text is added. If they are grey or unevenly lit – as with a photographed sheet – they are brightened as well; the message at the end tells you which happened. If the PDF was already searchable, it is left alone.

### Managing files

In the "Files" tab you create folders and upload files of any kind – individually or several at once. Both folders and files can be renamed and moved, files downloaded again; folders can also be shared into projects and downloaded as a ZIP.

Every account has a storage quota; the current usage is shown on the home page tile and in the settings under "Storage".

### Searching

The magnifying glass in Files and Documents opens a search field that searches all folders or dossiers at once, not just the one currently open. Clicking a result opens the file; next to it you see where it lives, and clicking that takes you to the folder.

Search finds files not only by name but also by their text: in PDFs, Word and OpenDocument texts, and text, CSV and Markdown files up to 50 MB. Openany reads this text bit by bit in the background while storage is open; until everything has been read, the search tells you how many files can for now only be found by name. For a match in the text, the list shows the passage with the search word highlighted. Scanned PDFs without a text layer only turn up once you have applied "Recognise text" to them.

### Viewing and editing PDFs

Clicking a PDF opens it right inside Openany, without downloading. You page through it, zoom in or out, fit it to the width and search the document with the magnifying glass – matches are highlighted on the page. If a PDF is password-protected, Openany asks for the password first.

You fill in PDF forms directly. "Edit" also brings up a toolbar: "Highlight" marks text in colour, the "Pen" lets you draw freely on the page, and "Text" places your own lines on the sheet – you choose colour, stroke width and font size, and every step can be undone. Calculated fields in forms do not calculate, though, because Openany does not run scripts from other people's files.

You can attach comments to highlights and strokes: choose "Comments" and tap the spot; the same view lists all comments in the document. Everything is saved as ordinary PDF annotations that other programs show too. On the first save Openany moves the previous version to the trash, so you can go back to the original; if you leave the PDF with unsaved changes, it asks first.

### Photo gallery and albums

The "Gallery" tab holds your photos and videos – in albums with preview tiles or, without an album, below them under "Photos". Albums can be nested – for instance one album per trip with a sub-album per day – and moved at any time. The lightbox flips through all pictures of an album.

The newest shot comes first, grouped by month with a heading for each month. What counts is the capture date from the photo data; if there is none – screenshots or edited pictures, say – the upload day counts.

Whole albums can be downloaded as ZIP, single pictures as the original file. Albums can be shared into projects as well.

### Videos and camera

Besides photos, the gallery also takes videos. They appear as a tile with a still frame and play in the lightbox. Some phones record in HEVC, which not every browser can play; in that case download the video and open it with another program.

On phones and tablets the gallery and every album have two camera buttons: one for a photo, one for a video. The recording lands directly wherever is currently open. On a computer the buttons are not there; you upload as usual.

### Capture date and location

For every picture the lightbox shows the capture date and – if the photo contains it – the capture location; "Show on map" opens the spot on OpenStreetMap. When uploading photos from an Android phone, pick them via the Files app rather than the gallery: otherwise Android strips the location before the picture reaches Openany.

### Trash

Deleted content – documents, files, photos, notes, contacts, but also projects, boards, calendars and events – first moves to the shared trash and can be restored there. After 30 days the trash empties automatically; only then is the storage space finally freed.

## Calendar

### Multiple calendars

You can keep any number of calendars side by side – for example private, family, club – each with its own colour. Individual calendars can be shown and hidden with a click. At the top you switch between month and week view; the week shows events by time of day, and past events are subtly greyed out.

### Creating events

You create events by clicking a day: with title, description, time, or as an all-day or multi-day event. Recurring events are supported too – daily, weekly, monthly or yearly. Existing events are edited or deleted right from the view. Times apply in the time zone set in your profile.

### Calendar subscriptions

External calendars – such as public holidays or your club's fixtures – are added via a subscription URL in ICS format. Subscribed calendars refresh automatically. Their events are read-only in Openany: changes are made where the calendar comes from, because every refresh replaces the content with the original again.

The other way round you can share your own calendars via a subscription link: the feed icon in the calendar list creates a secret URL that Google, Apple or Outlook can subscribe to – changes arrive there automatically. Treat the link like a password; "Renew link" invalidates the old one immediately.

## Projects

### Projects and members

A project is a shared workspace: you invite other users as members, and the invitees accept or decline the invitation. The project owner manages the members and can also hand the project over to another member.

Every project has four tabs: Chat, Planning, Members and Shared. The numbers on the tabs show unread chat messages or how much each tab contains.

### Project chat

Every project has a shared real-time chat: messages appear instantly for all members without reloading the page.

Any member can pin important messages – pinned messages are easy to find for everyone in the pin bar above the history, where they can also be unpinned again.

Two square brackets let you point at project content in mid-sentence: at notes, at boards, roadmaps and place collections including the individual cards, milestones and places inside them, and at shared folders, files, albums and images. A suggestion list shows what is available as you type and inserts the finished link. The chain-symbol button next to the send button does the same – usually handier on a phone than hunting for two brackets.

Clicking a link takes you straight to its target: into the note, onto the board with the card opened, into the folder with the file highlighted, or to the image in the lightbox. You can only point at what is visible in the project anyway – anything nobody shared appears neither in the suggestion list nor as a clickable link.

### The planning tab

The "Planning" tab collects the project's planning tools. Via the "New" button you pick the right type: scheduling, survey, shift plan, bring list, guest list, board, roadmap, places, school holidays, timetable or care plan. The "Preset: School" template creates school holidays, a timetable, tests and a homework board in one go. While doing so you pick your federal state, so the holidays are in right away, and optionally your school's bell schedule. Any member may create and edit; deleting is reserved for the respective creator or the project owner.

### Scheduling

With scheduling you propose several date options and everyone votes yes, no or maybe – so you agree on a meeting without endless back and forth.

### Surveys

A survey asks one question with fixed answers – yes/no or your own, anonymous if desired. With checkable options it also serves as a shared task list; polls can be duplicated and closed.

### Shift plan, bring list, guest list

Three list types take organisational work off your hands: with the shift plan you create shifts with slots and members sign up. The bring list clarifies who brings what – entries can be claimed and added. The guest list keeps track of guests (invited, accepted, declined), including guests without an Openany account.

### Kanban boards

On kanban boards you organise tasks in freely arranged columns and cards – for example "Open", "In progress", "Done". Cards are moved by dragging; changes appear live for all members. A card can also carry a place from the project's place collections and a subject – so a homework task belongs to the matching lesson.

### Roadmap

The roadmap shows milestones with date and status on a timeline – so the whole team sees what should be done by when. Milestones can be marked as reached, overdue ones are flagged. Like cards, milestones can also carry a place and a subject, a test for instance.

### Places

Under places you collect map points on OpenStreetMap – meeting points, addresses, parking spots. You drop the point by clicking the map, optionally with a name and note; clicking the name of a place tile opens the point directly on OpenStreetMap. Once collected, you simply pick the places elsewhere: on cards, milestones, subjects and entries in the timetable or care plan.

### School holidays

School holidays record a school year: its period and the days off within it. You can import your region's holidays as an ICS file and add individual training days and public holidays by hand. The difference matters: during holidays a different care arrangement often applies, while on a single day off it continues in the usual rhythm. The import only adds and replaces nothing you entered yourself.

### Timetable and care plan

Both are the same building block: a weekly grid from Monday to Sunday that repeats week after week. The timetable carries times and subjects, the care plan whole days and a person – "who has the child when". Link school holidays to it and lessons drop out during the holidays by themselves; a care plan can be limited to term time or to the holidays. If your rhythm alternates weekly, switch to A/B weeks: either every two weeks from the Monday of week A, or by even and odd calendar weeks if your arrangement says so in writing – the rhythm then jumps, though, in years with 53 calendar weeks.

Below the grid are the periods. They override it within their date range and cover everything that is not a rhythm: the summer holidays split in half, a swapped weekend, a school trip. The time zone belongs to the plan, not to the viewer – so the timetable shows the same times for both parents, even if one of them lives abroad.

### Subjects

You maintain a project's subjects via "Subjects" in the timetable: name, abbreviation, teacher, colour and optionally a place. They belong to the project and not to a single plan, so they survive the change of term. A subject decides what a lesson in the timetable shows, and links homework cards and tests to the matching lesson. If you delete a subject, cards, milestones and lessons only lose the reference to it – they themselves stay.

### Sharing content

The "Shared" tab holds everything members have made available to the project: note folders, albums, and folders and dossiers from storage. You share each item where it lives – in Notes, in the Gallery or in Storage – either with the level "Read only" or "Edit". A share applies to all project members, is inherited by subfolders and sub-albums, and can be removed at any time. The content itself stays with its owner; when a share ends, everything remains theirs.

Clicking a share opens it inside the project: notes in the familiar workspace with search, backlinks and a graph across all shared notes of the project; albums as a photo grid with lightbox; folders as a file list. With "Edit" members can contribute themselves – write notes, upload photos, drop files and create subfolders; "Read only" allows viewing and downloading.

### Shared project notes

In a note folder shared with "Edit", members can create, edit and delete notes and create subfolders right inside the project; embedded images and files are visible to all members, and anyone allowed to edit can insert their own. Everything created – including uploaded images and files – belongs to the folder's owner and counts against their storage; deleted notes land in their trash, so nothing is lost for good.

### Nearby-only projects

In the app, "+ Project" creates a project on this device only – without an account and without a server; it carries the mark "Nearby only". You invite others under "Invite a member nearby": Openany must be open on the other device, on the same Wi-Fi or hotspot, and both devices show the same six digits, which you compare and confirm. The project is managed by the owner's devices; if there is only one, the app warns you – pair a second device of your own beforehand.

Members share their own notes folders, folders and albums into the project under "Share something", read-only at first. Notes folders can also be shared for editing: then only one person edits a note at a time, and it is saved on the device of the person who owns the folder – if that device is not nearby, the note stays read-only. The chat reaches everyone nearby at once, the others at the next sync. The planning – boards, roadmaps, places and school planning – travels along the same way.

You sync the rest with "Sync" as soon as a member is nearby. The owner can remove members; anyone who wants to leave does so with "Leave", which only works when another member is nearby to learn of it. What departed members had shared disappears for the others; what they already had stays on their devices.

## Messages

### Direct messages

You reach the messages on a computer via the envelope in the header, on the phone via the menu button at the bottom left. "Message" at the top right lets you write directly to any other user – their Openany name is all you need as recipient. New messages appear instantly without reloading the page, and links in them are clickable.

Under every message you receive, "Reply" opens the compose field with the sender already filled in, on the same channel the message came by. Long messages are collapsed; "Read more" shows them in full. The unread counter always shows whether something new is waiting – on a computer on the envelope, on a phone on the menu button in the bottom bar.

### Searching and filtering

Above the timeline sits a search field. It finds messages by their text, by the other person's name and by Matrix IDs, and highlights the matches. Below it, toggles narrow the list down: to unread messages, to those with an attachment and – if you use more than one channel – to single channels such as Openany or Matrix. The toggles can be combined.

Next to the search field you switch between two views: "Timeline" shows all messages in order of time, "By contact" sums them up in one row per person, with the latest message and the number of unread ones. Clicking a row shows the timeline with exactly that person; "Close conversation" takes you back to everyone.

### Attachments

Over Matrix you can send files too: the paper clip in the compose field attaches a file of up to 10 MB, and on phones and tablets the camera button next to it takes a photo straight away. In the timeline pictures appear as a preview, other files with name and size; a click opens pictures and PDFs right inside Openany, everything else you download. The Openany channel carries text only.

### Messages via Matrix

In the app your device itself is the Matrix device. You sign in with an existing account in the settings under "Matrix accounts", and your messages are end-to-end encrypted between this device and the other person – openany.de does not read along. The app cannot yet do device verification by emoji comparison; other Matrix programs therefore list it as an unverified device, but it is encrypted all the same.

You can connect several Matrix accounts. Their messages share one timeline; new ones go from the default account, replies from the account the message arrived at, and "Make default" makes another account the default. If you delete a Matrix message, it disappears only on this device; it stays in the room.

### Email via your mailbox

In the app, email joins as a further channel – not a mail program of its own, but your existing mailbox, at Posteo, mailbox.org or GMX, say. In the settings under "Email mailboxes" you enter address and password; usually that is an app password you first create with your provider. The app looks up the servers itself; if it finds none, you enter them under "Servers by hand". The password stays locked away on this device, and the device talks to the mail server itself – openany.de never sees a mail.

Mails appear with their subject in the shared timeline under Messages; "Fetch now" fetches new ones immediately. If you connect several mailboxes, new mails go from the default mailbox and replies from the one the mail arrived at; when writing you choose under "From". When deleting you decide whether a mail only disappears here or also moves to the trash folder on the mail server. If your provider has filed something as spam, a notice appears above the timeline; there "Not spam" brings a mail back to the inbox.

Email attachments may be up to 15 MB. Received attachments go into your files with "Save" or into the download folder with "To this device". If an attachment arrived encrypted, the app asks first: in Files it lies unencrypted and is synced with openany.de.

### Encrypted email with OpenPGP

You can encrypt mails end to end with OpenPGP. Each mailbox needs a key of its own: in the settings at the mailbox you create it with "Create key" – or, if you already use the mailbox with PGP, in Thunderbird for instance, you import the existing one with "Import key" so both programs read the same mails. The secret key lives only on this device. "Save backup copy" puts it, locked with a passphrase, into the download folder; "Save public key" gives you the file you send to others.

The app collects your contacts' public keys under "Contacts' keys" by itself from mails that carry them. If one is missing, you search for it by address – optionally also at keys.openpgp.org, which then learns whom you are asking about – or import it from a file. Best compare the fingerprint once with the other person; if a key changes, the app points it out.

When writing you switch on "Send encrypted"; below it you see whether the recipient's key is known. When you reply to an encrypted mail, the switch is already on. In the timeline mails carry the mark "encrypted" and, if they are signed, "signed" – or a warning if the signature does not match.

### Messages nearby

With the "Nearby" channel you write to devices close by directly, entirely without a server. Openany must be open on the other device, and both must be on the same Wi-Fi. You can write to each other once you know each other: your own devices, members of a shared project – or people you have confirmed nearby. For that you choose "Confirm nearby", both devices show the same 6 digits, and both tap "Matches". If the other device is not there right now, a message waits and goes across at the next meeting.

At first nothing arrives from strangers. If you switch on "Allow requests from nearby devices" in the settings under "Nearby devices", they can send you up to 3 messages of 500 characters each; these appear under "Requests" above the timeline, not in the timeline itself. You can only reply after confirming with the 6 digits. Anyone who should not send you anything more, you stop with "Block". If one of your own messages was refused, it shows "not delivered", and the reason appears when you point at it.

### Notifications and replies

Events from your projects – such as new surveys or shares – reach you as automatic direct messages, provided the sender chose to notify the members. The reply to a message sent via the contact form also shows up here.

## Address book

### Contacts and how to reach them

You open the address book at the top of the messages page; the button next to it creates a new contact right away. A contact records how someone can be reached: name, picture, phone and email plus any number of further routes – Matrix, Meshtastic, openany name, postal address, employer, birthday or other, each with its own label such as home or work. The search field finds contacts by name.

The routes are clickable: a phone number places a call, an email address opens your mail program, an openany name or a Matrix ID opens the compose form with the recipient already filled in. A contact can be linked to an Openany account – that is only a reference and grants no access to projects or content. Deleted contacts go to the trash.

## Settings and account

### Modules on the homepage

Here you choose which modules appear as tiles on the home page – Notes, Calendar, Storage, Projects, Messages and Address book. The selection hides nothing; all modules remain reachable. Without a selection, the home page shows the welcome overview.

### Appearance and language

Under "Appearance" you choose between two designs: "Clear" in teal with squarer corners and a calm surface, or "Lilac" with round corners and a patterned background. Both come in light and dark – you switch that independently with the toggle in the header or in the phone menu. Under "Language" you set which language Openany speaks to you in: German, English, Spanish, French, Portuguese, Polish or Ukrainian. Until you choose, it follows the language of your device or browser; if that one is not available, Openany speaks English.

### Syncing with Openany

Under "Sync" you connect the app to your account with "Connect to openany.de". The app shows a code that you type in and confirm on anyid's page in the browser. This confirmation in the browser is not a detour but the proof that it really is you adding the device. Connecting is optional: without a connection everything stays on this device, and "Disconnect" undoes it at any time without losing data here.

After that, "Sync" syncs in both directions and reports afterwards what was fetched, pushed or skipped. "Fetch everything again" downloads the whole stock again instead of just what is new; nothing is deleted in the process.

With "Refresh in the background" the app syncs by itself roughly every hour – only with a connection and not on a low battery. On mobile data only the list comes, files and pictures only on Wi-Fi. Nearby devices are not included; for them the app has to be open.

### Nearby devices

Your own devices also sync with each other directly, without any server – as long as both are on the same Wi-Fi or hotspot and Openany is open on both. Under "Nearby devices" you find them with "Search" and connect them with "Pair": both devices show the same code, and on both you confirm that it matches. From then on they sync with each other on every "Sync"; "Forget" undoes the pairing.

Under "Your name" you set how you appear to others nearby. That is the name under which people invite you into projects – you as a person, not a single device; it applies to all your paired devices.

### Storage on this device

Under "Storage on this device" you decide how much of your files and pictures the app keeps at hand. "On demand" shows all files in the list but only fetches the content when you open it. With "Selected folders and albums", whatever you marked in storage with "Keep on this device" always stays on the device, sub-folders included. "Keep everything" fetches what is missing after every sync, as long as there is room.

### Backup in one file

Under "Backup", "Create backup" makes an encrypted file with everything Openany has on this device: notes, calendar, contacts, files, gallery, projects, messages and your mailboxes including password and your own OpenPGP key. The app suggests a passphrase of six words; write it down and keep it apart from the file. Without it the file cannot be opened – not even by us.

"Restore backup" opens such a file on another device. It replaces everything that was there; afterwards the connection to openany.de is removed, and you pair nearby devices again. What identifies a device is not included.

### Notify immediately

With "Notify immediately" the app keeps economical connections to Openany and to your mailboxes so new messages and mails arrive at once – entirely without Google. The mailboxes do not need Openany for this. Android shows a permanent notice for it, which you can hide in the system settings.

### Credentials and keys in the vault

What this device uses to identify itself to Openany and other devices lies locked on the device; the key to it stays in the Android Keystore. Your mailbox passwords and your secret OpenPGP keys are kept there too. "Disconnect" removes the credentials only here – they are finally revoked in anyid's device list and under Openany's app passwords.
