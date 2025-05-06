# ModListBot

The official Discord bot for Skatebit, using the same API powering Skatebit's mod lists.

ModListBot lets you browse and search the current working mod list for Skater XL directly in Discord. It pulls data from the Skatebit API so you always get up‑to‑date information on Alpha, Beta, and Public mod branches.

## ⚙️ Features

- Browse Full Lists: /modlist version:<alpha|beta|public> [dm:true]Paginated embeds display the entire mod list in chunks. Optionally DM the full list to yourself.
- Search Specific Mods: /modsearch version:<alpha|beta|public> query:<mod-name>
  Autocomplete suggestions as you type, then display details for the matching mod.
- Invite‑Ready: ModListBot uses global commands—invite it to any server via the Invite button on its Discord profile (or by generating an OAuth2 link).

## 💬 Commands Reference

/ping
Bot replies with Pong!

/modlist version:<alpha|beta|public> [dm:true]
Browse the full mod list for a specific version.

/modsearch version:<alpha|beta|public> query:<mod-name>
Search for a specific mod by name within a version branch.
