# WIC LIVE v4 - Tauri Commands

All commands are invoked from the Vue frontend via `invoke()`.

## Install / Detection

### `get_install_path() -> Option<String>`
Find the game installation directory from registry.
Checks standard + GOG registry keys. Returns None if not found.

### `get_game_version() -> Option<String>`
Read `Version` value from game registry key.
Returns the version string (e.g. "1.0.0.0", "1.0.1.0", "1.0.1.1").

### `set_install_path(path: String) -> Result<(), String>`
Manually set the install path (user picked via dialog).
Validates that wic.exe exists at the given path.

## Patching

### `get_patch_status() -> PatchStatus`
Returns which patch is needed based on current registry version.
```
PatchStatus { current_version: String, needs_full: bool, needs_p11: bool, up_to_date: bool }
```

### `apply_patch(patch_type: String) -> Result<(), String>`
Downloads the patch zip from backend, extracts over game dir,
sets registry version to `1.0.1.1`.
- `patch_type`: `"full"` or `"p11"`
- Emits `download-progress` events during download
- Emits `extract-progress` events during extraction

## VC++ Redistributable

### `check_vcredist() -> bool`
Returns true if VC++ 2015-2022 x86 is installed.

### `install_vcredist() -> Result<(), String>`
Downloads vc_redist.x86.exe, runs `/install /quiet /norestart`.
Emits `download-progress` events during download.

## Hooks (wicgate DLLs)

### `check_hooks() -> HooksStatus`
```
HooksStatus { installed: bool, version: Option<String>, latest: String, needs_update: bool }
```

### `install_hooks() -> Result<(), String>`
Downloads hooks zip from backend, extracts to game directory.
Emits `download-progress` events.

## CD Key

### `get_cd_key() -> Option<String>`
Read CD key from registry.

### `set_cd_key(key: String) -> Result<(), String>`
Write CD key to registry.

## Maps

### `get_local_maps() -> Vec<LocalMap>`
List .sdf files in the game's maps directory with MD5 hashes.
```
LocalMap { name: String, hash: String, size: u64 }
```

### `download_map(filename: String) -> Result<(), String>`
Download a map from backend to the local maps directory.
Emits `download-progress` events with map name as identifier.

## Game

### `start_game() -> Result<(), String>`
Launch wic.exe from the install path.

## Events (Rust -> Frontend)

### `download-progress`
```json
{ "type": "download-patch|download-vcredist|download-hooks|download-map",
  "id": "<optional identifier>",
  "percentage": 0-100 }
```

### `extract-progress`
```json
{ "type": "extract-patch|extract-hooks",
  "percentage": 0-100 }
```
