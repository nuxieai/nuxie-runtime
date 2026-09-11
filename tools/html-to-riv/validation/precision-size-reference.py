"""Diagnostic actual shaping across sizes; requires the outline_probe example.
No browser-equivalence or universal font-range claim is made by this probe.
"""
import json
from pathlib import Path
import subprocess

module = Path(__file__).resolve().parents[1]
root = module.parents[1]
fixtures = [
    ('Inter-Regular.ttf', 'WWW ffi agypqj'),
    ('OpenSans-Regular.ttf', 'WWW ffi agypqj'),
    ('NuxieJapaneseFixture-Regular.otf', '／＿ agypqj'),
    ('NotoSansOgham-Regular.ttf', '\u1681\u1682\u1683'),
]
results = []
for name, text in fixtures:
    for size in [.01, .1, 1, 24, 100, 1000, 16384, 32768, 1000000]:
        run = subprocess.run([
            str(root / 'target/debug/examples/outline_probe'),
            str(module / 'tests/assets' / name), text, str(size), 'css',
        ], capture_output=True, text=True, check=True)
        glyphs = json.loads(run.stdout)
        advances = [g['advance'] for g in glyphs]
        results.append({
            'font': name, 'text': text, 'size': size,
            'advancePerEm': sum(advances) / size,
            'glyphCount': len(glyphs),
            'missingGlyphs': sum(g['glyph'] == 0 for g in glyphs),
            'negativeAdvances': any(a < 0 for a in advances),
        })
output = module / 'validation/precision-size-reference.json'
output.write_text(json.dumps({'qualification': False, 'results': results}, indent=2) + '\n')
print(json.dumps({'samples': len(results), 'negative': sum(r['negativeAdvances'] for r in results), 'missing': sum(r['missingGlyphs'] for r in results)}))
