"""Regenerate test fixtures using an existing Ubisoft/Massgate source checkout.

Requires Python 3 and g++. Runs only local C++ code; no game registry or server access.
The hash, bitstream, key validator and edition check are compiled from that checkout.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


ALPHABET = "ABCDEFGHIJKLMNOPRSTUVWXY23456789"
KEYS = [
    "IEAT-2DAX-34D7-4YGB-KJ3A",  # product 0
    "JEAT-2DA5-G4D7-4YGB-KJ3A",  # product 1
    "KEAT-2DA7-V4D7-4YGB-KJ3A",  # product 2
    "LEAT-2DAP-S4D7-4YGB-KJ3A",  # product 3
    "MEAT-2DA3-C4D7-4YGB-KJ3A",  # product 4
    "NEAT-2DAW-L4D7-4YGB-KJ3A",  # product 5
    "OEAT-2DA5-24D7-4YGB-KJ3A",  # product 6
    "PEAT-2DA3-34D7-4YGB-KJ3A",  # product 7
    "3993-MGUU-GGM8-EG53-VB9H",  # maximum sequence/batch, product 1
    "5993-MGUR-RGM8-EG53-VB9H",  # maximum sequence/batch, product 3
]


def between(source, start, end):
    return source[source.index(start):source.index(end)]


def make_reference(sources):
    # Only scaffolding for unrelated engine/platform types is substituted.
    # The functions under comparison and their lookup tables are unmodified.
    return r"""
#include <cassert>
#include <cctype>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>
#define __int64 long long
#define _toupper std::toupper
#define MC_PROFILER_BEGIN(...)
template<bool B> struct ct_assert { static_assert(B, "assertion failed"); };
template<int N> using MC_StaticString = std::string;
typedef unsigned long long word64;
""" + between(sources["MMG_Tiger.cpp"], "#undef OPTIMIZE_FOR_ALPHA", "// MMG_Tiger implementation below") + r"""
class MMG_CryptoHash {
public:
    MMG_CryptoHash(const void* data, size_t length, int) : myHashLength(length) {
        memcpy(myHash, data, length);
    }
    uint32_t Get32BitSubset() const;
private:
    alignas(uint32_t) unsigned char myHash[24];
    size_t myHashLength;
};
class MMG_Tiger {
public:
    MMG_CryptoHash GenerateHash(const void*, unsigned long) const;
    int GetIdentifier() const { return 0; }
};
""" + between(sources["MMG_CryptoHash.cpp"], "uint32_t\nMMG_CryptoHash::Get32BitSubset", "unsigned long long \nMMG_CryptoHash::Get64BitSubset") + between(
        sources["MMG_Tiger.cpp"], "MMG_CryptoHash \nMMG_Tiger::GenerateHash", "void\nMMG_Tiger::Start"
    ) + sources["MMG_CdKeyValidator.h"] + sources["MMG_BitStream.h"] + between(
        sources["MMG_CdKeyValidator.cpp"], "static const char padding1[]", "MMG_AccessCode::Validator::Validator()"
    ) + between(sources["MMG_Constants.h"], "typedef enum", "} ProductId;") + "} ProductId;\n" + r"""
class MMG_CdKeyChecker {
public:
    static bool ValidateProductId(MMG_CdKey::Validator&, bool);
};
""" + between(sources["MMG_CdKeyChecker.cpp"], "bool MMG_CdKeyChecker::ValidateProductId", "MMG_CdKeyChecker::MMG_CdKeyChecker(") + r"""
static_assert(sizeof(MMG_CdKey::KeyDefinition) == 16, "key layout differs");
int main() {
    const uint32_t endian = 1;
    assert(*reinterpret_cast<const unsigned char*>(&endian) == 1);
    std::string line;
    while (std::getline(std::cin, line)) {
        const auto input = line.substr(2);
        if (line[0] == 'H') {
            const auto size = input.size() / 2;
            std::vector<word64> words((size + 7) / 8 + 1, 0);
            auto bytes = reinterpret_cast<unsigned char*>(words.data());
            for (size_t i = 0; i < size; ++i)
                bytes[i] = std::stoul(input.substr(2 * i, 2), nullptr, 16);
            word64 result[3];
            tiger(words.data(), size, result);
            for (auto byte : reinterpret_cast<unsigned char(&)[24]>(result))
                std::cout << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
            std::cout << std::dec << '\n';
        } else {
            MMG_CdKey::Validator key;
            key.SetKey(input.c_str());
            std::cout << key.IsKeyValid() << ' '
                << (key.IsKeyValid() ? unsigned(key.GetProductIdentifier()) : 0) << ' '
                << MMG_CdKeyChecker::ValidateProductId(key, false) << ' '
                << MMG_CdKeyChecker::ValidateProductId(key, true) << '\n';
        }
    }
}
"""


def mutations(key):
    raw = key.replace("-", "")
    for index, original in enumerate(raw):
        for symbol in ALPHABET:
            if symbol != original:
                yield raw[:index] + symbol + raw[index + 1:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("massgate", type=Path, help="Massgate checkout containing src/MMassgate")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = args.massgate / "src" / "MMassgate"
    names = ["MMG_Tiger.cpp", "MMG_CryptoHash.cpp", "MMG_CdKeyValidator.h",
             "MMG_CdKeyValidator.cpp", "MMG_BitStream.h", "MMG_CdKeyChecker.cpp", "MMG_Constants.h"]
    sources = {name: (root / name).read_text() for name in names}

    hashes = [b"", b"abc"]
    hashes += [bytes(range(length)) for length in [16, 55, 56, 63, 64, 65, 127, 128]]
    for key in KEYS:
        bits = sum(ALPHABET.index(symbol) << (5 * i)
                   for i, symbol in enumerate(key.replace("-", "")))
        data = bytearray(bits.to_bytes(16, "little"))
        scramble = (data[12] & 15) * 17
        for index in range(2, 11):
            data[index] ^= scramble
        bits = int.from_bytes(data, "little")
        bits = (bits & ~(0x3ff << 35)) | (0x244 << 35)
        hashes.append(bits.to_bytes(16, "little"))

    # Include every single-symbol mutation of every synthetic key. A checksum
    # collision is possible, so preserve the reference's accepted exceptions.
    key_inputs = []
    for key in KEYS:
        key_inputs += [key, *mutations(key)]
    commands = ["H " + data.hex() for data in hashes] + ["K " + key for key in key_inputs]
    with tempfile.TemporaryDirectory(prefix="wiclive-cdkey-reference-") as temporary:
        source = Path(temporary) / "reference.cpp"
        binary = Path(temporary) / "reference"
        source.write_text(make_reference(sources))
        subprocess.run(["g++", "-std=c++17", "-O2", "-fno-strict-aliasing", "-mms-bitfields",
                        str(source), "-o", str(binary)], check=True)
        output = subprocess.run([str(binary)], input="\n".join(commands) + "\n",
                                text=True, capture_output=True, check=True).stdout.splitlines()
    assert len(output) == len(commands)

    fixtures = {
        "source_sha256": {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in names},
        "tiger": [{"input": data.hex(), "hash": digest} for data, digest in zip(hashes, output)],
        "keys": [],
    }
    rows = iter(output[len(hashes):])
    for key in KEYS:
        valid, product, base, soviet = map(int, next(rows).split())
        assert valid, key
        accepted_mutations = []
        for mutation in mutations(key):
            mutation_valid, mutation_product, _, _ = map(int, next(rows).split())
            if mutation_valid:
                accepted_mutations.append({"key": mutation, "product": mutation_product})
        fixtures["keys"].append({"key": key, "product": product, "base": bool(base),
                                 "soviet": bool(soviet), "valid_mutations": accepted_mutations})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixtures, indent=2) + "\n")
    print(f"Recorded {len(hashes)} full Tiger digests and {len(key_inputs)} key checks from Massgate C++.")


if __name__ == "__main__":
    main()
