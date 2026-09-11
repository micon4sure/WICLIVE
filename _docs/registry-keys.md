# WIC LIVE v4 - Windows Registry Keys

## Game Installation

Detection checks the following values in order and uses the first absolute directory containing a file named `wic.exe` or `wic_online.exe`. Missing files or invalid paths do not prevent checking later values. GOG Galaxy does not need to be running or installed; detection reads the game's registry entries.

### Install Path
- **Key**: `HKLM\SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict`
- **Value**: `InstallPath` (REG_SZ)
- **Example**: `C:\Program Files (x86)\Sierra Entertainment\World in Conflict\`

### GOG Variant (fallback)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\1438332414`
- **Values**: `WORKINGDIR`, then `path` (REG_SZ)

The registered-install check uses the same values even when game files are missing. Registry cleanup removes these install-path values, including the GOG `path` fallback.

### Game Version (read/write)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict`
- **Value**: `Version` (REG_SZ)
- **Values observed**:
  - `1.0.0.0` - unpatched (v1.0)
  - `1.0.1.0` - patch 10 applied (v1.010)
  - `1.0.1.1` - patch 11 applied (v1.011) - target version

## CD Key

### CD Key (read/write)
- **Key**: `HKCU\Software\Massive Entertainment AB\World In Conflict`
- **Value**: `CDKEY` (REG_SZ)

Readiness validates the key's checksum and installed edition before accepting it.
See [CD-key validation](cd-key-validation.md) for the rules and reference tests.

## VC++ Redistributable

### VC++ 2015-2022 x86 (read only)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X86`
- **Presence of key** indicates installed
- **Value**: `Installed` (REG_DWORD) = 1
