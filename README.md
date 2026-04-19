# ICARUS Prospect Editor

A desktop utility for browsing, editing, and backing up prospect save files from the Steam game [ICARUS](https://store.steampowered.com/app/1149460/ICARUS/).

Built with [Tauri 2](https://tauri.app/), [Svelte 5](https://svelte.dev/), and a custom Rust parser for the zlib-compressed Unreal Engine property blobs embedded in each prospect.

## Features

- **Library view** — auto-detects your ICARUS prospects folder and lists every save with map, difficulty, state, elapsed time, and associated players
- **Search** — filter prospects by ID, map key, difficulty, or player name
- **Metadata editor** — edit top-level prospect fields (difficulty, state, associated members, etc.)
- **Component inspector** — drill into the decompressed UE4 blob and browse every actor/component with its properties
- **Property tree editor** — read and mutate 10+ UE4 property types (bool, int, float, string, name, enum, struct, array, object, text)
- **Safe save** — writes go through an atomic envelope: re-serialize → recompress → checksum → replace, with an automatic `.bak` backup created beside the original file

## Screenshots

_Coming soon._

## Install

Grab the latest Windows installer from the [Releases](https://github.com/WheresPizza/icarus-world-editor/releases) page:

- `.exe` — NSIS installer (recommended)
- `.msi` — standard Windows installer

## Usage

1. Launch the app. On first run, it tries to auto-detect your prospects folder at:

   ```
   %LOCALAPPDATA%\Icarus\Saved\PlayerData\<SteamID>\Prospects\
   ```

2. If auto-detect fails, open **Settings** and paste the path manually.
3. Click any prospect card to open the detail view.
4. Edit metadata on the left, or pick a component on the right to inspect and edit its properties.
5. Click **Save** to write changes back. The original file is copied to `<filename>.bak` first.

> **Back up your saves before editing.** The app creates a single `.bak` alongside each modified prospect, but keeping an extra copy somewhere safe is always a good idea.

## Build from source

Requires [Node.js 22+](https://nodejs.org/), [Rust (stable)](https://rustup.rs/), and the [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/WheresPizza/icarus-world-editor.git
cd icarus-world-editor
npm ci
npm run tauri:dev        # run in dev mode
npm run tauri:build      # produce a release installer
```

## Project layout

```
src/                     Svelte 5 frontend (TypeScript)
  lib/components/        UI components (cards, editors, property tree)
  lib/api.ts             Tauri command bindings
src-tauri/
  src/commands.rs        Tauri command handlers
  src/prospect/
    domain.rs            ProspectDocument & in-memory state
    envelope.rs          JSON wrapper + zlib + checksum handling
    property_engine.rs   UE4 property (de)serializer
    backup.rs            .bak file handling
    types.rs             Shared types
```

## Tech

- **Frontend:** Svelte 5 (runes mode) + Vite 8 + TypeScript 6
- **Backend:** Rust + Tauri 2 with `tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-log`
- **Parser:** Custom UE4 property reader/writer (no external UE4 dependency) over `flate2` + `byteorder`

## Troubleshooting

**The app won't open on Windows.** Grab the log folder and open an issue:

1. Press `Win + R`, paste `%APPDATA%\ICARUS Prospect Editor\logs`, press Enter
2. Zip the folder
3. Open an issue at [github.com/WheresPizza/icarus-world-editor/issues](https://github.com/WheresPizza/icarus-world-editor/issues) and attach the zip

## Disclaimer

This is an unofficial community tool. **ICARUS** is a trademark of RocketWerkz. This project is not affiliated with, endorsed by, or supported by RocketWerkz in any way.

## License

[MIT](./LICENSE) — use it, fork it, ship it.
