//! Local key checksum and edition checks, following Massgate's key format:
//! https://github.com/ubisoft/massgate/blob/master/src/MMassgate/MMG_CdKeyValidator.cpp
//! https://github.com/ubisoft/massgate/blob/master/src/MMassgate/MMG_CdKeyChecker.cpp
//!
//! Server registration, bans and time limits are not checked here.

use tiger::{Digest, Tiger};

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPRSTUVWXY23456789";
const CHECKSUM_SHIFT: u32 = 35;
const CHECKSUM_MASK: u128 = 0x3ff;

#[derive(Debug, PartialEq, Eq)]
pub enum CdKeyError {
    Missing,
    Invalid,
    WrongEdition,
}

impl std::fmt::Display for CdKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Missing => "Not set",
            Self::Invalid => "Invalid CD key",
            Self::WrongEdition => "Wrong game edition",
        })
    }
}

/// Check the key without changing it or contacting Massgate.
pub fn validate(key: &str, has_soviet_assault: bool) -> Result<(), CdKeyError> {
    if key.is_empty() {
        return Err(CdKeyError::Missing);
    }

    let product = decode_product(key).ok_or(CdKeyError::Invalid)?;
    match product {
        // Match the game's standard and time-limited product families.
        1 | 2 if !has_soviet_assault => Ok(()),
        3 | 4 if has_soviet_assault => Ok(()),
        1..=4 => Err(CdKeyError::WrongEdition),
        _ => Err(CdKeyError::Invalid),
    }
}

fn decode_product(key: &str) -> Option<u8> {
    // The game reads at most 26 input bytes. A key encodes 100 bits in 20 symbols.
    if !(20..=26).contains(&key.len()) {
        return None;
    }

    let mut bits = 0u128;
    let mut count = 0;
    for byte in key.bytes().filter(|&byte| byte != b'-') {
        let symbol = match byte.to_ascii_uppercase() {
            b'0' => b'O',
            b'1' => b'I',
            other => other,
        };
        let value = ALPHABET.iter().position(|&candidate| candidate == symbol)?;
        if count == 20 {
            return None;
        }
        bits |= (value as u128) << (count * 5);
        count += 1;
    }
    if count != 20 {
        return None;
    }

    let mut data = bits.to_le_bytes();
    // Bits 96..99 supply the repeated nibble used to scramble bytes 2..10.
    let scramble = (data[12] & 0x0f) * 0x11;
    for byte in &mut data[2..11] {
        *byte ^= scramble;
    }

    let decoded = u128::from_le_bytes(data);
    let checksum = (decoded >> CHECKSUM_SHIFT) & CHECKSUM_MASK;
    // The original 10-bit field truncates the checksum seed 0x5244 to 0x244.
    let seeded = (decoded & !(CHECKSUM_MASK << CHECKSUM_SHIFT))
        | ((0x5244 & CHECKSUM_MASK) << CHECKSUM_SHIFT);
    let hash = Tiger::digest(seeded.to_le_bytes());
    let computed = u16::from_le_bytes([hash[0], hash[1]]) as u128 & CHECKSUM_MASK;
    (checksum == computed).then_some((decoded & 7) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn massgate_reference() -> serde_json::Value {
        serde_json::from_str(include_str!("../tests/fixtures/cdkey-reference.json")).unwrap()
    }

    fn hex_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn tiger_matches_massgate_full_digests_including_key_payloads_and_padding_boundaries() {
        let reference = massgate_reference();
        for vector in reference["tiger"].as_array().unwrap() {
            let input = vector["input"].as_str().unwrap();
            let expected = hex_bytes(vector["hash"].as_str().unwrap());
            let actual = Tiger::digest(hex_bytes(input));
            assert_eq!(
                &actual[..],
                expected,
                "Tiger differs from Massgate for {input}"
            );
        }
    }

    #[test]
    fn decoder_matches_massgate_for_keys_and_every_single_symbol_mutation() {
        let reference = massgate_reference();
        for vector in reference["keys"].as_array().unwrap() {
            let key = vector["key"].as_str().unwrap();
            let product = vector["product"].as_u64().unwrap() as u8;
            assert_eq!(decode_product(key), Some(product), "{key}");
            assert_eq!(
                validate(key, false).is_ok(),
                vector["base"].as_bool().unwrap(),
                "{key}"
            );
            assert_eq!(
                validate(key, true).is_ok(),
                vector["soviet"].as_bool().unwrap(),
                "{key}"
            );

            let raw = key.replace('-', "").into_bytes();
            let accepted = vector["valid_mutations"].as_array().unwrap();
            for index in 0..raw.len() {
                for &symbol in ALPHABET {
                    if symbol == raw[index] {
                        continue;
                    }
                    let mut changed = raw.clone();
                    changed[index] = symbol;
                    let changed = String::from_utf8(changed).unwrap();
                    let expected = accepted
                        .iter()
                        .find(|entry| entry["key"].as_str() == Some(&changed))
                        .map(|entry| entry["product"].as_u64().unwrap() as u8);
                    assert_eq!(
                        decode_product(&changed),
                        expected,
                        "Massgate differs for {changed}"
                    );
                }
            }
        }
    }

    // Synthetic fixtures from the independent WiCGate WASM generator/decoder.
    // seq=123456, batch=17, seed=12345; never registered with a server.
    const BASE: &str = "JEAT-2DA5-G4D7-4YGB-KJ3A";
    const BASE_TIMED: &str = "KEAT-2DA7-V4D7-4YGB-KJ3A";
    const SOVIET: &str = "LEAT-2DAP-S4D7-4YGB-KJ3A";
    const SOVIET_TIMED: &str = "MEAT-2DA3-C4D7-4YGB-KJ3A";

    #[test]
    fn accepts_matching_editions_and_rejects_both_mismatch_directions() {
        for key in [BASE, BASE_TIMED] {
            assert_eq!(validate(key, false), Ok(()));
            assert_eq!(validate(key, true), Err(CdKeyError::WrongEdition));
        }
        for key in [SOVIET, SOVIET_TIMED] {
            assert_eq!(validate(key, true), Ok(()));
            assert_eq!(validate(key, false), Err(CdKeyError::WrongEdition));
        }
    }

    #[test]
    fn checks_keys_with_nonzero_scrambling_and_maximum_sequence_and_batch() {
        // seq=33554431, batch=127, seed=0xffffffff.
        assert_eq!(validate("3993-MGUU-GGM8-EG53-VB9H", false), Ok(()));
        assert_eq!(validate("5993-MGUR-RGM8-EG53-VB9H", true), Ok(()));
    }

    #[test]
    fn preserves_the_games_case_separator_and_zero_one_alias_rules() {
        assert_eq!(validate(&BASE.to_ascii_lowercase(), false), Ok(()));
        assert_eq!(validate(&SOVIET.replace('-', ""), true), Ok(()));
        // Valid product-0/6 fixtures contain I/O; alias handling precedes edition checks.
        assert_eq!(decode_product("1EAT-2DAX-34D7-4YGB-KJ3A"), Some(0));
        assert_eq!(decode_product("0EAT-2DA5-24D7-4YGB-KJ3A"), Some(6));
    }

    #[test]
    fn rejects_unsupported_products_even_with_a_valid_checksum() {
        for key in [
            "IEAT-2DAX-34D7-4YGB-KJ3A",
            "NEAT-2DAW-L4D7-4YGB-KJ3A",
            "OEAT-2DA5-24D7-4YGB-KJ3A",
            "PEAT-2DA3-34D7-4YGB-KJ3A",
        ] {
            for soviet in [false, true] {
                assert_eq!(validate(key, soviet), Err(CdKeyError::Invalid));
            }
        }
    }

    #[test]
    fn rejects_corrupt_checksum_before_reporting_an_edition_mismatch() {
        let corrupt = "JEAT-2DA5-H4D7-4YGB-KJ3A";
        assert_eq!(validate(corrupt, false), Err(CdKeyError::Invalid));
        assert_eq!(validate(corrupt, true), Err(CdKeyError::Invalid));
    }

    #[test]
    fn rejects_missing_malformed_truncated_and_extended_keys() {
        assert_eq!(validate("", false), Err(CdKeyError::Missing));
        for key in [
            "invalid",
            " ",
            "AAAA-AAAA-AAAA-AAAA-AAAA",
            "JEAT-2DA5-G4D7-4YGB-KJ3",
            "JEAT-2DA5-G4D7-4YGB-KJ3AA",
            "JEAT-2DA5-G4D7-4YGB-KJ3Q",
            "JEAT-2DA5-G4D7-4YGB-KJ3Z",
            "JEAT-2DA5-G4D7-4YGB-KJ3é",
            "JEAT-2DA5-G4D7-4YGB-KJ3A\0",
            " JEAT-2DA5-G4D7-4YGB-KJ3A",
            "JEAT-2DA5-G4D7-4YGB-KJ3A ",
            "JEAT-2DA5-G4D7-4YGB-KJ3A---",
        ] {
            assert_eq!(validate(key, false), Err(CdKeyError::Invalid), "{key:?}");
        }
    }
}
