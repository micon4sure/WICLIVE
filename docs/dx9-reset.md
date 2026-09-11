# DirectX preference reset

Config's **Suppress DX10** switch controls how the selected game executable
saves `myDX10Flag` in `Game Options.txt`. It does not select the renderer for the
running game or edit an existing options file. With reset enabled, the saver
writes zero; with reset disabled, it writes the game's current preference.

WIC LIVE reads the existing state without changing it. The inspected GOG
`wic.exe` therefore starts enabled. Ordinary 1.011 copies and the inspected GOG
`wic_online.exe` start disabled. Either state can be selected on a recognized
executable, independently of the installed proxy. Unknown save routines are
unavailable and remain untouched. The command uses the same executable as launch:
`wic.exe`, or `wic_online.exe` if `wic.exe` is absent.

## Recognition and writes

The verified x86 PE32 image base is `0x400000`. Recognition validates the PE
headers and section mapping, `.text` size, 15 instruction bytes before the site,
37 bytes after it, and the referenced `myDX10Flag`, `Game Options.txt`, and
`%s %d\r\n` strings. It requires one unambiguous recognized site.

| 1.011 executable | Site RVA | `.text` virtual size |
| --- | --- | --- |
| `wic.exe` | `0x59A99E` | `0x7EA08B` |
| `wic_online.exe` | `0x55D8BE` | `0x78C6EB` |

Only these six bytes change:

- Reset enabled: `B9 00 00 00 00 90` (`mov ecx, 0; nop`).
- Reset disabled: `8B 8F FC 02 00 00` (`mov ecx, [edi+0x2fc]`).

Writes require exclusive access to the executable; close the game first. A full
original backup is created once at `<exe>.wiclive-dx9-reset.bak`. An existing
backup must match the executable, allowing only this setting, the LAA bit, and
the separately validated welcome-launcher flag to differ. The switch patches
only its six bytes when restoring the setting, preserving those other flags.
Writes are flushed and read back; failure triggers an attempt to restore the
previous bytes. Existing backups are never overwritten.

## Validation on Windows

Run the normal suite:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
bun test
bun run build
```

For real executable coverage, create a JSON manifest containing entries such as
`{"path":"C:\\Games\\World in Conflict\\wic.exe","available":true,"enabled":false}`.
Set `WICLIVE_DX9_RESET_FIXTURES` to its path and run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib supplied_executables_round_trip_on_temporary_copies -- --ignored
```

This test reads supplied binaries, toggles temporary copies in both directions,
checks that only the six expected bytes change, and verifies the original files
remain unchanged. Include GOG and ordinary 1.011 executables plus unsupported
older versions (`available: false`, `enabled: false`). Do not commit game binaries.
