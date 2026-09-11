"""Regression: valid wide hmtx advances must not wrap negative.

Derive temporary fonts from the bundled OFL fixture and recompute SFNT
checksums. Exercise actual shaping, not only the integer scale helper.
"""
import json
from pathlib import Path
import struct
import subprocess
import tempfile

module = Path(__file__).resolve().parents[1]
source = module / 'tests/assets/NuxieJapaneseFixture-Regular.otf'
probe = str(module.parents[1] / 'target/debug/examples/outline_probe')
glyph = json.loads(subprocess.check_output([probe, str(source), '／']))[0]['glyph']
original = source.read_bytes()
tables = {}
for index in range(struct.unpack_from('>H', original, 4)[0]):
    position = 12 + 16 * index
    tag = original[position:position + 4].decode()
    offset, length = struct.unpack_from('>II', original, position + 8)
    tables[tag] = (position, offset, length)
head = tables['head'][1]
upem = struct.unpack_from('>H', original, head + 18)[0]
hhea = tables['hhea'][1]
count = struct.unpack_from('>H', original, hhea + 34)[0]
assert glyph < count and upem * 2 < 65536


def checksum(data):
    data = bytes(data)
    data += b'\0' * ((-len(data)) % 4)
    return sum(struct.unpack('>' + str(len(data) // 4) + 'I', data)) & 0xffffffff


results = []
with tempfile.TemporaryDirectory() as directory:
    for advance_units in [upem * 2, 65535]:
        buf = bytearray(original)
        struct.pack_into('>H', buf, tables['hmtx'][1] + glyph * 4, advance_units)
        struct.pack_into('>H', buf, hhea + 10, max(
            advance_units, struct.unpack_from('>H', buf, hhea + 10)[0]))
        struct.pack_into('>I', buf, head + 8, 0)
        for tag in ['head', 'hhea', 'hmtx']:
            position, offset, length = tables[tag]
            struct.pack_into('>I', buf, position + 4, checksum(buf[offset:offset + length]))
        struct.pack_into('>I', buf, head + 8, (0xB1B0AFBA - checksum(buf)) & 0xffffffff)
        assert checksum(buf) == 0xB1B0AFBA
        output = Path(directory) / 'wide-advance.otf'
        output.write_bytes(buf)
        for size in [24, 16384, 32768, 1000000]:
            run = subprocess.run([probe, str(output), '／', str(size), 'css'],
                                 capture_output=True, text=True, check=True)
            advance = json.loads(run.stdout)[0]['advance']
            expected = size * advance_units / upem
            result = {'advanceUnits': advance_units, 'size': size,
                      'expected': expected, 'advance': advance}
            results.append(result)
            assert abs(advance - expected) <= expected * 1e-6, result
print(json.dumps(results, indent=2))
