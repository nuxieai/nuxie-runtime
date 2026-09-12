"""Build/test and freeze the compiler inputs used for value-recovery evidence.

Run from any directory with a fresh output directory argument. Native comparison
and prior-output regression consume the frozen CLI, never mutable Cargo output.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

module = Path(__file__).resolve().parents[1]
repo = module.parents[1]
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(name, value):
    (out / name).write_text(json.dumps(value, indent=2) + '\n')


paths = sorted({
    *(p for directory in ['src', 'js', 'tests', 'examples']
      for p in (module / directory).rglob('*') if p.is_file()),
    *(module / name for name in ['Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json']),
    *(p for pattern in ['*-cases.json', '*-rejections.json'] for p in (module / 'validation').glob(pattern)),
    Path(__file__).resolve(),
})
before = [dict(path=str(p), sha256=sha(p)) for p in paths]
write('inputs-before.json', before)
checks = []


def run(name, argv, env=None):
    log = out / f'{name}.log'
    with log.open('wb') as stream:
        result = subprocess.run(argv, cwd=repo, env=env, stdout=stream, stderr=subprocess.STDOUT)
    check = dict(name=name, argv=argv, cwd=str(repo), exitCode=result.returncode,
                 log=str(log), logSha256=sha(log))
    if env is not None:
        check['environmentOverrides'] = {key: env[key] for key in ['RUSTC', 'HTML_TO_RIV_BIN', 'HTML_TO_RIV_WASM'] if key in env}
    checks.append(check)
    write('checks.json', checks)
    print(json.dumps(check), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)


manifest = str(module / 'Cargo.toml')
run('full-rust', ['cargo', 'test', '--manifest-path', manifest, '--locked', '--no-fail-fast'])
run('native-build', ['cargo', 'build', '--manifest-path', manifest, '--locked'])
wasm_env = os.environ.copy()
wasm_env['RUSTC'] = subprocess.check_output(['rustup', 'which', 'rustc'], text=True).strip()
run('wasm-build', ['cargo', 'build', '--manifest-path', manifest, '--locked', '--target', 'wasm32-unknown-unknown', '--lib'], wasm_env)
run('typescript', ['node', str(module / 'node_modules/typescript/bin/tsc'), '--noEmit', '--strict', '--module', 'nodenext',
                   '--moduleResolution', 'nodenext', '--target', 'es2022', '--lib', 'es2022,dom', str(module / 'tests/types.mts')])
assert all(sha(Path(row['path'])) == row['sha256'] for row in before), 'Source changed during build/tests'
frozen = out / 'frozen'
frozen.mkdir()
bindings = []
for row in before:
    target = frozen / 'inputs' / Path(row['path']).relative_to(module)
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(row['path'], target)
    assert sha(target) == row['sha256']
    bindings.append(dict(**row, snapshot=str(target)))
for original, name in [(module / 'target/debug/html-to-riv', 'html-to-riv'),
                       (module / 'target/wasm32-unknown-unknown/debug/nuxie_html_to_riv.wasm', 'compiler.wasm')]:
    target = frozen / name
    shutil.copy2(original, target)
    bindings.append(dict(path=str(target), sha256=sha(target)))
(frozen / 'source-bindings.json').write_text(json.dumps(dict(files=bindings), indent=2) + '\n')
print(json.dumps(dict(frozen=str(frozen), compilerSha256=sha(frozen / 'html-to-riv'), wasmSha256=sha(frozen / 'compiler.wasm'))), flush=True)
node_env = os.environ.copy()
node_env.update(HTML_TO_RIV_BIN=str(frozen / 'html-to-riv'), HTML_TO_RIV_WASM=str(frozen / 'compiler.wasm'))
run('full-node', ['node', '--test', str(module / 'tests/transport-parity.mjs')], node_env)
run('runtime-guard', ['python3', str(module / 'validation/check-target-runtime.py')])
assert all(sha(Path(row['path'])) == row['sha256'] for row in before), 'Source changed during transport validation'
rust = (out / 'full-rust.log').read_text()
node = (out / 'full-node.log').read_text()
write('summary.json', dict(status='public-build-and-transport-pass', sourceUnchanged=True,
      rustTests=sum(map(int, re.findall(r'test result: ok\. (\d+) passed;', rust))),
      nodeTests=int(re.search(r'(?:#|ℹ) pass (\d+)\b', node).group(1)),
      compilerSha256=sha(frozen / 'html-to-riv'), wasmSha256=sha(frozen / 'compiler.wasm'),
      nativeQualification='Separate native/Chrome and regression receipts required.'))
