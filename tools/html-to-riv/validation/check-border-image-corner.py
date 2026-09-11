"""Focused regression for the rounded image content corner missed by aggregate gates.
Usage: python3 check-border-image-corner.py REPLAY_DIRECTORY
The fixture has an8px border and12px/8px padding; pixel40,36 is inside
Chrome's rounded content-corner cutout. No aggregate tolerance is changed.
"""
import json
import sys
from pathlib import Path
from PIL import Image
out=Path(sys.argv[1])
rows=json.loads((out/'replay.json').read_text())['cases']
rows=[r for r in rows if r['name']=='border-content-image-content-box-hidden-r24']
assert len(rows)==3, 'Expected all three viewport references'
failures=[]
for row in rows:
    colors={kind:Image.open(row['prefix']+'.'+kind+'.png').convert('RGB').getpixel((40,36)) for kind in ('browser','native')}
    assert colors['browser']==(234,164,61), 'Pinned Chrome corner reference changed'
    if max(abs(a-b) for a,b in zip(colors['browser'],colors['native']))>2:
        failures.append({'width':row['width'],'pixel':[40,36],**colors})
print(json.dumps({'comparisons':len(rows),'failures':failures}))
sys.exit(bool(failures))
