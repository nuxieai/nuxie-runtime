"""Contact sheets for box-only visual runs; full screenshots are retained at 1/2 scale.

Usage: python3 box-contact-review.py OUTPUT REVIEW_DIR [REVIEW_DIR ...]
Groups exactly identical browser/native PNG pairs within the supplied runs.
The manifest retains every source case and both SHA256 hashes. Inspecting a
sheet is still required; generating sheets alone does not mark any case reviewed.
Do not use this reduced scale to qualify text raster quality.
"""
import hashlib
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
groups = {}
for directory in sys.argv[2:]:
    for case in json.loads((Path(directory) / 'review.json').read_text())['cases']:
        prefix = case['artifactPrefix']
        hashes = tuple(hashlib.sha256(Path(prefix + '.' + kind + '.png').read_bytes()).hexdigest()
                       for kind in ['browser', 'native'])
        group = groups.setdefault(hashes, {'browserSha256': hashes[0], 'nativeSha256': hashes[1], 'cases': []})
        group['cases'].append({'reviewDirectory': directory, **case})
rows = list(groups.values())
for start in range(0, len(rows), 6):
    sheet = Image.new('RGB', (1250, 1200), '#ddd')
    draw = ImageDraw.Draw(sheet)
    for offset, row in enumerate(rows[start:start + 6]):
        index = start + offset
        row['index'] = index
        row['sheet'] = f'sheet-{start // 6:02}.png'
        case = row['cases'][0]
        y = offset * 200
        draw.text((8, y + 3), f"{index}: {case['name']} at {case['width']}px; {len(row['cases'])} identical pair(s)", fill='black')
        for col, kind in enumerate(['browser', 'native', 'diff']):
            draw.text((8 + col * 412, y + 16), kind, fill='black')
            im = Image.open(case['artifactPrefix'] + '.' + kind + '.png').convert('RGB')
            im = im.resize((im.width // 2, im.height // 2), Image.Resampling.LANCZOS)
            sheet.paste(im, (8 + col * 412, y + 32))
    sheet.save(out / f'sheet-{start // 6:02}.png')
(out / 'manifest.json').write_text(json.dumps({'purpose': 'Box inspection, pending manual review', 'cases': sum(len(r['cases']) for r in rows), 'uniquePairs': len(rows), 'rows': rows}, indent=2) + '\n')
print(json.dumps({'cases': sum(len(r['cases']) for r in rows), 'uniquePairs': len(rows), 'sheets': (len(rows) + 5) // 6}))
