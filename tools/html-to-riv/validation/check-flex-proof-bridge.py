#!/usr/bin/env python3
"""Source-backed private compiler flex proof and immutable native geometry checks.

Usage: python3 validation/check-flex-proof-bridge.py NEW_OUTPUT [--render]
Run from the repository root or any directory. Source copies, direct dependency
bindings, classifications and failed observations are retained in NEW_OUTPUT.
No public admission, whole-scene qualification or rendered-edge claim is made.
"""
from pathlib import Path
from fractions import Fraction as Q
import argparse, hashlib, json, shutil, struct, subprocess

BASE = Path(__file__).resolve().parents[1]
ROOT = BASE.parents[1]
VIEWPORTS = [[400, 200], [100, 80], [1, 1], [16384, 16384], [400, 200]]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def source_hashes():
    return {str(p.relative_to(BASE / 'src')): sha(p) for p in (BASE / 'src').rglob('*') if p.is_file()}


def run(command, log):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True)
    Path(log).write_text(result.stdout + result.stderr)
    return result


def build(out):
    if not (out / 'source').exists():
        shutil.copytree(BASE / 'src', out / 'source')
        write(out / 'frozen-sources.json', source_hashes())
    frozen = json.loads((out / 'frozen-sources.json').read_text())
    assert frozen == source_hashes(), 'source changed before build'
    assert frozen == {str(p.relative_to(out / 'source')): sha(p) for p in (out / 'source').rglob('*') if p.is_file()}
    parts = ['use nuxie_html_to_riv::{CompileInput, CompileOutput, Diagnostic, SourceNode};']
    for name in ['color', 'css', 'css_whitespace', 'numeric_tokens', 'variables', 'wire', 'compiler']:
        parts.append(f'#[path="{out / "source" / (name + ".rs")}"] mod {name};')
    parts.append(r'''
use compiler::flex_numeric::ErrorEnvelope as E;
fn scalar(v:f64)->E {E::new(v,v,0.).unwrap()}
fn envelope(v:E)->serde_json::Value {serde_json::json!({"lower":v.lower(),"upper":v.upper(),"error":v.error_upper()})}
fn proofs(groups:&[compiler::flex_descriptor::Group],viewport:[E;2],origins:[E;2])->serde_json::Value {
    let analysis=compiler::flex_proof::analyze(groups,viewport,origins,0.125);
    serde_json::Value::Array(groups.iter().map(|g|{
        let parent=g.parent_id;
        match &analysis[&parent] {
            Err(e)=>serde_json::json!({"parent":parent,"parentPath":g.parent_path,"status":"unresolved","reason":format!("{e:?}"),"memberIds":g.items.iter().map(|i|i.authored_id).collect::<Vec<_>>()}),
            Ok(p)=>{
                let nodes:Vec<_>=g.items.iter().enumerate().map(|(k,i)|{
                    let mut sizes=[p.main.sizes[k],p.cross_sizes[k]];
                    let mut origins=[p.main.positions[k],p.cross_origins[k]];
                    let mut edges=[p.far_main[k],p.far_cross[k]];
                    if !g.row {sizes.swap(0,1);origins.swap(0,1);edges.swap(0,1);}
                    serde_json::json!({"id":i.authored_id,"sourceId":i.source_id,"sizes":sizes.map(envelope),"origins":origins.map(envelope),"edges":edges.map(envelope)})
                }).collect();
                serde_json::json!({"parent":parent,"parentPath":g.parent_path,"status":"bounded","nodes":nodes,"maxGeometryError":p.max_geometry_error,"joinError":p.main.join_error_upper})
            }
        }
    }).collect())
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let input:CompileInput=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let (output,groups)=match compiler::compile_profile_with_descriptors(&input,compiler::FlexPolicy::Candidate){Ok(v)=>v,Err(e)=>{eprintln!("{e:?}");std::process::exit(2)}};
    let path=std::path::Path::new(&args[2]);
    std::fs::write(path,&output.riv).unwrap();
    std::fs::write(path.with_extension("map.json"),serde_json::to_vec_pretty(&output.source_map).unwrap()).unwrap();
    std::fs::write(path.with_extension("descriptors.json"),serde_json::to_vec_pretty(&groups).unwrap()).unwrap();
    let whole=proofs(&groups,[E::new(0.,16384.,0.001).unwrap();2],[scalar(0.);2]);
    let large_root_error=proofs(&groups,[E::new(0.,16384.,0.001).unwrap();2],[E::new(1_000_000.,1_000_000.,1.).unwrap();2]);
    let frames:Vec<_>=[(400.,200.),(100.,80.),(1.,1.),(16384.,16384.),(400.,200.)].iter().map(|&(w,h)|serde_json::json!({"width":w,"height":h,"groups":proofs(&groups,[scalar(w),scalar(h)],[scalar(0.);2])})).collect();
    std::fs::write(path.with_extension("domains.json"),serde_json::to_vec_pretty(&serde_json::json!({"whole":whole,"largeRootError":large_root_error,"frames":frames})).unwrap()).unwrap();
}
''')
    (out / 'harness.rs').write_text('\n'.join(parts))
    cargo = run(['cargo', 'build', '--manifest-path', BASE / 'Cargo.toml', '--locked', '--message-format=json'], out / 'cargo.log')
    cargo.check_returncode()
    artifacts = {}; resolved_artifacts = {}
    for line in cargo.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') == 'compiler-artifact':
            for file in message['filenames']:
                if file.endswith(('.rlib', '.so', '.dylib')):
                    resolved_artifacts[file] = sha(file)
                if file.endswith('.rlib'):
                    artifacts[message['target']['name']] = file
    command = ['rustc', '--edition=2024', out / 'harness.rs', '-L', 'dependency=' + str(BASE / 'target/debug/deps'), '-C', 'debuginfo=0', '-o', out / 'harness']
    bindings = {}
    for name in ['nuxie_html_to_riv', 'nuxie_schema', 'scraper', 'selectors', 'cssparser', 'serde_json', 'serde']:
        original = Path(artifacts[name]); target = out / original.name
        shutil.copy2(original, target)
        bindings[str(target)] = sha(target)
        command += ['--extern', name + '=' + str(target)]
    write(out / 'build-command.json', list(map(str, command)))
    run(command, out / 'build.log').check_returncode()
    assert source_hashes() == frozen, 'source changed during build'
    shutil.copy2(__file__, out / 'invoked-driver.py')
    run(['rustc', '-vV'], out / 'rustc-version.txt').check_returncode()
    receipt = {'sourceHashes': frozen, 'directDependencyHashes': bindings, 'resolvedArtifactHashes': resolved_artifacts, 'dependencyLockSha256': sha(BASE / 'Cargo.lock'), 'cargoManifestSha256': sha(BASE / 'Cargo.toml'), 'harnessSourceSha256': sha(out / 'harness.rs'), 'harnessBinarySha256': sha(out / 'harness'), 'scriptSha256': sha(__file__), 'rustcVersionSha256': sha(out / 'rustc-version.txt')}
    write(out / 'build-receipt.json', receipt)
    return receipt


def f32(value):
    return struct.unpack('<f', struct.pack('<f', float(value)))[0]


def validate(cases, out, harness, probe, native):
    rows = []
    for case in cases:
        directory = out / case['name']; directory.mkdir()
        request = directory / 'request.json'
        write(request, {'html': case['html'], 'css': case['css'], 'width': 400, 'height': 200})
        result = run([harness, request, directory / 'scene.riv'], directory / 'compile.log')
        row = {'name': case['name'], 'compileStatus': result.returncode, 'requestSha256': sha(request)}
        rows.append(row)
        if result.returncode:
            row['compileError'] = result.stderr
            continue
        domains = json.loads((directory / 'scene.domains.json').read_text())
        source_map = json.loads((directory / 'scene.map.json').read_text())
        row.update({'whole': domains['whole'], 'largeRootError': domains['largeRootError'], 'exact': domains['frames'], 'rivSha256': sha(directory / 'scene.riv'), 'mapSha256': sha(directory / 'scene.map.json'), 'domainsSha256': sha(directory / 'scene.domains.json'), 'descriptorSha256': sha(directory / 'scene.descriptors.json')})
        if 'target' in case:
            target = next(n['object_id'] for n in source_map if n['id'] == case['target'])
            actual = next(g for g in domains['whole'] if g['parent'] == target)
            row.update({'target': target, 'expectedWhole': case['expectWhole'], 'targetWhole': actual, 'expectationPassed': actual['status'] == case['expectWhole']})
        if not native:
            continue
        run([harness, request, directory / 'repeat.riv'], directory / 'repeat.log').check_returncode()
        row['repeatBytesIdentical'] = sha(directory / 'repeat.riv') == row['rivSha256']
        row['repeatMapIdentical'] = sha(directory / 'repeat.map.json') == row['mapSha256']
        native_out = directory / 'native'
        command = [probe, directory / 'scene.riv', native_out] + [f'{w}x{h}' for w, h in VIEWPORTS]
        observed = run(command, directory / 'probe.log')
        row['nativeStatus'] = observed.returncode
        if observed.returncode:
            row['nativeError'] = observed.stderr
            continue
        assert sha(native_out / 'scene.riv') == row['rivSha256']
        frames = json.loads((native_out / 'frames.json').read_text())['frames']
        assert len(frames) == 2 * len(VIEWPORTS)
        checks = {'whole': 0, 'exact': 0}; failures = []; observations = []
        for frame in frames:
            assert frame['instance'] == frame['frame'] // len(VIEWPORTS)
            assert frame['step'] == frame['frame'] % len(VIEWPORTS)
            assert [frame['width'], frame['height']] == VIEWPORTS[frame['step']]
            geometry = json.loads((native_out / frame['geometry']).read_text())
            actual_nodes = {n['objectId']: n for n in geometry}
            observations.append({'frame': frame, 'geometrySha256': sha(native_out / frame['geometry'])})
            for mode, groups in [('whole', domains['whole']), ('exact', domains['frames'][frame['step']]['groups'])]:
                for group in groups:
                    if group['status'] != 'bounded':
                        continue
                    for node in group['nodes']:
                        actual = actual_nodes[node['id']]
                        matrix = list(map(f32, actual['worldMatrix']))
                        assert matrix[:4] == [1, 0, 0, 1]
                        size = [f32(actual['width']), f32(actual['height'])]
                        origin = matrix[4:6]
                        edge = [f32(a + b) for a, b in zip(size, origin)]
                        for kind, values in [('sizes', size), ('origins', origin), ('edges', edge)]:
                            for axis, value in enumerate(values):
                                bound = node[kind][axis]; checks[mode] += 1
                                if not Q(bound['lower']) - Q(bound['error']) <= Q(value) <= Q(bound['upper']) + Q(bound['error']):
                                    failures.append({'frame': frame['frame'], 'mode': mode, 'parent': group['parent'], 'id': node['id'], 'kind': kind, 'axis': axis, 'native': value, 'bound': bound})
        row.update({'checks': checks, 'failures': failures, 'observations': observations, 'nativeCommand': list(map(str, command))})
        print(case['name'], row.get('targetWhole', {}).get('status'), checks, 'failures', len(failures), flush=True)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--render', action='store_true')
    args = parser.parse_args(); out = args.output.resolve(); out.mkdir(parents=True, exist_ok=True)
    assert not (out / 'build-receipt.json').exists(), 'fresh output or prepared source snapshot required'
    build_receipt = build(out)
    cases_file = BASE / 'validation/flex-proof-bridge-cases.json'
    shutil.copy2(cases_file, out / 'source-cases.json')
    cases = json.loads(cases_file.read_text())
    baseline = BASE / 'output/immutable-baseline-toolchain-r2'
    probe = baseline / 'baseline-probe'; renderer = baseline / 'renderer-replay'
    native_dir = out / 'native'; native_dir.mkdir()
    rows = validate(cases, native_dir, out / 'harness', probe, True)
    write(out / 'native-results.json', rows)
    old = []; source_bindings = {}
    for batch in ['flex-factors-native-r1', 'flex-factors-expanded-r1']:
        path = BASE / 'output' / batch / 'cases.json'
        source_bindings[str(path)] = sha(path)
        old.extend(json.loads(path.read_text()))
    old_dir = out / 'historical'; old_dir.mkdir()
    historical = validate(old, old_dir, out / 'harness', probe, False)
    write(out / 'historical-classifications.json', historical)
    summary = {'scope': 'Private actual-record group proof only; no public admission or whole-scene qualification. Native f32 sizes/origins and derived f32 far edges; derived edges are not rendered pixels.', 'cases': len(rows), 'historicalCases': len(historical), 'newCompileFailures': sum(r['compileStatus'] != 0 for r in rows), 'historicalCompileFailures': sum(r['compileStatus'] != 0 for r in historical), 'expectationFailures': sum(not r.get('expectationPassed', False) for r in rows), 'nativeFailures': sum(r.get('nativeStatus', 1) != 0 for r in rows), 'nativeChecks': {mode: sum(r.get('checks', {}).get(mode, 0) for r in rows) for mode in ['whole', 'exact']}, 'boundFailures': sum(len(r.get('failures', [])) for r in rows), 'determinismFailures': sum(not r.get('repeatBytesIdentical', False) or not r.get('repeatMapIdentical', False) for r in rows), 'historicalBoundedGroups': sum(g['status'] == 'bounded' for r in historical for g in r.get('whole', []))}
    write(out / 'summary.json', summary)
    receipt = {'summary': summary, 'build': build_receipt, 'baselineProbeSha256': sha(probe), 'baselineRendererSha256': sha(renderer), 'casesSha256': sha(cases_file), 'historicalSourceHashes': source_bindings, 'viewportDomain': {'lower': 0, 'upper': 16384, 'error': 0.001}, 'rootOrigins': [0, 0], 'geometryBudget': 0.125, 'viewports': VIEWPORTS, 'nativeResultsSha256': sha(out / 'native-results.json'), 'historicalClassificationsSha256': sha(out / 'historical-classifications.json')}
    assert source_hashes() == build_receipt['sourceHashes'], 'source changed during native validation'
    if args.render:
        selected = [dict(case, viewports=[[400, 200], [100, 80], [600, 320], [400, 200]]) for case, row in zip(cases, rows) if row.get('targetWhole', {}).get('status') == 'bounded']
        render_cases = out / 'render-cases.json'; write(render_cases, selected)
        command = ['node', BASE / 'validation/check-public-baseline.mjs', render_cases, out / 'harness', probe, renderer, out / 'render']
        result = run(command, out / 'render.log')
        receipt['render'] = {'exitCode': result.returncode, 'command': list(map(str, command)), 'scope': 'Shared pixel driver invoked with the PRIVATE Candidate compiler harness, not the public compiler. Driver public-baseline labels are historical and do not imply public admission.', 'receiptSha256': sha(out / 'render/receipt.json') if (out / 'render/receipt.json').exists() else None}
        if (out / 'render/receipt.json').exists():
            rendered = json.loads((out / 'render/receipt.json').read_text())
            receipt['render']['counts'] = {'frames': len(rendered['rows']), 'geometryPass': sum(not r['geometryFailures'] for r in rendered['rows']), 'pixelPass': sum(not r['pixelFailures'] for r in rendered['rows']), 'clearPass': sum(c['samePixels'] for r in rendered['rows'] for c in r['clearChecks'])}
    assert source_hashes() == build_receipt['sourceHashes'], 'source changed during validation'
    write(out / 'receipt.json', receipt)
    print(json.dumps(summary, indent=2))
    assert all(summary[key] == 0 for key in ['newCompileFailures', 'historicalCompileFailures', 'expectationFailures', 'nativeFailures', 'boundFailures', 'determinismFailures']), 'retained validation failures'
    if args.render:
        assert receipt['render']['exitCode'] == 0, 'retained Chrome/native rendering failures (see receipt.json)'


if __name__ == '__main__':
    main()
