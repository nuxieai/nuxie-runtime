"""Diagnostic greedy reducer; retain only the observed red-support failure."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
output = root / 'output/playwright/html-to-riv/underline-dpr-reduction'
output.mkdir(parents=True, exist_ok=True)
text = 'agypqj gap agypqj gap agypqj ga'
trials = []
index = 0
while index < len(text):
    candidate = text[:index] + text[index + 1:]
    if not candidate.strip():
        index += 1
        continue
    directory = output / str(len(trials))
    env = {**os.environ, 'NUXIE_DPR_TEXT': candidate,
           'NUXIE_HTML_REVIEW_DIR': str(directory)}
    run = subprocess.run(['node', 'tools/html-to-riv/validation/underline-dpr-control.mjs',
                          '--focus', '--transparent-glyphs'], cwd=root, env=env,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    result = directory / 'results.json'
    if not result.exists():
        raise RuntimeError(run.stderr.decode()[-2000:])
    cases = json.loads(result.read_text())['cases']
    retained = cases[0]['failures'] == ['red decoration support']
    trials.append({'text': candidate, 'retained': retained, 'directory': str(directory)})
    if retained:
        text = candidate
        index = 0
    else:
        index += 1
    (output / 'reduction.json').write_text(json.dumps({'text': text, 'trials': trials}, indent=2))
print(json.dumps({'text': text, 'trials': len(trials)}))
