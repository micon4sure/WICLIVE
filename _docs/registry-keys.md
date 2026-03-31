# WIC LIVE v4 - Windows Registry Keys

## Game Installation

### Install Path (read only)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict`
- **Value**: `InstallPath` (REG_SZ)
- **Example**: `C:\Program Files (x86)\Sierra Entertainment\World in Conflict\`

### GOG Variant (read only, fallback)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\1438332414`
- **Value**: `WORKINGDIR` (REG_SZ)

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

## VC++ Redistributable

### VC++ 2015-2022 x86 (read only)
- **Key**: `HKLM\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X86`
- **Presence of key** indicates installed
- **Value**: `Installed` (REG_DWORD) = 1
