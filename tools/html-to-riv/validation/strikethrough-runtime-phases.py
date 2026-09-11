"""Native-glyph phase regression through the public CLI, import and renderer."""
import itertools
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
out = Path(os.environ.get('NUXIE_STRIKETHROUGH_MATRIX_DIR', str(root / 'output/playwright/html-to-riv/strikethrough-runtime-phases')))
out.mkdir(parents=True, exist_ok=True)
results = []
heights = [float(value) for value in os.environ.get('NUXIE_STRIKETHROUGH_HEIGHTS', '30,30.5').split(',')]
for font, size, height, padding in itertools.product(['Inter', 'OpenSans'], [20, 24], heights, [0, .25, .5, .75]):
    name = f'{font}-{size}-{height}-{padding}'
    folder = out / name
    env = dict(os.environ, NUXIE_STRIKETHROUGH_RUNTIME_DIR=str(folder), NUXIE_STRIKETHROUGH_FONT=font,
               NUXIE_STRIKETHROUGH_SIZE=str(size), NUXIE_STRIKETHROUGH_LINE_HEIGHT=str(height),
               NUXIE_STRIKETHROUGH_PADDING=str(padding), NUXIE_STRIKETHROUGH_TEXT='M\nM')
    run = subprocess.run(['node', str(root / 'tools/html-to-riv/validation/strikethrough-runtime-control.mjs'),
                          '--focus', '--transparent-glyphs'], env=env, cwd=root, text=True, capture_output=True)
    (out / f'{name}.log').write_text(run.stdout + run.stderr)
    report_path = folder / 'report.json'
    failures = json.loads(report_path.read_text())['cases'][0]['failures'] if report_path.exists() else ['missing report']
    results.append(dict(name=name, exitCode=run.returncode, failures=failures))
(out / 'summary.json').write_text(json.dumps(results, indent=2))
failed = [result for result in results if result['exitCode'] or result['failures']]
print(json.dumps(dict(total=len(results), passed=len(results)-len(failed), failures=failed), indent=2))
raise SystemExit(bool(failed))
