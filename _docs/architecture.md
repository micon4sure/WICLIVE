# WIC LIVE v4 - Architecture

## Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **Target**: Windows x64, always runs as administrator
- **Backend API**: wiclive backend (Hono + SQLite on Bun)
- **Design**: wicgate design language/branding

## Design Principles

1. **Always elevated** — app requires admin from the start via Windows manifest.
   No runtime UAC prompts, no elevation juggling.
2. **No installer automation** — patches are pre-extracted zips, not InstallShield
   exes driven by AutoIt. We own the file delivery.
3. **CD keys from backend** — no hardcoded keys. Each user gets a unique key
   from the wicgate keygen system via API.
4. **Fixes as actions** — all setup/fix steps are defined as a global list of
   actions. Each action has a check (is it needed?) and an apply (do it).
   The setup flow iterates the list. No bespoke per-fix code paths.
5. **wicgate branding** — consistent design language with the wicgate ecosystem.

## Action System

All setup tasks are modeled as actions with a uniform interface:

```
Action {
  id: string           // e.g. "patch", "vcredist", "hooks", "cdkey"
  label: string        // display name
  check() -> Status    // returns: done | needed | error
  apply() -> Result    // performs the fix, emits progress events
}
```

The setup view iterates all actions in order:
1. Run `check()` for each
2. If `needed`, run `apply()`
3. Show progress/status per action
4. All done -> Home

Actions are defined once in a global registry. Adding a new fix = adding
one action definition. No touching the setup flow.

### Defined Actions

| # | ID | Label | Check | Apply |
|---|-----|-------|-------|-------|
| 1 | `laa` | LAA Flag | wic.exe PE header has LAA bit | patch PE header |
| 2 | `vcredist` | VC++ Redist | registry key exists | download + silent install |
| 3 | `patch` | Game Patches | registry version == 1.0.1.1 | download + extract patch zip, set registry |
| 4 | `cdkey` | CD Key | registry key has value | detect variant, request from backend, write |
| 5 | `hooks` | Proxy (Hooks) | DLLs exist + version matches | download + extract hooks zip |

## CD Key Flow

The old client used two hardcoded shared keys (one vanilla, one Soviet Assault).
The new client requests a unique key from the wicgate backend keygen.

### Product IDs (baked into the key, 3 bits)

| ID | Constant | Meaning |
|----|----------|---------|
| 1 | `PRODUCT_ID_WIC07_STANDARD_KEY` | WIC vanilla |
| 2 | `PRODUCT_ID_WIC07_TIMELIMITED_KEY` | WIC vanilla (time-limited) |
| 3 | `PRODUCT_ID_WIC08_STANDARD_KEY` | WIC Soviet Assault |
| 4 | `PRODUCT_ID_WIC08_TIMELIMITED_KEY` | WIC Soviet Assault (time-limited) |

### Group Membership (stored in DB, not in key)

| Group | Bit | Permission |
|-------|-----|------------|
| 0 | — | Player (no server perms) |
| 16 | 4 | Ranked server |
| 32 | 5 | Tournament server |
| 64 | 6 | Clan match server |
| 255 | all | All permissions |

### Flow

1. Client detects game variant (vanilla vs Soviet Assault)
2. Client checks registry for existing CD key
3. If missing: request key from backend with correct `productId` (1 for vanilla, 3 for Soviet)
4. Backend generates unique key, inserts into massgate DB, returns `regKey`
5. Client writes key to registry

One key per variant. If the user has vanilla, they get a product 1 key.
If they have Soviet Assault, they get a product 3 key. No pairs.

> **Open question**: auth strategy for key provisioning endpoint.
> Current admin endpoint requires JWT. Need a client-facing endpoint.
> Options:
> - Dedicated endpoint on wiclive backend that proxies to keygen
> - Public keygen endpoint with rate limiting
> - Hardware-bound key assignment (tie to machine ID)

## Rust Modules

### `actions` (new)
- Action trait: `check()` and `apply()`
- Action registry: ordered list of all actions
- Each action is a separate impl

### `install`
- Detect game installation path from Windows registry
- Read game version from registry
- Validate install path (wic.exe exists)

### `laa`
- Action: check LAA (Large Address Aware) bit in wic.exe PE header
- If not set: patch the PE header to enable it
- Allows the game to use >2GB RAM on 64-bit systems

### `patch`
- Action: compare registry version against target (`1.0.1.1`)
- Download the correct patch zip from backend:
  - `1.0.0.0` -> full patch zip (~1.3GB compressed)
  - `1.0.1.0` -> patch 11 only zip (~400MB compressed)
  - `1.0.1.1` -> already patched, skip
- Extract zip over game directory
- Set registry version to `1.0.1.1`

### `vcredist`
- Action: check registry, download + silent install if missing

### `hooks`
- Action: check DLLs + version, download + extract if needed

### `cdkey`
- Action: check registry, request from backend if missing

### `maps`
- List local .sdf files with MD5 hashes
- Fetch metadata from backend
- Download maps with progress

### `game`
- Launch `wic.exe` from install path

## Frontend — Single Page, Three Sections

No separate views/routes. One main view with three sections.
If game path not found, overlay dialog to locate it.

### Section 1: Game Readiness
- Runs all actions on launch automatically
- Each action: label + status (ok / needed / applying / error)
- Progress bars for downloads/extractions
- Launch Game button active when all green

### Section 2: Config (tab/panel)
- wicautoexec.txt editor
- Game config file editor
- Preset configs (pro bindings, etc.)

### Section 3: Maps (tab/panel)
- Available maps from backend API
- Local install status per map (hash comparison)
- Download with progress
- Upload (API key auth, time-gated)

## Tauri v2 Configuration

### Capabilities
- `core:default` (invoke, events)
- `dialog:default` (folder picker for game directory)
- `opener:default` (open URLs in browser)

### Plugins
- `tauri-plugin-opener` — open URLs/files externally
- `tauri-plugin-dialog` — folder picker

## Backend API Endpoints (wiclive backend)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/maps/data` | All map metadata |
| GET | `/maps/download/:filename` | Download a map file |
| POST | `/maps/upload` | Upload a map (API key + time gate) |
| GET | `/patches/full` | Full patch zip (1.0.0.0 -> 1.0.1.1) |
| GET | `/patches/p11` | Patch 11 only zip (1.0.1.0 -> 1.0.1.1) |
| GET | `/hooks/latest` | Latest hooks zip + version info |
| POST | `/cdkey/provision` | Request a new CD key (TODO: auth strategy) |

> Note: patch, hook, and cdkey endpoints are not yet implemented in the backend.

## Headless Mode (future, separate)

A simple Bun CLI executable for server-side use cases:
- Map sync (pull all maps from backend to server)
- Set CD key (e.g. ranked key provisioned out of band)

Not part of the Tauri app. Separate Bun script, built independently.
Not a priority for v4 launch.
