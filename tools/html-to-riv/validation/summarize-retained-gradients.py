"""Validate retained benchmark evidence and summarize paired completed-frame timings.

Usage: SCRIPT OUTPUT [--samples 100] [--warmups 20]
Requires three complete rounds of the 48 semantic-preserving configurations.
Optional forced-clockwise runs are intentionally outside this summary's scope.
Run --self-test for synthetic completeness, corruption and validation controls.
"""
import argparse
import copy
import hashlib
import itertools
import json
import math
from pathlib import Path
import statistics
import tempfile

BACKENDS = ('rust-metal', 'rust-metal-atomic')
KINDS = ('ordinary', 'css', 'tiled')
SIZES = ((240, 160), (1024, 768))

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def require(condition, message):
    if not condition:
        raise ValueError(message)

def configurations():
    return set((r, b, k, w, h, s, sharing) for r, b, k, (w, h), s, sharing in
               itertools.product(range(3), BACKENDS, KINDS, SIZES, (2, 256), ('distinct', 'shared')))

def audit(out, samples=100, warmups=20, harness=None):
    out = Path(out).resolve()
    receipt_path = out / 'receipt.json'
    initial_hash = sha(receipt_path)
    receipt = read(receipt_path)
    require(receipt['status'] == 'measured' and receipt.get('finishedAt'), 'benchmark not terminal')
    require(sha(receipt['binary']) == receipt['binarySha256'], 'binary hash changed')
    harness = harness or Path(__file__).with_name('benchmark-retained-gradients.py')
    require(sha(harness) == receipt['harnessSha256'], 'harness hash changed')
    require(len(receipt['cases']) == 144, 'expected exactly 3 rounds x 48 cases')
    found = {}
    bindings = []
    for entry in receipt['cases']:
        name = entry['name']
        require(Path(name).name == name and name not in ('.', '..'), 'unsafe case name')
        folder = out / name
        result_path = folder / 'receipt.json'
        require(entry['exitCode'] == 0, f'{name}: process failed')
        require(sha(result_path) == entry['receiptSha256'], f'{name}: receipt hash changed')
        result = read(result_path)
        require(result == entry['result'], f'{name}: embedded result differs')
        require(result['status'] == 'measured', f'{name}: incomplete result')
        config = result['config']
        key = (entry['round'], config['backend'], config['kind'], config['width'], config['height'], config['stops'], 'shared' if config['sharedStops'] else 'distinct')
        require(key in configurations() and key not in found, f'{name}: duplicate/unexpected configuration')
        require(config['samples'] == samples and config['warmups'] == warmups and config['gradients'] == 12 and config['cssEquivalenceEligible'] is True, f'{name}: sampling/semantic configuration differs')
        expected_command = [receipt['binary'], str(folder), config['backend'], config['kind'], str(config['width']), str(config['height']), str(config['stops']), str(warmups), str(samples), key[-1]]
        require(entry['command'] == expected_command, f'{name}: command/config mismatch')
        validation = result['validation']
        require(validation['analyticSamples'] > 0 and 0 <= validation['maxChannelError'] <= 2 and validation['allFramesByteIdentical'] is True, f'{name}: correctness validation failed')
        require(set(entry['pngHashes']) == {'first.png', 'last.png'}, f'{name}: missing PNG binding')
        for png, digest in entry['pngHashes'].items():
            require(sha(folder / png) == digest, f'{name}: PNG hash changed')
        require(entry['pngHashes']['first.png'] == entry['pngHashes']['last.png'], f'{name}: repeat PNG differs')
        raw = result['raw']
        require(len(raw) == 1 + warmups + samples, f'{name}: missing raw samples')
        times = []
        for i, row in enumerate(raw):
            expected_phase = 'cold' if i == 0 else ('warmup' if i <= warmups else 'measured')
            require(row['iteration'] == i and row['phase'] == expected_phase, f'{name}: invalid sample phase/order')
            for field in ('totalMs', 'beginAndDrawMs', 'finishWaitReadbackMs'):
                require(isinstance(row[field], (int, float)) and math.isfinite(row[field]) and row[field] >= 0, f'{name}: invalid timing')
            require(row['totalMs'] > 0 and 'backendWork' in row and 'executionInventory' in row, f'{name}: missing timing/work evidence')
            if expected_phase == 'measured':
                times.append(row['totalMs'])
        median = statistics.median(times)
        require(result['medianMs'] == median and result['minMs'] == min(times) and result['maxMs'] == max(times), f'{name}: aggregate differs from raw samples')
        found[key] = median
        bindings.append(dict(name=name, receiptSha256=entry['receiptSha256'], pngHashes=entry['pngHashes']))
    require(set(found) == configurations(), 'missing configurations')
    rows = []
    for backend, (width, height), stops, sharing in itertools.product(BACKENDS, SIZES, (2, 256), ('distinct', 'shared')):
        medians = {kind: [found[(r, backend, kind, width, height, stops, sharing)] for r in range(3)] for kind in KINDS}
        ratios = {kind: [medians[kind][r] / medians['ordinary'][r] for r in range(3)] for kind in ('css', 'tiled')}
        rows.append(dict(backend=backend, width=width, height=height, stops=stops, sharing=sharing, roundMediansMs=medians, medianOfRoundMediansMs={k: statistics.median(v) for k, v in medians.items()}, pairedRoundRatios=ratios, medianPairedRoundRatios={k: statistics.median(v) for k, v in ratios.items()}))
    require(sha(receipt_path) == initial_hash, 'receipt changed during audit')
    return dict(status='validated', metric='Completed-frame wall latency including GPU completion wait and readback; not GPU-only time or presentation throughput', rounds=3, configurationsPerRound=48, samplesPerCase=samples, warmupsPerCase=warmups, receiptSha256=initial_hash, binarySha256=receipt['binarySha256'], harnessSha256=receipt['harnessSha256'], environment=receipt['environment'], limitations=receipt['limitations'], cases=bindings, summaries=rows)

def markdown(result):
    lines = [result['metric'] + '.', '', 'Values are medians of three round medians. Ratios are medians of paired CSS/ordinary or tiled/ordinary ratios within each round. Cold and warmup frames are excluded. No performance pass cutoff is applied; raw receipts remain unchanged.', '', '| Backend | Size | Stops | Sharing | Ordinary ms | CSS ms | Tiled ms | CSS/ordinary | Tiled/ordinary |', '|---|---|---:|---|---:|---:|---:|---:|---:|']
    for row in result['summaries']:
        m, r = row['medianOfRoundMediansMs'], row['medianPairedRoundRatios']
        lines.append(f"| {row['backend']} | {row['width']}×{row['height']} | {row['stops']} | {row['sharing']} | {m['ordinary']:.3f} | {m['css']:.3f} | {m['tiled']:.3f} | {r['css']:.3f} | {r['tiled']:.3f} |")
    lines += ['', 'Environment: `' + json.dumps(result['environment'], sort_keys=True) + '`', '', 'Limits: ' + '; '.join(result['limitations']) + '.','', 'Receipt SHA-256: `' + result['receiptSha256'] + '`.']
    return '\n'.join(lines) + '\n'

def self_test():
    with tempfile.TemporaryDirectory() as temp:
        out = Path(temp).resolve(); binary = out/'binary'; binary.write_bytes(b'synthetic binary'); harness = out/'harness'; harness.write_bytes(b'synthetic harness')
        top = dict(status='measured', finishedAt='synthetic', binary=str(binary), binarySha256=sha(binary), harnessSha256=sha(harness), environment={'synthetic': True}, limitations=['synthetic fixtures only'], cases=[])
        for index, key in enumerate(sorted(configurations())):
            round_, backend, kind, width, height, stops, sharing = key
            name = str(index); folder = out/name; folder.mkdir();(folder/'first.png').write_bytes(b'bound PNG bytes');(folder/'last.png').write_bytes(b'bound PNG bytes')
            value = {'ordinary':1., 'css':2., 'tiled':3.}[kind] * (round_+1)
            raw = [dict(iteration=i, phase='cold' if i==0 else ('warmup' if i==1 else 'measured'), totalMs=value, beginAndDrawMs=value/2, finishWaitReadbackMs=value/2, backendWork={}, executionInventory={}) for i in range(5)]
            result = dict(status='measured', config=dict(backend=backend, kind=kind, width=width, height=height, stops=stops, sharedStops=sharing=='shared', samples=3, warmups=1, gradients=12, cssEquivalenceEligible=True), validation=dict(analyticSamples=1,maxChannelError=0,allFramesByteIdentical=True), raw=raw, medianMs=value,minMs=value,maxMs=value)
            (folder/'receipt.json').write_text(json.dumps(result))
            top['cases'].append(dict(name=name,round=round_,exitCode=0,command=[str(binary),str(folder),backend,kind,str(width),str(height),str(stops),'1','3',sharing],result=result,receiptSha256=sha(folder/'receipt.json'),pngHashes={p:sha(folder/p) for p in ('first.png','last.png')}))
        def run(data):
            (out/'receipt.json').write_text(json.dumps(data));return audit(out,3,1,harness)
        good=run(top);require(all(r['medianPairedRoundRatios']=={'css':2.,'tiled':3.} for r in good['summaries']), 'paired ratio control')
        mutations=[lambda d:d.update(status='running'),lambda d:d['cases'].pop(),lambda d:d['cases'].__setitem__(0,copy.deepcopy(d['cases'][1])),lambda d:d['cases'][0].update(receiptSha256='bad'),lambda d:d['cases'][0]['pngHashes'].update({'first.png':'bad'}),lambda d:d.update(binarySha256='bad')]
        for mutate in mutations:
            data=copy.deepcopy(top);mutate(data)
            try:run(data)
            except ValueError:pass
            else:raise AssertionError('negative control accepted')
        for mutate in [lambda r:r['raw'].pop(),lambda r:r['validation'].update(allFramesByteIdentical=False),lambda r:r['validation'].update(analyticSamples=0),lambda r:r['config'].update(warmups=0),lambda r:r.update(medianMs=99),lambda r:r['raw'][2].update(totalMs=float('nan'))]:
            data=copy.deepcopy(top);entry=data['cases'][0];mutate(entry['result']);path=out/entry['name']/'receipt.json';path.write_text(json.dumps(entry['result']));entry['receiptSha256']=sha(path)
            try:run(data)
            except ValueError:pass
            else:raise AssertionError('malformed result accepted')
            path.write_text(json.dumps(top['cases'][0]['result']))
        print('PASS: complete 144-case fixture, paired ratios, and 12 malformed/incomplete/hash controls')

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path,nargs='?');parser.add_argument('--samples',type=int,default=100);parser.add_argument('--warmups',type=int,default=20);parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    if args.self_test:self_test()
    else:
        if args.output is None:parser.error('output required')
        result=audit(args.output,args.samples,args.warmups)
        (args.output/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
        (args.output/'summary.md').write_text(markdown(result))
        print(f"Validated {result['rounds'] * result['configurationsPerRound']} cases; wrote summary.json and summary.md")
