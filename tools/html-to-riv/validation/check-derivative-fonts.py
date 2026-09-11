#!/usr/bin/env python3
"""Run an exploratory font matrix; original font remains the Chrome reference.

Usage: check-derivative-fonts.py CONFIG.json NEW_OUTPUT
Config: {referenceFont: absolute path, cases: [{name, font: absolute path,
fontSize, nativeY, extraFillAlpha, text, x}]}. The first case is the shaping
control. This runner does not qualify public text, wrapping or line-box layout.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

MODULE = Path(__file__).resolve().parents[1]

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def run(command, log, allowed=(0,)):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    if result.returncode not in allowed:
        raise RuntimeError(f"Command failed ({result.returncode}): {command}; see {log}")
    return result.returncode

def main():
    config_path, output = map(lambda p: Path(p).resolve(), sys.argv[1:])
    config = json.loads(config_path.read_text())
    output.mkdir(parents=True, exist_ok=False)
    shutil.copy2(config_path, output / 'config.json')
    shutil.copy2(__file__, output / 'check-derivative-fonts.py')
    reference = Path(config['referenceFont'])
    generator = MODULE / 'target/debug/examples/ordinary-text'
    baseline = MODULE / 'output/immutable-baseline-toolchain-r2'
    observer = MODULE / 'output/ordinary-text-observer-r1/text-probe'
    driver = MODULE / 'validation/check-ordinary-text.mjs'
    inputs = [config_path, reference, generator, observer, driver,
              baseline / 'baseline-probe', baseline / 'renderer-replay']
    inputs += [Path(c['font']) for c in config['cases']]
    bindings = {str(p): digest(p) for p in inputs}
    rows = []
    control = None
    for case in config['cases']:
        name = case['name']
        assert name and all(c.isalnum() or c in '-_' for c in name)
        directory = output / name
        directory.mkdir()
        scene = directory / 'scene.riv'
        command = [generator, scene, case['font'], case['fontSize'], case['nativeY'],
                   'normal', 0, case['extraFillAlpha'], case['text'], case['x']]
        run(command, directory / 'generate.log')
        expected = dict(fontSize=case['fontSize'], x=case['x'], y=20,
                        text=case['text'], lineHeight='normal')
        ref = directory / 'reference.json'
        ref.write_text(json.dumps(expected, indent=2) + '\n')
        code = run(['node', driver, scene, reference, baseline / 'baseline-probe',
                    baseline / 'renderer-replay', directory / 'render', ref],
                   directory / 'render.log', (0, 1))
        receipt_path = directory / 'render/receipt.json'
        receipt = json.loads(receipt_path.read_text())
        assert len(receipt['rows']) == 8
        assert code == (0 if receipt['pixelPass'] == 8 else 1)
        run([observer, scene, directory / 'metrics', '240x160', '390x200',
             '768x120', '240x160'], directory / 'metrics.log')
        metrics = json.loads((directory / 'metrics/textmetrics.json').read_text())
        frames = metrics['frames']
        assert len(frames) == 8
        texts = [f['texts'] for f in frames]
        assert texts[0] and texts[0][0]['paragraphRuns'], 'Font did not shape'
        if control is None:
            control = texts[0]
        stable = all(t == control for t in texts)
        row = dict(name=name, command=list(map(str, command)),
                   pixelPass=receipt['pixelPass'], frames=8,
                   shapingEqualsFirstCase=stable,
                   receiptSha256=digest(receipt_path), sceneSha256=digest(scene))
        rows.append(row)
        print(json.dumps(row), flush=True)
    for path, sha in bindings.items():
        assert digest(path) == sha, f'Input changed: {path}'
    result = dict(scope='Exploratory derivative-font matrix; no public admission or wrap qualification',
                  bindings=bindings, cases=rows,
                  frames=sum(r['frames'] for r in rows),
                  pixelPass=sum(r['pixelPass'] for r in rows))
    result['files'] = {str(p.relative_to(output)): digest(p)
                       for p in sorted(output.rglob('*')) if p.is_file()}
    (output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')

if __name__ == '__main__':
    main()
