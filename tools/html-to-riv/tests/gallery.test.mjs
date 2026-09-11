import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';

const moduleDir=fileURLToPath(new URL('../',import.meta.url));
for(const lane of ['missing-renderer','geometry']) {
  test(`review artifact faithfully reports ${lane}`,()=>{
    const directory=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-review-'));
    try {
      const reportDir=path.join(directory,'review');
      const command=lane==='geometry'?'test:geometry':'test';
      const result=spawnSync('npm',['run',command,'--','--grep','fixed-box at 240px$','--output',path.join(directory,'artifacts')],{
        cwd:moduleDir,encoding:'utf8',maxBuffer:8*1024*1024,
        env:{...process.env,NUXIE_HTML_REVIEW_DIR:reportDir,NUXIE_HTML_RENDERER:path.join(directory,'missing-renderer')},
      });
      assert.equal(result.status,lane==='geometry'?0:1,result.stdout+result.stderr);
      const report=JSON.parse(fs.readFileSync(path.join(reportDir,'review.json')));
      assert.equal(report.expected,1);
      assert.equal(report.cases.length,1);
      const entry=report.cases[0];
      assert.equal(entry.imagesAvailable.native,false);
      assert.equal(entry.imagesAvailable.diff,false);
      assert.equal(entry.metrics,null);
      assert.equal(entry.imagesAvailable.browser,lane==='geometry');
      assert.equal(entry.pixels,lane!=='geometry');
      assert.equal(report.status,lane==='geometry'?'passed':'failed');
      assert.equal(entry.status,lane==='geometry'?'passed':'failed');
      const html=fs.readFileSync(path.join(reportDir,'gallery.html'),'utf8');
      assert.ok(html.includes('not visually validated'));
      assert.ok(html.includes('Artifact unavailable'));
      assert.ok(!html.includes('/* REPORT_DATA */'));
    } finally {fs.rmSync(directory,{recursive:true,force:true});}
  });
}
