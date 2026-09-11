"""Build full-resolution Chrome/native/diff sheets for a replay (no review claim).

Usage: python3 make-replay-sheets.py REPLAY_DIRECTORY [CASE_NAME ...]
Each case gets one sheet with viewport rows; record-visual-review.py records
inspection separately. Source screenshots are retained without resampling.
"""
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

out = Path(sys.argv[1])
requested = set(sys.argv[2:])
cases = json.loads((out / 'replay.json').read_text())['cases']
names = {case['name'] for case in cases}
if requested - names:
    raise ValueError(f'Unknown cases: {requested - names}')
folder = out / 'review-sheets'
folder.mkdir(exist_ok=True)
for name in sorted(requested or names):
    rows = sorted((case for case in cases if case['name'] == name), key=lambda case: case['width'])
    column_width = max(case['width'] for case in rows) + 8
    sheet = Image.new('RGB', (column_width * 3, 350 * len(rows)), '#ddd')
    draw = ImageDraw.Draw(sheet)
    for row, case in enumerate(rows):
        for col, kind in enumerate(['browser', 'native', 'diff']):
            x, y = col * column_width, row * 350
            draw.text((x + 4, y + 4), f'{kind} / {case["width"]}', fill='black')
            with Image.open(case['prefix'] + '.' + kind + '.png') as source:
                if source.size != (case['width'], 320):
                    raise ValueError(f'Unexpected screenshot dimensions: {case["prefix"]}')
                sheet.paste(source.convert('RGB'), (x + 4, y + 24))
    target = folder / (name + '.png')
    if target.exists():
        with Image.open(target) as existing:
            if existing.size != sheet.size or existing.convert('RGB').tobytes() != sheet.tobytes():
                raise ValueError(f'Refusing to replace a changed review sheet: {target}')
    else:
        sheet.save(target)
print(f'{len(requested or names)} sheets prepared; inspection remains separate')
