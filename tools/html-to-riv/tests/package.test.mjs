import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';

const moduleRoot = fileURLToPath(new URL('../', import.meta.url));

test('packed compiler exports work from an isolated installation', () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'nuxie-compiler-package-'));
  try {
    // The normal prepack hook builds from the checkout. This test runs after the
    // CI WASM build and deliberately skips hooks to avoid an overlapping Cargo build.
    const packed = JSON.parse(execFileSync('npm', ['pack', '--ignore-scripts', '--json', '--pack-destination', workspace],
      {cwd:moduleRoot, encoding:'utf8'}))[0];
    const listed = new Set(packed.files.map(entry => entry.path));
    for (const required of ['package.json', 'js/index.mjs', 'js/index.d.mts', 'dist/html-to-riv.wasm', 'src/reset.css']) {
      assert(listed.has(required), `package is missing exported asset ${required}; run npm run build`);
    }
    assert(![...listed].some(name => name.startsWith('tests/') || name.startsWith('test-results/')));
    const installed = path.join(workspace, 'node_modules', '@nuxie', 'html-to-riv');
    fs.mkdirSync(installed, {recursive:true});
    execFileSync('tar', ['-xzf', path.join(workspace, packed.filename), '--strip-components=1', '-C', installed]);
    fs.writeFileSync(path.join(workspace, 'check.mjs'), `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import {createCompiler, LANGUAGE_VERSION} from '@nuxie/html-to-riv';
      const bytes = fs.readFileSync(new URL(import.meta.resolve('@nuxie/html-to-riv/compiler.wasm')));
      assert(fs.readFileSync(new URL(import.meta.resolve('@nuxie/html-to-riv/reset.css')), 'utf8').length > 0);
      const compiler = await createCompiler(bytes);
      const result = compiler.compile({languageVersion:LANGUAGE_VERSION, html:'<div id="box"></div>',
        css:'div{width:80px;height:40px;background:linear-gradient(red,blue)}', width:120,height:80});
      assert.equal(result.ok, true, JSON.stringify(result));
      assert(result.riv instanceof Uint8Array && result.riv.length > 0);
      assert.equal(result.runtimeRequirements.layout_linear_gradients.length, 1);
    `);
    execFileSync(process.execPath, [path.join(workspace, 'check.mjs')], {cwd:workspace, stdio:'pipe'});
  } finally {
    fs.rmSync(workspace, {recursive:true, force:true});
  }
});
