"""Build a private source-snapshot observer and compare Chrome grammar receipts.

This observes only the grammar classifier, never changes production sources or
claims public compiler/native support. Requires already built local Rust deps.
"""
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
out = Path(sys.argv[1]).resolve()
assert not out.exists()
out.mkdir(parents=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text())
sources = []
for name in ['numeric_tokens.rs', 'value_grammar.rs']:
    source = root / 'src' / name
    snapshot = out / name
    shutil.copyfile(source, snapshot)
    sources.append(dict(path=str(source), snapshot=str(snapshot), sha256=sha(snapshot)))
(out / 'main.rs').write_text('''mod numeric_tokens;
mod value_grammar;
fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut rows: serde_json::Value = serde_json::from_str(&text).unwrap();
    for row in rows.as_array_mut().unwrap() {
        let validity = value_grammar::classify(row["property"].as_str().unwrap(), row["value"].as_str().unwrap());
        row["validity"] = format!("{validity:?}").into();
    }
    println!("{}", serde_json::to_string_pretty(&rows).unwrap());
}
''')
deps = root / 'target/debug/deps'
cssparser = next(deps.glob('libcssparser-*.rlib'))
serde = next(deps.glob('libserde_json-*.rlib'))
command = ['rustc', '--edition=2024', str(out / 'main.rs'), '-L', 'dependency=' + str(deps), '--extern', 'cssparser=' + str(cssparser), '--extern', 'serde_json=' + str(serde), '-o', str(out / 'observer')]
build = subprocess.run(command, capture_output=True, text=True)
(out / 'build.log').write_text(build.stdout + build.stderr)
assert build.returncode == 0, build.stderr
forms = read(root / 'output/public-value-native-r1/browser-r2/forms.json')
browser = read(root / 'output/public-value-native-r1/browser-r2/receipt.json')
probe_path = root / 'output/public-value-native-r1/grammar-probes-r3/receipt.json'
probes = read(probe_path)
by_name = {r['name']: r for r in browser['rows']}
inputs = [dict(name=f['name'], property=f['property'], value=f['tokens'], browserSupports=by_name[f['name']]['literalSupport'], corpus='original476') for f in forms]
inputs += [dict(name='probe-' + str(i), property=f['property'], value=f['value'], browserSupports=f['supports'], corpus='additional118') for i, f in enumerate(probes['rows'])]
(out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
run_command = [str(out / 'observer'), str(out / 'inputs.json')]
run = subprocess.run(run_command, capture_output=True, text=True)
assert run.returncode == 0, run.stderr
(out / 'rows.json').write_text(run.stdout)
rows = json.loads(run.stdout)
over_recovery = [r for r in rows if r['browserSupports'] and r['validity'] == 'Invalid']
valid_browser_rejects = [r for r in rows if not r['browserSupports'] and r['validity'] == 'Valid']
receipt = dict(scope='Private grammar-only observer against independently captured Chrome. No target admission, transport or native qualification claim.',
               cases=len(rows), invalidDespiteBrowserSupport=over_recovery, grammarValidBrowserRejects=valid_browser_rejects,
               sources=sources, command=command, runCommand=run_command,
               bindings=[dict(path=str(p), sha256=sha(p)) for p in [Path(__file__), out / 'main.rs', out / 'observer', out / 'inputs.json', out / 'rows.json', out / 'build.log', cssparser, serde, probe_path, root / 'output/public-value-native-r1/browser-r2/receipt.json', root / 'output/public-value-native-r1/browser-r2/forms.json']])
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(cases=len(rows), invalidDespiteBrowserSupport=over_recovery, grammarValidBrowserRejects=valid_browser_rejects), indent=2))
if over_recovery:
    sys.exit(1)
