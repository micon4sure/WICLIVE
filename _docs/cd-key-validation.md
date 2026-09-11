# CD-key readiness

WIC LIVE checks a stored CD key locally before marking it ready. The check decodes
the game's 20-symbol key, verifies its Tiger checksum, and matches its product ID
to the installed edition. Edition detection uses `assault.dat`, the same check
used when requesting replacement keys:

| Installation | Accepted key products | Requested replacement product |
| --- | --- | --- |
| World in Conflict / multiplayer only | 1 (standard), 2 (time-limited) | 1 |
| Soviet Assault / Complete Edition | 3 (standard), 4 (time-limited) | 3 |

Readiness reports `Not set`, `Invalid CD key`, or `Wrong game edition` for failed
checks. The existing Fix action requests a replacement for the detected edition.
The `set_cd_key` Tauri command validates nonempty replacements before writing them;
an empty value still clears the key. Key-service responses are validated before
being returned to the UI, including rejecting an empty response. A previous
successful repair cannot bypass a subsequent failed CD-key check.

These are local format, checksum and edition checks. They do not establish server
registration, ban status, time-limit validity, or whether an account can use a key.
Readiness does not contact a server or change the stored key.

## Reference tests

The implementation is in [cdkey.rs](../src-tauri/src/cdkey.rs). It uses RustCrypto's
original Tiger algorithm, with the game's little-endian digest layout and 10-bit
checksum field. It does not use Tiger2.

[Reference fixtures](../src-tauri/tests/fixtures/cdkey-reference.json) were produced
by compiling the Massgate C++ Tiger, bitstream, key validator and edition-check
functions. They are independent of the WIC LIVE Rust implementation. Source-file
SHA-256 hashes are stored with the fixtures. All fixture keys are synthetic and
were generated offline; none was registered with a server.

The Rust tests compare:

- All 24 digest bytes for 20 inputs: empty/short messages, block and padding
  boundaries, and the seeded 16-byte payloads used for key checksums.
- Ten keys covering product IDs 0–7, both editions, time-limited products, maximum
  sequence/batch values, and nonzero scrambling.
- All 6,200 single-symbol mutations of those keys. The fixtures preserve the
  reference validator's checksum collisions instead of assuming every mutation
  must be invalid.
- Malformed input, length limits, case, separators and the game's `0`/`O` and
  `1`/`I` aliases.

Run the native suite on Windows:

```sh
cd src-tauri
cargo test --locked --lib
```

Run the frontend readiness tests from the repository root with `bun test`.
They mock Tauri and check that invalid/mismatched keys block launch, remain blocked
after a previous repair, and become ready after a compatible replacement.

To regenerate the reference fixtures on a little-endian host with Python 3 and
g++, supply a local Massgate checkout:

```sh
python3 tests/generate-cdkey-reference.py /path/to/massgate src-tauri/tests/fixtures/cdkey-reference.json
```

The generator compiles the reference functions with Windows-compatible bitfield
layout and verifies the 16-byte key structure. Only unrelated engine/platform
types are stubbed. No registry, game installation or server is accessed.

Upstream format and edition rules:
[MMG_CdKeyValidator.cpp](https://github.com/ubisoft/massgate/blob/master/src/MMassgate/MMG_CdKeyValidator.cpp),
[MMG_CdKeyChecker.cpp](https://github.com/ubisoft/massgate/blob/master/src/MMassgate/MMG_CdKeyChecker.cpp).
