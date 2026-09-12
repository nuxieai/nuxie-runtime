#!/usr/bin/env python3
"""Build a PRIVATE ordinary content-owner experiment from a frozen source copy.

Usage: python3 validation/content-owner-candidate.py NEW_OUTPUT [--packing parent-axis] [--content-bounds]
The resulting `compiler` accepts REQUEST.json OUTPUT.riv like the public CLI.
Only the copied compiler emitter changes. The public library and immutable
runtime are untouched. Existing numeric admission is NOT a proof for this
different graph; this harness is finite experimental evidence only.
"""
from pathlib import Path
import argparse
import difflib
import hashlib
import json
import shutil
import subprocess

BASE = Path(__file__).resolve().parents[1]
ROOT = BASE.parents[1]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def hashes(directory):
    return {str(p.relative_to(directory)): sha(p)
            for p in sorted(directory.rglob('*')) if p.is_file()}


def run(command, log):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True)
    Path(log).write_text(result.stdout + result.stderr)
    result.check_returncode()
    return result


def patch(text, packing, content_bounds):
    def replace(old, new):
        nonlocal text
        assert text.count(old) == 1, ('source patch no longer unique', old)
        text = text.replace(old, new)

    replace('    element: ElementRef<\'a>, object_id: u32, path: String, style: Style,',
            '    element: ElementRef<\'a>, object_id: u32, content_id: u32, path: String, style: Style,')
    replace('self.children(child.element, child.object_id, &child.style,',
            'self.children(child.element, child.content_id, &child.style,')
    replace('    if ![input.width, input.height].into_iter()',
            '    assert!(!capture, "Private content-owner graph has no qualified descriptor certificate");\n'
            '    if ![input.width, input.height].into_iter()')
    replace('        let lowered = box_sizing::lower(&style, &path)?;', '''        // PRIVATE experiment: preserve point/auto content dimensions in an
        // unpainted owner. Percentage owner fields need another composition.
        let content_owner = style.box_sizing == box_sizing::BoxSizing::ContentBox
            && !style.padding.is_zero()
            && ![style.width, style.height, style.min_width, style.min_height,
                style.max_width, style.max_height].iter().any(|v| matches!(v, Size::Percent(_)));
        let lowered = box_sizing::lower(&style, &path)?;''')
    replace('''        let object_id = self.layout_box(&id, authored_parent, style.direction, style.spacing.alignment(style.direction),
            native_parent_direction, sizes, bounds, stretch, authored_margins)?;
        style.padding.emit(&mut self.records[object_id as usize + 2])?;
        style.gap.emit(&mut self.records[object_id as usize + 2])?;''', '''        let outer_direction = if content_owner { Direction::Column } else { style.direction };
        let outer_alignment = if content_owner { Direction::Column.alignment() }
            else { style.spacing.alignment(style.direction) };
        let object_id = self.layout_box(&id, authored_parent, outer_direction, outer_alignment,
            native_parent_direction, sizes, bounds, stretch, authored_margins)?;
        style.padding.emit(&mut self.records[object_id as usize + 2])?;
        if !content_owner { style.gap.emit(&mut self.records[object_id as usize + 2])?; }''')
    replace('''        self.capture_item(descriptor_items, &style, &lowered, parent_style.direction, &id, &path,''', '''        let content_id = if content_owner {
            let inner = self.layout_box("", object_id, style.direction,
                style.spacing.alignment(style.direction), Direction::Column,
                [style.width, style.height],
                [style.min_width, style.min_height, style.max_width, style.max_height],
                true, [false; 4])?;
            style.gap.emit(&mut self.records[inner as usize + 2])?;
            eprintln!("private-content-owner source={} outer={} inner={}", id, object_id, inner);
            inner
        } else { object_id };
        self.capture_item(descriptor_items, &style, &lowered, parent_style.direction, &id, &path,''')
    replace('PreparedChild { element, object_id, path, style, index, order, effective_alignment,',
            'PreparedChild { element, object_id, content_id, path, style, index, order, effective_alignment,')
    if packing == 'parent-axis':
        replace('''        let outer_direction = if content_owner { Direction::Column } else { style.direction };
        let outer_alignment = if content_owner { Direction::Column.alignment() }''', '''        // Preserve the authored participant's parent cross axis so automatic
        // cross sizes stay stretched through the ordinary inner owner.
        let packing_direction = if native_parent_direction.is_row() { Direction::Row } else { Direction::Column };
        let outer_direction = if content_owner { packing_direction } else { style.direction };
        let outer_alignment = if content_owner { packing_direction.alignment() }''')
        replace('style.spacing.alignment(style.direction), Direction::Column,\n                [style.width, style.height],',
                'style.spacing.alignment(style.direction), packing_direction,\n                [style.width, style.height],')
    if content_bounds:
        assert packing == 'parent-axis', 'content-bounds experiment matches parent-axis packing only'
        replace('''        let child_bounds = numeric_bounds.child_with_sizing(&style, &lowered, parent_style, &path)?;''', '''        let outer_content_bounds = numeric_bounds.child_with_sizing(&style, &lowered, parent_style, &path)?;
        let child_bounds = if content_owner {
            // PRIVATE candidate guard: apply ordinary sizing twice, reflecting
            // the two emitted layout owners. Authored content is not the
            // rounded outer dimension minus padding. Automatic cross sizes
            // still inherit the outer owner's available content space.
            let mut inner = style.clone();
            inner.box_sizing = box_sizing::BoxSizing::BorderBox;
            inner.padding = padding::Padding::default();
            inner.flex = flex::Flex::default();
            inner.margins = margins::Margins::default();
            inner.self_alignment = SelfAlignment::AUTO;
            let inner_sizing = box_sizing::lower(&inner, &path)?;
            let packing_parent = Style { direction: if parent_style.direction.is_row() { Direction::Row }
                else { Direction::Column }, ..Style::default() };
            outer_content_bounds.child_with_sizing(&inner, &inner_sizing, &packing_parent, &path)?
        } else { outer_content_bounds };''')
    return text


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--packing', choices=['column', 'parent-axis'], default='column')
    parser.add_argument('--content-bounds', action='store_true')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    original = hashes(BASE / 'src')
    shutil.copytree(BASE / 'src', out / 'source')
    assert hashes(out / 'source') == original
    compiler_source = out / 'source/compiler.rs'
    before = compiler_source.read_text()
    after = patch(before, args.packing, args.content_bounds)
    compiler_source.write_text(after)
    (out / 'compiler.patch').write_text(''.join(difflib.unified_diff(
        before.splitlines(keepends=True), after.splitlines(keepends=True),
        fromfile='public/compiler.rs', tofile='private/compiler.rs')))
    parts = ['use nuxie_html_to_riv::{CompileInput, CompileOutput, Diagnostic, SourceNode};']
    for name in ['color', 'css', 'css_whitespace', 'numeric_tokens', 'variables', 'wire', 'compiler']:
        parts.append(f'#[path="{out / "source" / (name + ".rs")}"] mod {name};')
    parts.append(r'''
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 2, "REQUEST.json OUTPUT.riv");
    let input: CompileInput = serde_json::from_slice(&std::fs::read(&args[0]).unwrap()).unwrap();
    let output = match compiler::compile(&input) {
        Ok(output) => output,
        Err(error) => { eprintln!("{}", serde_json::to_string(&vec![error]).unwrap()); std::process::exit(1); }
    };
    let path = std::path::PathBuf::from(&args[1]);
    std::fs::write(path.with_extension("map.json"), serde_json::to_vec_pretty(&output.source_map).unwrap()).unwrap();
    std::fs::write(path, output.riv).unwrap();
}
''')
    (out / 'harness.rs').write_text('\n'.join(parts))
    cargo_command = ['cargo', 'build', '--manifest-path', BASE / 'Cargo.toml', '--locked', '--message-format=json']
    cargo = run(cargo_command, out / 'cargo.log')
    artifacts = {}
    resolved = {}
    for line in cargo.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') == 'compiler-artifact':
            for file in message['filenames']:
                if file.endswith(('.rlib', '.so', '.dylib')):
                    resolved[file] = sha(file)
                if file.endswith('.rlib'):
                    artifacts[message['target']['name']] = file
    command = ['rustc', '--edition=2024', out / 'harness.rs', '-L', 'dependency=' + str(BASE / 'target/debug/deps'),
               '-C', 'debuginfo=0', '-o', out / 'compiler']
    direct = {}
    for name in ['nuxie_html_to_riv', 'nuxie_schema', 'scraper', 'selectors', 'cssparser', 'serde_json', 'serde']:
        dependency = Path(artifacts[name])
        target = out / dependency.name
        shutil.copy2(dependency, target)
        direct[str(target)] = sha(target)
        command += ['--extern', name + '=' + str(target)]
    run(command, out / 'build.log')
    run(['rustc', '-vV'], out / 'rustc-version.txt')
    assert hashes(BASE / 'src') == original, 'public source changed during build'
    shutil.copy2(__file__, out / 'invoked-driver.py')
    receipt = dict(scope='Private point/auto content-owner composition. No public admission or numerical certificate.',
                   packing=args.packing,
                   contentBounds=args.content_bounds,
                   originalSourceHashes=original, experimentalSourceHashes=hashes(out / 'source'),
                   directDependencyHashes=direct, resolvedArtifactHashes=resolved,
                   compilerSha256=sha(out / 'compiler'), harnessSha256=sha(out / 'harness.rs'),
                   patchSha256=sha(out / 'compiler.patch'), scriptSha256=sha(__file__),
                   cargoManifestSha256=sha(BASE / 'Cargo.toml'), cargoLockSha256=sha(BASE / 'Cargo.lock'),
                   rustcVersionSha256=sha(out / 'rustc-version.txt'),
                   commands=[list(map(str, cargo_command)), list(map(str, command))])
    write(out / 'build-receipt.json', receipt)
    print(json.dumps({'output': str(out), 'compilerSha256': receipt['compilerSha256']}))


if __name__ == '__main__':
    main()
