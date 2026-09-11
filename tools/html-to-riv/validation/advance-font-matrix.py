"""Compare CSS advance quantization across embedded fonts and sizes."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
output = Path(os.environ.get('NUXIE_ADVANCE_REVIEW_DIR', root / 'output/playwright/html-to-riv/advance-callback-fonts'))
output.mkdir(parents=True, exist_ok=True)
summary = []
for name in ['Inter-Regular.ttf', 'OpenSans-Regular.ttf', 'NuxieJapaneseFixture-Regular.otf']:
    for size in [float(value) for value in os.environ.get('NUXIE_ADVANCE_SIZES', '16,24,36').split(',')]:
        env = {**os.environ, 'NUXIE_ADVANCE_FONT': str(root / 'tools/html-to-riv/tests/assets' / name),
               'NUXIE_ADVANCE_SIZE': str(size)}
        run = subprocess.run(['node', 'tools/html-to-riv/validation/underline-advance-components.mjs', '--check'],
                             cwd=root, env=env, capture_output=True, text=True)
        if not run.stdout.strip():
            raise RuntimeError(run.stderr[-1500:])
        result = json.loads(run.stdout)
        (output / f'{name}-{size}.json').write_text(run.stdout)
        failures = [{'text': r['text'], 'expected': r['width'], 'actual': r['nativeFloatWidth']}
                    for r in result['results'] if r['width'] != r['nativeFloatWidth']]
        summary.append({'font': name, 'size': size, 'cases': len(result['results']), 'failures': failures})
(output / 'summary.json').write_text(json.dumps(summary, indent=2))
print(json.dumps({'cases': sum(r['cases'] for r in summary), 'failures': sum(len(r['failures']) for r in summary)}))
raise SystemExit(1 if any(r['failures'] for r in summary) else 0)
