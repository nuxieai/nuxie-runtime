#!/usr/bin/env python3
"""Experimental static glyf font transformation; no browser or runtime dependency.

Requires isolated fonttools==4.59.1 and freetype-py==2.5.1. Native normal
FreeType hinting is a candidate, not an assertion about Chrome's rasterizer.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import platform

import fontTools
import freetype
from fontTools.misc.roundTools import otRound
from fontTools.ttLib import TTFont
from fontTools.ttLib.tables._g_l_y_f import Glyph, GlyphCoordinates
from fontTools.ttLib.tables.ttProgram import Program


def sha(data):
    return hashlib.sha256(data).hexdigest()


def transform(source, mode, size):
    font = TTFont(io.BytesIO(source), recalcTimestamp=False, recalcBBoxes=mode != 'identity')
    if 'glyf' not in font or any(t in font for t in ('fvar', 'CFF ', 'CFF2', 'COLR', 'SVG ')):
        raise ValueError('Only static monochrome glyf fonts are accepted by this experiment')
    if mode != 'identity':
        face = freetype.Face(io.BytesIO(source))
        face.set_pixel_sizes(0, size)
        flags = freetype.FT_LOAD_NO_BITMAP | freetype.FT_LOAD_NO_AUTOHINT
        if mode == 'unhinted':
            flags |= freetype.FT_LOAD_NO_SCALE | freetype.FT_LOAD_NO_HINTING
            scale = 1
        else:
            flags |= freetype.FT_LOAD_TARGET_NORMAL
            scale = font['head'].unitsPerEm / (size * 64)
        for index, name in enumerate(font.getGlyphOrder()):
            face.load_glyph(index, flags)
            outline = face.glyph.outline
            if any(tag & 3 not in (0, 1) for tag in outline.tags):
                raise ValueError('Unexpected cubic outline')
            glyph = Glyph()
            glyph.numberOfContours = len(outline.contours)
            glyph.coordinates = GlyphCoordinates([(otRound(x * scale), otRound(y * scale)) for x, y in outline.points])
            glyph.endPtsOfContours = list(outline.contours)
            glyph.flags = bytearray(tag & 1 for tag in outline.tags)
            glyph.program = Program()
            glyph.program.fromBytecode(b'')
            glyph.recalcBounds(font['glyf'])
            font['glyf'][name] = glyph
        for table in ('fpgm', 'prep', 'cvt '):
            if table in font:
                del font[table]
        for field in ('maxZones', 'maxTwilightPoints', 'maxStorage', 'maxFunctionDefs', 'maxInstructionDefs', 'maxStackElements', 'maxSizeOfInstructions'):
            setattr(font['maxp'], field, 1 if field == 'maxZones' else 0)
        suffix = f' Outline Experiment {mode} {size}'
        for record in font['name'].names:
            if record.nameID in (1, 3, 4, 6, 16):
                value = record.toUnicode() + suffix
                if record.nameID == 6:
                    value = value.replace(' ', '-')
                record.string = value.encode(record.getEncoding())
    result = io.BytesIO()
    font.save(result, reorderTables=False)
    return result.getvalue()


def verify(source, result, mode):
    a = TTFont(io.BytesIO(source), recalcTimestamp=False)
    b = TTFont(io.BytesIO(result), recalcTimestamp=False, checkChecksums=2)
    assert a.getGlyphOrder() == b.getGlyphOrder(), 'glyph IDs changed'
    preserved = ('cmap', 'GSUB', 'GPOS', 'GDEF', 'hmtx', 'OS/2')
    tables = {}
    for table in preserved:
        if table in a:
            assert a.getTableData(table) == b.getTableData(table), f'{table} changed'
            tables[table] = sha(b.getTableData(table))
    for field in ('ascent', 'descent', 'lineGap', 'advanceWidthMax'):
        assert getattr(a['hhea'], field) == getattr(b['hhea'], field), f'hhea.{field} changed'
    assert a['head'].unitsPerEm == b['head'].unitsPerEm
    assert a['hmtx'].metrics == b['hmtx'].metrics
    changed = 0
    for name in a.getGlyphOrder():
        old_coords, old_ends, old_flags = a['glyf'][name].getCoordinates(a['glyf'])
        new_coords, new_ends, new_flags = b['glyf'][name].getCoordinates(b['glyf'])
        # Flattening can apply composite offset rounding; record, never hide it.
        if list(old_coords) != list(new_coords) or list(old_ends) != list(new_ends) or list(old_flags) != list(new_flags):
            changed += 1
        if mode != 'identity':
            assert not b['glyf'][name].isComposite()
            assert not getattr(b['glyf'][name], 'program', Program()).getBytecode()
    if mode == 'identity':
        assert changed == 0, 'Identity changed outlines'
    return {'preservedTableSha256': tables, 'glyphCount': len(b.getGlyphOrder()), 'changedGlyphOutlines': changed, 'advancesAndBearingsUnchanged': True, 'unitsPerEm': b['head'].unitsPerEm}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sizes', type=int, nargs='+', default=[16])
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError('Output directory must be new')
    if any(size < 1 or size > 256 for size in args.sizes):
        raise ValueError('Size must be 1..256')
    source = args.source.read_bytes()
    args.output.mkdir(parents=True)
    manifest = {'status': 'experimental-unqualified', 'source': str(args.source.resolve()), 'sourceSha256': sha(source), 'scriptSha256': sha(Path(__file__).read_bytes()), 'python': platform.python_version(), 'fontTools': fontTools.__version__, 'freetypePy': '2.5.1', 'freetype': freetype.version(), 'hintTarget': 'FT_LOAD_TARGET_NORMAL, FT_LOAD_NO_BITMAP, FT_LOAD_NO_AUTOHINT; default interpreter properties', 'unhintedTarget': 'FT_LOAD_NO_SCALE, FT_LOAD_NO_HINTING, FT_LOAD_NO_BITMAP, FT_LOAD_NO_AUTOHINT; exact font-unit coordinates', 'quantization': 'fontTools otRound to integer source font units', 'fonts': []}
    cases = [('identity', 16, 'identity.ttf')]
    for size in args.sizes:
        cases.extend([('unhinted', size, f'unhinted-{size}.ttf'), ('hinted-native-normal', size, f'hinted-native-normal-{size}.ttf')])
    for mode, size, filename in cases:
        result = transform(source, mode, size)
        assert result == transform(source, mode, size), 'Generation is not deterministic'
        invariant = verify(source, result, mode)
        path = args.output / filename
        path.write_bytes(result)
        manifest['fonts'].append({'mode': mode, 'size': size, 'path': str(path.resolve()), 'sha256': sha(result), 'bytes': len(result), 'repeatByteIdentical': True, **invariant})
    (args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
