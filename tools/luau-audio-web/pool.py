#!/usr/bin/env python3
"""Build and package the Luau browser qualification exclusively on the pool."""
import argparse
import importlib.util
import json
import os
import re
from pathlib import Path
import shutil
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['build', 'test', 'package'])
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--probe', type=Path)
    parser.add_argument('--kernel', type=Path)
    parser.add_argument('--output', type=Path, default=HERE / 'pkg')
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('audio_pool_native', Path.home() / '.local/share/nuxie-homelab/native.py')
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    sys.path.insert(0, str(ROOT / 'tools'))
    from bazel_runtime_build import BazelRuntime
    os.environ['NUXIE_HOMELAB_RUNTIME_INVOCATION'] = str(uuid.uuid4())
    if args.command == 'package':
        if not args.probe or not args.kernel:
            parser.error('package requires --probe and --kernel from pool build products')
        probe = args.probe.resolve(strict=True)
        kernel = args.kernel.resolve(strict=True)
        output = args.output.resolve()
        # Pool products live under .nuxie, which is deliberately excluded from
        # runtime inputs. Stage only the explicitly selected compiled artifact.
        with tempfile.TemporaryDirectory(prefix='nuxie-audio-package-') as staging:
            staged_probe = Path(staging) / 'probe.wasm'
            shutil.copy2(probe, staged_probe)
            os.environ['NUXIE_HOMELAB_RUNTIME_IO'] = json.dumps({
                'schema':'nuxie-runtime-io.v1', 'platform':'macos', 'inputs':[str(staged_probe)],
                'outputs':[{'path':str(output),'directory':True}]})
            result = native.run(ROOT, 'run', ['//bazel:wasm_bindgen'], _required_platform='macos',
                _run_arguments=[str(staged_probe),'--target','web','--out-name','probe','--out-dir',str(output),'--keep-lld-exports'])
        if result != 0: return result
        # Packaging only: generated glue and compiled code came from the pool.
        glue = output / 'probe.js'
        source = glue.read_text()
        # wasm-bindgen represents raw Wasm import modules as JS imports. This
        # harness supplies nuxie_audio explicitly at instantiation instead.
        source, audio_modules = re.subn(
            r'^import \* as (\w+) from "nuxie_audio";?$',
            r'const \1 = Object.freeze({});', source, flags=re.MULTILINE)
        if audio_modules == 0:
            raise RuntimeError('wasm-bindgen audio import declarations changed')
        for name in ['__wbg_get_imports', '__wbg_finalize_init']:
            if name not in source: raise RuntimeError('wasm-bindgen glue shape changed: '+name)
        source += '''\nexport async function initWithAudioHost(bytes, audioImports) {
    const imports = __wbg_get_imports();
    Object.assign(imports, audioImports);
    const {instance, module} = await WebAssembly.instantiate(bytes, imports);
    __wbg_finalize_init(instance, module);
    return instance.exports;
}\n'''
        glue.write_text(source)
        with tempfile.NamedTemporaryFile(dir=output, prefix='.kernel-', delete=False) as staged:
            staged_kernel = Path(staged.name)
        try:
            shutil.copyfile(kernel, staged_kernel)
            staged_kernel.replace(output/'kernel.wasm')
        finally:
            staged_kernel.unlink(missing_ok=True)
        for source in [*HERE.glob('*.mjs'), HERE/'qualification.html']:
            shutil.copy2(source, output/source.name)
        print(output)
        return 0
    labels = (['//tools/luau-audio-web:kernel','//tools/luau-audio-web:probe'] if args.command == 'build'
              else ['//tools/luau-audio-web:transport_test','//tools/luau-audio-web:probe_native_test'])
    flags = BazelRuntime.provenance_options()
    if args.release: flags += ['--config=release']
    if args.command == 'build': flags += ['--platforms=@rules_rust//rust/platform:wasm', '--@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=abort']
    else: flags += ['--test_output=all']
    pointer = ROOT/'.nuxie/audio-web-products.json'
    result = native.run(ROOT,args.command,flags+labels,_required_platform='macos' if args.command == 'build' else 'linux',
        _products=args.command=='build',_pointer=pointer if args.command=='build' else None)
    if result is None: raise RuntimeError('pool unavailable; no local compile fallback')
    if result == 0 and args.command == 'build':
        products=json.loads(pointer.read_text())
        for path in products['outputs']: print(ROOT/'.nuxie/homelab'/products['runId']/path)
    return result
if __name__=='__main__': sys.exit(main())
