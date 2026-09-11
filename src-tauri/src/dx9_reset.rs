//! Toggle DX9 reset in the verified 1.011 wic.exe and wic_online.exe serializers.
//! Recognition uses executable instructions, not installation registry entries.
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const RESET: [u8; 6] = [0xB9, 0, 0, 0, 0, 0x90]; // mov ecx, 0; nop
const REMEMBER: [u8; 6] = [0x8B, 0x8F, 0xFC, 0x02, 0, 0]; // mov ecx, [edi+0x2fc]
const BASE_BEFORE: [u8; 15] = [
    0x50, 0x8D, 0x44, 0x24, 0x74, 0x50, 0x8D, 0x4C, 0x24, 0x28, 0xE8, 0x22, 0x28, 0x06, 0x00,
];
const BASE_AFTER: [u8; 37] = [
    0x0F, 0xB7, 0x97, 0x00, 0x03, 0x00, 0x00, 0x83, 0xEC, 0x08, 0x8B, 0xC4, 0x89, 0x08, 0x66, 0x89,
    0x50, 0x04, 0x8D, 0x44, 0x24, 0x28, 0x50, 0xB9, 0x9C, 0x46, 0xD3, 0x00, 0x8D, 0x44, 0x24, 0x7C,
    0xE8, 0xB7, 0xE1, 0xA8, 0xFF,
];

struct Layout {
    site_rva: usize,
    text_size: usize,
    flag_rva: usize,
    filename_rva: usize,
    format_rva: usize,
    before: [u8; 15],
    after: [u8; 37],
}

const LAYOUTS: [Layout; 2] = [
    Layout {
        site_rva: 0x59A99E,
        text_size: 0x7EA08B,
        flag_rva: 0x93469C,
        filename_rva: 0x933F64,
        format_rva: 0x9347B0,
        before: BASE_BEFORE,
        after: BASE_AFTER,
    },
    Layout {
        site_rva: 0x55D8BE,
        text_size: 0x78C6EB,
        flag_rva: 0x8D5AF4,
        filename_rva: 0x8D53BC,
        format_rva: 0x8D5C08,
        before: [
            0x50, 0x8D, 0x44, 0x24, 0x74, 0x50, 0x8D, 0x4C, 0x24, 0x28, 0xE8, 0x12, 0x37, 0x06,
            0x00,
        ],
        after: [
            0x0F, 0xB7, 0x97, 0x00, 0x03, 0x00, 0x00, 0x83, 0xEC, 0x08, 0x8B, 0xC4, 0x89, 0x08,
            0x66, 0x89, 0x50, 0x04, 0x8D, 0x44, 0x24, 0x28, 0x50, 0xB9, 0xF4, 0x5A, 0xCD, 0x00,
            0x8D, 0x44, 0x24, 0x7C, 0xE8, 0xC7, 0x9E, 0xAC, 0xFF,
        ],
    },
];

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ResetState {
    pub available: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
struct Site {
    offset: usize,
    characteristics: usize,
    enabled: bool,
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<usize> {
    Some(u32::from_le_bytes(bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?) as usize)
}

/// Restrict the patch to the verified x86 1.011 layout and complete save sequence.
fn find_site_for(bytes: &[u8], layout: &Layout) -> Option<Site> {
    if bytes.get(..2)? != b"MZ" {
        return None;
    }
    let pe = u32_at(bytes, 0x3C)?;
    if bytes.get(pe..pe.checked_add(4)?)? != b"PE\0\0"
        || u16_at(bytes, pe.checked_add(4)?)? != 0x014C
    {
        return None;
    }
    let optional = pe.checked_add(24)?;
    let optional_size = u16_at(bytes, pe.checked_add(20)?)? as usize;
    if optional_size < 96
        || u16_at(bytes, optional)? != 0x010B
        || u32_at(bytes, optional.checked_add(28)?)? != 0x00400000
    {
        return None;
    }
    let sections = optional.checked_add(optional_size)?;
    let count = u16_at(bytes, pe.checked_add(6)?)? as usize;
    if !(1..=96).contains(&count) {
        return None;
    }
    bytes.get(sections..sections.checked_add(count.checked_mul(40)?)?)?;

    // Map RVAs through file-backed sections; refuse ambiguous or truncated mappings.
    let map = |rva: usize, length: usize, code: bool| -> Option<usize> {
        let mut found = None;
        for i in 0..count {
            let s = sections + i * 40;
            let va = u32_at(bytes, s + 12)?;
            let size = u32_at(bytes, s + 16)?;
            let raw = u32_at(bytes, s + 20)?;
            let Some(delta) = rva.checked_sub(va) else {
                continue;
            };
            if delta.checked_add(length)? > size {
                continue;
            }
            if code
                && (bytes.get(s..s + 8)? != b".text\0\0\0"
                    || va != 0x1000
                    || u32_at(bytes, s + 8)? != layout.text_size
                    || u32_at(bytes, s + 36)? & 0x20000000 == 0)
            {
                return None;
            }
            bytes.get(raw..raw.checked_add(size)?)?;
            let offset = raw.checked_add(delta)?;
            bytes.get(offset..offset.checked_add(length)?)?;
            if found.replace(offset).is_some() {
                return None;
            }
        }
        found
    };
    let start = map(
        layout.site_rva - layout.before.len(),
        layout.before.len() + 6 + layout.after.len(),
        true,
    )?;
    let offset = start + layout.before.len();
    if bytes.get(start..offset)? != layout.before
        || bytes.get(offset + 6..offset + 6 + layout.after.len())? != layout.after
    {
        return None;
    }
    for (rva, expected) in [
        (layout.flag_rva, b"myDX10Flag\0".as_slice()),
        (layout.filename_rva, b"Game Options.txt\0".as_slice()),
        (layout.format_rva, b"%s %d\r\n\0".as_slice()),
    ] {
        let at = map(rva, expected.len(), false)?;
        if bytes.get(at..at + expected.len())? != expected {
            return None;
        }
    }
    let instruction = bytes.get(offset..offset + 6)?;
    if instruction != RESET && instruction != REMEMBER {
        return None;
    }
    Some(Site {
        offset,
        characteristics: pe + 22,
        enabled: instruction == RESET,
    })
}

fn find_site(bytes: &[u8]) -> Option<Site> {
    let mut matches = LAYOUTS
        .iter()
        .filter_map(|layout| find_site_for(bytes, layout));
    let site = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(site)
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".wiclive-dx9-reset.bak");
    PathBuf::from(name)
}

/// Keep an original backup without clobbering a backup from another executable.
fn backup_matches(current: &[u8], backup: &[u8], site: Site) -> bool {
    let Some(original) = find_site(backup) else {
        return false;
    };
    if original.offset != site.offset || current.len() != backup.len() {
        return false;
    }
    // Other existing WIC LIVE toggles may change after this backup was created.
    let launcher = crate::core::find_launcher_flag_offset(current)
        .ok()
        .filter(|offset| crate::core::find_launcher_flag_offset(backup).ok() == Some(*offset));
    current.iter().zip(backup).enumerate().all(|(i, (a, b))| {
        if (site.offset..site.offset + 6).contains(&i) || launcher == Some(i) {
            return true;
        }
        if i == site.characteristics {
            return a & !0x20 == b & !0x20;
        }
        a == b
    })
}

fn state(site: Option<Site>) -> ResetState {
    ResetState {
        available: site.is_some(),
        enabled: site.is_some_and(|s| s.enabled),
    }
}

pub fn get_state(path: &Path) -> Result<ResetState, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    Ok(state(find_site(&bytes)))
}

fn ensure_backup(path: &Path, bytes: &[u8], site: Site) -> Result<(), String> {
    let backup = backup_path(path);
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(bytes).and_then(|_| file.sync_all()) {
                drop(file);
                let _ = std::fs::remove_file(&backup);
                return Err(format!(
                    "Could not save the original executable backup: {e}"
                ));
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let saved = std::fs::read(&backup).map_err(|e| e.to_string())?;
            if backup_matches(bytes, &saved, site) {
                Ok(())
            } else {
                Err("The existing DX9-reset backup does not match this executable.".into())
            }
        }
        Err(e) => Err(format!(
            "Could not create the original executable backup: {e}"
        )),
    }
}

fn write_instruction(file: &mut File, offset: usize, instruction: &[u8; 6]) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(offset as u64))?;
    file.write_all(instruction)?;
    file.sync_data()?;
    file.seek(SeekFrom::Start(offset as u64))?;
    let mut actual = [0; 6];
    file.read_exact(&mut actual)?;
    if actual != *instruction {
        return Err(std::io::Error::other(
            "Executable write verification failed",
        ));
    }
    Ok(())
}

pub fn set_enabled(path: &Path, enabled: bool) -> Result<ResetState, String> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0); // Refuse a running game or another writer.
    }
    let mut file = options.open(path).map_err(|e| {
        format!(
            "Could not update {}. Close World in Conflict before changing this option: {e}",
            path.display()
        )
    })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let site = find_site(&bytes)
        .ok_or("The DX9-reset setting is unavailable for this executable version.")?;
    if site.enabled == enabled {
        return Ok(state(Some(site)));
    }
    // Preserve the executable as it was before the first change in either direction.
    ensure_backup(path, &bytes, site)?;
    let original = if site.enabled { &RESET } else { &REMEMBER };
    let replacement = if enabled { &RESET } else { &REMEMBER };
    if let Err(error) = write_instruction(&mut file, site.offset, replacement) {
        if let Err(rollback) = write_instruction(&mut file, site.offset, original) {
            return Err(format!("DX9-reset update failed: {error}; restoring the original bytes also failed: {rollback}. Original backup: {}", backup_path(path).display()));
        }
        return Err(format!(
            "DX9-reset update failed; original bytes restored: {error}"
        ));
    }
    Ok(state(Some(Site { enabled, ..site })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "wiclive-dx9-reset-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn exe(&self) -> PathBuf {
            self.0.join("wic.exe")
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn put32(bytes: &mut [u8], offset: usize, value: usize) {
        bytes[offset..offset + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }

    // Minimal PE containers around the independently inspected instruction bytes.
    fn fixture(index: usize, enabled: bool) -> Vec<u8> {
        let layout = &LAYOUTS[index];
        let text_raw_size = (layout.text_size + 0xFFF) & !0xFFF;
        let rdata = 0x1000 + text_raw_size;
        let end = (layout.format_rva + 0x1000) & !0xFFF;
        let mut bytes = vec![0; end];
        bytes[..2].copy_from_slice(b"MZ");
        put32(&mut bytes, 0x3C, 0x80);
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        bytes[0x84..0x86].copy_from_slice(&0x14Cu16.to_le_bytes());
        bytes[0x86..0x88].copy_from_slice(&2u16.to_le_bytes());
        bytes[0x94..0x96].copy_from_slice(&0xE0u16.to_le_bytes());
        bytes[0x98..0x9A].copy_from_slice(&0x10Bu16.to_le_bytes());
        put32(&mut bytes, 0x98 + 28, 0x400000);
        for (s, name, va, size, raw_size, flags) in [
            (
                0x178,
                b".text\0\0\0",
                0x1000,
                layout.text_size,
                text_raw_size,
                0x60000020,
            ),
            (
                0x1A0,
                b".rdata\0\0",
                rdata,
                end - rdata,
                end - rdata,
                0x40000040,
            ),
        ] {
            bytes[s..s + 8].copy_from_slice(name);
            put32(&mut bytes, s + 8, size);
            put32(&mut bytes, s + 12, va);
            put32(&mut bytes, s + 16, raw_size);
            put32(&mut bytes, s + 20, va);
            put32(&mut bytes, s + 36, flags);
        }
        let at = layout.site_rva;
        bytes[at - 15..at].copy_from_slice(&layout.before);
        bytes[at..at + 6].copy_from_slice(if enabled { &RESET } else { &REMEMBER });
        bytes[at + 6..at + 43].copy_from_slice(&layout.after);
        for (rva, value) in [
            (layout.flag_rva, b"myDX10Flag\0".as_slice()),
            (layout.filename_rva, b"Game Options.txt\0".as_slice()),
            (layout.format_rva, b"%s %d\r\n\0".as_slice()),
        ] {
            bytes[rva..rva + value.len()].copy_from_slice(value);
        }
        bytes
    }

    fn assert_round_trip(original: &[u8], initially_enabled: bool) {
        let dir = TempDir::new();
        let path = dir.exe();
        std::fs::write(&path, original).unwrap();
        let site = find_site(original).unwrap();
        assert_eq!(
            get_state(&path).unwrap(),
            ResetState {
                available: true,
                enabled: initially_enabled
            }
        );
        // Reading and requesting the existing state must not create a backup or change bytes.
        set_enabled(&path, initially_enabled).unwrap();
        assert!(!backup_path(&path).exists());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(
            set_enabled(&path, !initially_enabled).unwrap().enabled,
            !initially_enabled
        );
        assert_eq!(get_state(&path).unwrap().enabled, !initially_enabled);
        let mut expected = original.to_vec();
        expected[site.offset..site.offset + 6].copy_from_slice(if initially_enabled {
            &REMEMBER
        } else {
            &RESET
        });
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert_eq!(std::fs::read(backup_path(&path)).unwrap(), original);
        set_enabled(&path, !initially_enabled).unwrap();
        set_enabled(&path, initially_enabled).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(std::fs::read(backup_path(&path)).unwrap(), original);
    }

    #[test]
    fn supports_both_executables_and_both_initial_states_without_brand_detection() {
        for index in 0..LAYOUTS.len() {
            for enabled in [true, false] {
                assert_round_trip(&fixture(index, enabled), enabled);
            }
        }
    }

    #[test]
    fn rejects_malformed_headers_and_mismatching_save_code_without_writing() {
        let original = fixture(0, true);
        for offset in [
            0,
            0x3C,
            0x80,
            0x84,
            0x86,
            0x94,
            0x98,
            0xB4,
            0x178,
            0x180,
            0x19F,
            LAYOUTS[0].site_rva - 1,
            LAYOUTS[0].site_rva + 2,
            LAYOUTS[0].site_rva + 6,
            LAYOUTS[0].flag_rva,
            LAYOUTS[0].filename_rva,
            LAYOUTS[0].format_rva,
        ] {
            let mut bytes = original.clone();
            bytes[offset] ^= 0xFF;
            let dir = TempDir::new();
            let path = dir.exe();
            std::fs::write(&path, &bytes).unwrap();
            assert!(!get_state(&path).unwrap().available, "offset {offset:x}");
            assert!(set_enabled(&path, false).is_err(), "offset {offset:x}");
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert!(!backup_path(&path).exists());
        }
        for length in [
            0,
            1,
            0x3F,
            0x82,
            0x196,
            LAYOUTS[0].site_rva + 5,
            original.len() - 1,
        ] {
            assert!(find_site(&original[..length]).is_none());
        }
    }

    #[test]
    fn rejects_overlapping_section_mappings() {
        let mut bytes = fixture(0, true);
        bytes[0x86..0x88].copy_from_slice(&3u16.to_le_bytes());
        let section = bytes[0x178..0x1A0].to_vec();
        bytes[0x1C8..0x1F0].copy_from_slice(&section);
        assert!(find_site(&bytes).is_none());
    }

    #[test]
    fn stale_or_unwritable_backup_prevents_any_executable_change() {
        let bytes = fixture(0, true);
        let dir = TempDir::new();
        let path = dir.exe();
        std::fs::write(&path, &bytes).unwrap();
        std::fs::write(backup_path(&path), b"unrelated backup").unwrap();
        assert!(set_enabled(&path, false).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            std::fs::read(backup_path(&path)).unwrap(),
            b"unrelated backup"
        );
        std::fs::remove_file(backup_path(&path)).unwrap();
        std::fs::create_dir(backup_path(&path)).unwrap();
        assert!(set_enabled(&path, false).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn preserves_laa_and_welcome_launcher_changes_after_backup() {
        let mut bytes = fixture(0, true);
        // Same independent branch fixture used by the welcome-launcher tests.
        let mut launcher = [0u8; 42];
        launcher[0..2].copy_from_slice(&[0x39, 0x1D]);
        launcher[6..9].copy_from_slice(&[0x75, 0x10, 0xE8]);
        launcher[13..20].copy_from_slice(&[0x83, 0xF8, 0x01, 0x75, 0x06, 0x89, 0x3D]);
        launcher[24] = 0xE8;
        launcher[29] = 0xE8;
        launcher[34..36].copy_from_slice(&[0x39, 0x1D]);
        launcher[40..42].copy_from_slice(&[0x75, 0x39]);
        bytes[0x10000..0x1002A].copy_from_slice(&launcher);
        let dir = TempDir::new();
        let path = dir.exe();
        std::fs::write(&path, &bytes).unwrap();
        set_enabled(&path, false).unwrap();
        crate::core::apply_laa(path.to_str().unwrap()).unwrap();
        crate::core::set_skip_launcher(dir.0.to_str().unwrap(), true).unwrap();
        set_enabled(&path, true).unwrap();
        bytes[0x96] |= 0x20;
        bytes[0x10006] = 0xEB;
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[cfg(windows)]
    #[test]
    fn refuses_locked_executable_without_writing_or_creating_backup() {
        use std::os::windows::fs::OpenOptionsExt;
        let bytes = fixture(0, true);
        let dir = TempDir::new();
        let path = dir.exe();
        std::fs::write(&path, &bytes).unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert!(set_enabled(&path, false).is_err());
        drop(lock);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!backup_path(&path).exists());
    }

    #[test]
    #[ignore = "Set WICLIVE_DX9_RESET_FIXTURES to a JSON manifest of local game executables"]
    fn supplied_executables_round_trip_on_temporary_copies() {
        #[derive(serde::Deserialize)]
        struct Fixture {
            path: PathBuf,
            available: bool,
            enabled: bool,
        }
        let manifest =
            std::env::var_os("WICLIVE_DX9_RESET_FIXTURES").expect("fixture manifest required");
        let fixtures: Vec<Fixture> =
            serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
        assert!(!fixtures.is_empty());
        for fixture in fixtures {
            let bytes = std::fs::read(&fixture.path).unwrap();
            let actual = state(find_site(&bytes));
            assert_eq!(
                actual,
                ResetState {
                    available: fixture.available,
                    enabled: fixture.enabled
                },
                "{}",
                fixture.path.display()
            );
            if fixture.available {
                assert_round_trip(&bytes, fixture.enabled);
            } else {
                let dir = TempDir::new();
                let path = dir.exe();
                std::fs::write(&path, &bytes).unwrap();
                assert!(set_enabled(&path, true).is_err());
                assert!(set_enabled(&path, false).is_err());
                assert_eq!(std::fs::read(&path).unwrap(), bytes);
                assert!(!backup_path(&path).exists());
            }
            // Never modify supplied binaries, even during integration validation.
            assert_eq!(std::fs::read(&fixture.path).unwrap(), bytes);
        }
    }
}
