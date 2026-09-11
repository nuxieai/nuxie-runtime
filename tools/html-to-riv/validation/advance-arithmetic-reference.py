"""Compare arithmetic candidates against measured single-glyph Chrome widths.
Diagnostic only: it does not replace native shaping or its regression checks.
"""
import json
from pathlib import Path
import struct
import sys

folder = Path(sys.argv[1])
f32 = lambda value: struct.unpack('f', struct.pack('f', value))[0]
rows = []
for result_path in sorted(folder.glob('*-*.json')):
    result = json.loads(result_path.read_text())
    data = Path(result['fontPath']).read_bytes()
    count = struct.unpack_from('>H', data, 4)[0]
    tables = {data[12+i*16:16+i*16].decode('ascii'): struct.unpack_from('>II', data, 20+i*16) for i in range(count)}
    units = struct.unpack_from('>H', data, tables['head'][0]+18)[0]
    long_metrics = struct.unpack_from('>H', data, tables['hhea'][0]+34)[0]
    size = f32(result['size'])
    for case in result['results']:
        if len(case['text']) != 1 or len(case['glyphs']) != 1:
            continue
        glyph = case['glyphs'][0]['glyph']
        advance = struct.unpack_from('>H', data, tables['hmtx'][0]+min(glyph,long_metrics-1)*4)[0]
        candidates = {
            'scale_first': f32(advance*f32(size*65536/units)),
            'multiply_first': f32(f32(advance*size)/units)*65536,
            'normalize_first': f32(f32(advance/units)*size)*65536,
            'double_pixel': f32(advance*size/units)*65536,
        }
        rows.append({'font':Path(result['fontPath']).name,'size':size,'text':case['text'],'glyph':glyph,'fontAdvance':advance,'expected':case['width'],
                     'candidates':{k:int(v)/65536 for k,v in candidates.items()}})
summary = {key:sum(row['candidates'][key]!=row['expected'] for row in rows) for key in rows[0]['candidates']}
print(json.dumps({'cases':len(rows),'failures':summary,'rows':rows},indent=2))
