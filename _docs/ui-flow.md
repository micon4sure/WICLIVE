# WIC LIVE v4 - UI Flow

## Core Principle

No "first run" vs "subsequent run" distinction. Every launch runs the same
checks. The UI has three sections, always visible. The difference between a
fresh install and a returning user is just how many checkmarks are already
green.

## App Startup

```
Launch (UAC prompt) -> Detect game install -> Main View
```

If game install not found: show folder picker overlay before proceeding.

## Main View — Single Page, Three Sections

### Section 1: Game Readiness

Always visible at top. Runs all checks on launch automatically.
Each action shows: label + status (ok / needed / applying / error).
If an action needs applying, it runs automatically with a progress indicator.

Actions (in order):

| # | Action | Check | Apply |
|---|--------|-------|-------|
| 1 | LAA Flag | wic.exe PE header has LAA bit set | Patch PE header |
| 2 | VC++ Redist | Registry key exists | Download + silent install |
| 3 | Game Patches | Registry version == 1.0.1.1 | Download zip, extract, set registry |
| 4 | CD Key | Registry has value | Detect variant, request from backend, write |
| 5 | Proxy (Hooks) | DLLs exist + version matches | Download + extract to game dir |

When all actions are green: **Launch Game** button becomes active.

On subsequent launches most actions resolve instantly (already done).
If something changes (e.g. hooks updated on backend), it auto-applies.

### Section 2: Config

User-driven, not automatic. Accessible as a tab/panel.

- wicautoexec.txt editor
- Game config file editor
- Preset configs (pro bindings, performance tweaks, etc.)
- Read/write from `Documents\World in Conflict\`

### Section 3: Maps

User-driven. Accessible as a tab/panel.

- List of available maps from backend API
- Each map: name, uploader, date, size, version
- Status: installed / not installed / outdated (hash mismatch)
- Download with progress bar
- Upload (API key auth, time-gated)

## No Separate Views

No Init view. No Setup wizard. No Home view. Just one main view with
three sections. Game readiness is always section 1. Config and Maps are
tabs/panels below or beside it. Launch button is always visible once
readiness is green.

If game path is not detected, an overlay/dialog appears on top of the
main view to locate it. Not a separate route.
