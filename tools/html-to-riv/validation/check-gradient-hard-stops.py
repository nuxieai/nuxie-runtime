"""Check pixels around opaque CSS discontinuities against retained Chrome images.
Usage: SCRIPT REPLAY OUTPUT
An aggregate scene tolerance cannot waive this local boundary check.
"""
import hashlib
import json
import math
from pathlib import Path
import sys
from PIL import Image

replay, output = (Path(s).resolve() for s in sys.argv[1:])
report = json.loads((replay/'replay.json').read_text())
rows = []
for row in report['cases']:
    kind = next((k for k in ['hard-stop','decreasing'] if row['name'].startswith('linear-gradient-'+k+'-')),None)
    if kind is None:
        continue
    boundary = 20+(row['width']-40)*(.5 if kind=='hard-stop' else .7)
    samples=[]
    paths=[Path(row['prefix']+'.'+k+'.png') for k in ['browser','native']]
    for kind_name,path in zip(['browser','native'],paths):
        assert hashlib.sha256(path.read_bytes()).hexdigest()==row[kind_name+'Sha256']
    browser,native=[Image.open(p).convert('RGBA') for p in paths]
    for x in range(math.floor(boundary)-2,math.ceil(boundary)+2):
        a,b=browser.getpixel((x,70)),native.getpixel((x,70))
        error=max(abs(u-v) for u,v in zip(a,b))
        samples.append(dict(x=x,browser=a,native=b,maxChannelError=error,passed=error<=2))
    rows.append(dict(name=row['name'],width=row['width'],samples=samples,passed=all(s['passed'] for s in samples)))
assert len(rows)==16, 'Expected both initial hard-stop scenes with eight lifecycle frames each'
failed=sum(not r['passed'] for r in rows)
output.write_text(json.dumps(dict(status='failed' if failed else 'passed',frames=len(rows),failed=failed,rows=rows),indent=2)+'\n')
print(json.dumps(dict(frames=len(rows),failed=failed)))
sys.exit(1 if failed else 0)
