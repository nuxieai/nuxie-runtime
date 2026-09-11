import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {renderGallery} from './gallery.mjs';

const directory = process.env.NUXIE_HTML_REVIEW_DIR
  ? path.resolve(process.env.NUXIE_HTML_REVIEW_DIR)
  : path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../test-results');
const read = (file,encoding) => fs.existsSync(file) ? fs.readFileSync(file,encoding) : null;
const json = file => {const content=read(file,'utf8');return content ? JSON.parse(content) : null;};

export default class GalleryReporter {
  cases = [];
  checks = [];
  errors = [];
  onBegin(config,suite) {
    this.expected = suite.allTests().length;
    this.command = config.argv ?? process.argv;
    this.started = new Date().toISOString();
    this.shapingPrecision = process.env.NUXIE_CSS_SHAPING_PRECISION ? 'css-experimental' : 'manifest-selected';
  }
  onError(error) { this.errors.push(error.message ?? String(error)); }
  onTestEnd(test,result) {
    const attachment = result.attachments.find(a => a.name === 'visual-review');
    const check = {name:test.title,project:test.parent.project()?.name,status:result.status,errors:result.errors.map(e => e.message ?? String(e))};
    this.checks.push(check);
    if (!attachment?.path) return;
    const manifest = json(attachment.path);
    const prefix = manifest.artifactPrefix;
    const images = {};
    for (const kind of ['browser','native','diff']) {
      const bytes=read(`${prefix}.${kind}.png`);
      images[kind] = bytes ? `data:image/png;base64,${bytes.toString('base64')}` : null;
    }
    const requirements=json(`${prefix}.requirements.json`);
    const shapingPrecision=process.env.NUXIE_CSS_SHAPING_PRECISION || requirements?.capabilities?.includes('text-css-shaping-precision-v1') ? 'css' : 'rive';
    this.cases.push({...manifest,shapingPrecision,status:check.status,project:check.project,errors:check.errors,testName:check.name,images,metrics:json(`${prefix}.metrics.json`),browserBounds:json(`${prefix}.browser.json`),nativeBounds:json(`${prefix}.bounds.json`)});
  }
  onEnd(result) {
    const renderingProfile = (process.env.NUXIE_NATIVE_GLYPHS === '1' ? 'macOS CoreText (experimental)' : 'vector') + ' / checked runtime requirements';
    const report = {shapingPrecision:this.shapingPrecision,renderingProfile,status:result.status,started:this.started,finished:new Date().toISOString(),expected:this.expected,command:this.command,errors:this.errors,checks:this.checks,cases:this.cases};
    fs.mkdirSync(directory,{recursive:true});
    fs.writeFileSync(path.join(directory,'gallery.html'),renderGallery(report));
    // Keep machine-readable review metadata without duplicating embedded PNGs.
    fs.writeFileSync(path.join(directory,'review.json'),JSON.stringify({...report,cases:report.cases.map(({images,...record}) => ({...record,imagesAvailable:Object.fromEntries(Object.entries(images).map(([key,value]) => [key,!!value]))}))},null,2));
    console.log(`Visual review: ${path.join(directory,'gallery.html')}`);
  }
}
