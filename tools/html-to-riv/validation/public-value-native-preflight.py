"""Compile candidate scenes and literal controls; preserve failures, never render."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
compiler = Path(sys.argv[1]).resolve()
out = Path(sys.argv[2]).resolve()
assert not out.exists()
out.mkdir(parents=True)
source = ROOT / 'validation/public-value-native-cases.json'
cases = json.loads(source.read_text())
(out / 'cases.json').write_bytes(source.read_bytes())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
rows = []
for case in cases:
    for width, height in [(240, 160), (390, 200), (768, 120)]:
        results = {}
        for role, css in [('actual', case['css']), ('control', case['literalCss'])]:
            directory = out / case['name'] / f'{width}x{height}' / role
            directory.mkdir(parents=True)
            request = directory / 'request.json'
            request.write_text(json.dumps(dict(html=case['html'], css=css, width=width, height=height)) + '\n')
            command = [str(compiler), str(request), str(directory / 'scene.riv')]
            result = subprocess.run(command, capture_output=True, text=True)
            (directory / 'compile.log').write_text(result.stdout + result.stderr)
            observation = dict(exitCode=result.returncode, command=command, requestSha256=sha(request), logSha256=sha(directory / 'compile.log'))
            if result.returncode == 0:
                observation.update(rivSha256=sha(directory / 'scene.riv'), mapSha256=sha(directory / 'scene.map.json'))
            else:
                observation['diagnostics'] = result.stdout + result.stderr
            results[role] = observation
        same = all(r['exitCode'] == 0 for r in results.values()) and all(results['actual'][k] == results['control'][k] for k in ['rivSha256', 'mapSha256'])
        rows.append(dict(name=case['name'], width=width, height=height, results=results, exactSourceControlBytesAndMap=same))
receipt = dict(scope='Compiler admission and exact source/control byte/map comparison only. No native rendering qualification.', compiler=dict(path=str(compiler), sha256=sha(compiler)), fixturesSha256=sha(source), driverSha256=sha(Path(__file__)), rows=rows)
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(observations=len(rows), acceptedSource=sum(r['results']['actual']['exitCode'] == 0 for r in rows), acceptedControl=sum(r['results']['control']['exitCode'] == 0 for r in rows), exactSourceControl=sum(r['exactSourceControlBytesAndMap'] for r in rows))))
