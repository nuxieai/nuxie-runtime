import test from 'node:test';
import assert from 'node:assert/strict';
import { PNG } from 'pngjs';
import { comparePixels, compareInkPresence } from './pixels.mjs';
import { compareImageContent, fullyContainedPixels, observeImageContent } from './image-content.mjs';

const background = [255, 237, 194];
function canvas() {
  const png = new PNG({ width: 32, height: 32 });
  png.data.fill(255); return png;
}
function paint(png, box, color) {
  for (let y = box.y; y < box.y + box.height; y++) for (let x = box.x; x < box.x + box.width; x++) {
    png.data.set([...color, 255], (y * png.width + x) * 4);
  }
}
function metadata(outer = { x: 2, y: 2, width: 16, height: 16 }, content = { x: 4, y: 4, width: 12, height: 12 }) {
  return { rectangles: { image: outer }, images: { image: { contentBox: content,
    backgroundColor: 'rgb(255, 237, 194)', backgroundImage: 'none', unsupported: [] } } };
}
function pair(metrics = metadata()) {
  const reference = canvas(), actual = canvas();
  const outer = fullyContainedPixels(metrics.rectangles.image, 32, 32);
  for (const image of [reference, actual]) paint(image, outer, background);
  return { reference, actual, metrics };
}
function compare(p) { return compareImageContent(p.reference, p.actual, p.metrics); }

test('fully contained cells exclude fractional edge pixels and retain visible intersection', () => {
  assert.deepEqual(fullyContainedPixels({ x: 2.2, y: 3.4, width: 4.5, height: 6.5 }, 32, 32), { x: 3, y: 4, width: 3, height: 5 });
  assert.deepEqual(fullyContainedPixels({ x: -2, y: 30, width: 40, height: 4 }, 32, 32), { x: 0, y: 30, width: 32, height: 2 });
  assert.throws(() => fullyContainedPixels({ x: NaN, y: 0, width: 2, height: 2 }, 32, 32));
});

test('content presence excludes a following sibling sampled by old ceil border bounds', () => {
  const p = pair(metadata({ x: 2, y: 2, width: 8, height: 10.6 }, { x: 4, y: 4, width: 4, height: 4 }));
  for (const image of [p.reference, p.actual]) paint(image, { x: 4, y: 4, width: 4, height: 4 }, [180, 20, 30]);
  paint(p.reference, { x: 2, y: 12, width: 8, height: 3 }, [0, 0, 0]);
  paint(p.actual, { x: 2, y: 14, width: 8, height: 3 }, [0, 0, 0]);
  p.metrics.rectangles.tail = { x: 2, y: 12.6, width: 8, height: 3 };
  const r = compare(p);
  assert.equal(r.borderBoxImagePresence.image.passed, false);
  assert.deepEqual(r.borderBoxImagePresence.image, compareInkPresence(p.reference, p.actual, p.metrics.rectangles.image, [255, 255, 255]));
  assert.equal(r.imagePresence.image.passed, true);
  assert.deepEqual(r.imagePresence.image.sampledRegion, { x: 4, y: 4, width: 4, height: 4 });
  assert.equal(r.metrics.regionMeanRgbError['image:content'], 0);
  assert(r.failures.includes('tail: local RGB error'));
});

test('missing sparse content fails presence even when whole-frame averaging is small', () => {
  const p = pair(); paint(p.reference, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  const r = compare(p);
  assert.equal(r.imagePresence.image.referenceInk, 1);
  assert.equal(r.imagePresence.image.actualInk, 0);
  assert.equal(r.imagePresence.image.passed, false);
  assert.equal(r.imagePresence.image.failure, 'missing-or-misaligned-content-ink');
});

test('shifted nonempty content fails bounded ink alignment', () => {
  const p = pair();
  paint(p.reference, { x: 5, y: 6, width: 3, height: 3 }, [0, 0, 0]);
  paint(p.actual, { x: 10, y: 6, width: 3, height: 3 }, [0, 0, 0]);
  const r = compare(p);
  assert.equal(r.imagePresence.image.referenceInk, r.imagePresence.image.actualInk);
  assert.equal(r.imagePresence.image.passed, false);
});

test('zero content is explicitly blank, with nonempty padded outer evidence', () => {
  for (const content of [{ x: 6, y: 6, width: 0, height: 8 }, { x: 6, y: 6, width: 8, height: 0 }, { x: 6, y: 6, width: 0, height: 0 }]) {
    const p = pair(metadata(undefined, content)); const r = compare(p);
    assert.equal(r.imagePresence.image.expectation, 'blank-content');
    assert.equal(r.imagePresence.image.passed, true);
    assert.equal(r.imagePresence.image.referenceInk, 0);
    assert.equal(r.imagePresence.image.actualInk, 0);
    assert.equal(r.imagePresence.image.pixelRegion, null);
    assert.equal(r.borderBoxImagePresence.image.passed, false);
    assert.deepEqual(r.failures, []);
  }
});

test('blank-content control retains original threshold32 and ordinary RGB gates', () => {
  const p = pair(metadata(undefined, { x: 6, y: 6, width: 0, height: 0 }));
  paint(p.actual, { x: 2, y: 2, width: 16, height: 16 }, [254, 236, 193]);
  const r = compare(p);
  assert.equal(r.imagePresence.image.passed, true);
  assert.equal(r.metrics.regionMeanRgbError.image, 1);
  assert.deepEqual(r.failures, []);
});

test('unexpected sparse native ink inside a zero-content outer box fails', () => {
  const p = pair(metadata(undefined, { x: 6, y: 6, width: 0, height: 0 }));
  paint(p.actual, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  const r = compare(p);
  assert.equal(r.imagePresence.image.passed, false);
  assert.equal(r.imagePresence.image.actualInk, 1);
  assert.equal(r.imagePresence.image.failure, 'unexpected-ink-in-empty-content');
});

test('a nonblank zero-content reference is an unsupported test condition', () => {
  const p = pair(metadata(undefined, { x: 6, y: 6, width: 0, height: 0 }));
  for (const image of [p.reference, p.actual]) paint(image, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  const r = compare(p).imagePresence.image;
  assert.equal(r.passed, false); assert.equal(r.failure, 'unsupported-control');
  assert(r.unsupported.includes('Zero-content reference is not blank against the known background'));
});

test('ambiguous background, sample area or reference ink never silently passes', () => {
  const blankReference = pair();
  assert.equal(compare(blankReference).imagePresence.image.failure, 'unsupported-control');
  const transparent = pair(); transparent.metrics.images.image.backgroundColor = 'rgba(255, 237, 194, 0.5)';
  assert.equal(compare(transparent).imagePresence.image.failure, 'unsupported-control');
  const tooThin = pair(metadata(undefined, { x: 6.1, y: 6, width: 0.5, height: 8 }));
  assert.equal(compare(tooThin).imagePresence.image.failure, 'unsupported-control');
  const noOuter = pair(metadata({ x: 2, y: 2, width: 0, height: 0 }, { x: 2, y: 2, width: 0, height: 0 }));
  assert.equal(compare(noOuter).imagePresence.image.failure, 'unsupported-control');
});

test('adding content-local checks cannot erase whole, outer or sibling pixel failures', () => {
  const p = pair();
  for (const image of [p.reference, p.actual]) paint(image, { x: 5, y: 5, width: 3, height: 3 }, [0, 0, 0]);
  p.metrics.rectangles.tail = { x: 20, y: 20, width: 8, height: 8 };
  paint(p.reference, p.metrics.rectangles.tail, [0, 0, 0]);
  const before = comparePixels(p.reference, p.actual, p.metrics.rectangles, false), after = compare(p);
  assert(before.failures.includes('tail: local RGB error'));
  assert(before.failures.length > 1);
  assert(before.failures.every(failure => after.failures.includes(failure)));
  for (const name of ['mismatchedPixels', 'mismatchRatio', 'meanChannelError']) assert.equal(after.metrics[name], before.metrics[name]);
  for (const id of Object.keys(p.metrics.rectangles)) {
    assert.equal(after.metrics.regionMeanRgbError[id], before.metrics.regionMeanRgbError[id]);
    assert.equal(after.metrics.interiorMeanRgbError[id], before.metrics.interiorMeanRgbError[id]);
  }
  assert.equal(after.imagePresence.image.passed, true);
});

test('observer uses border-box delivery for empty content and maps padding offset from border origin', async () => {
  const defaults = { borderLeftWidth: '0px', borderTopWidth: '0px', borderRightWidth: '0px', borderBottomWidth: '0px',
    backgroundColor: 'rgb(255, 237, 194)', backgroundImage: 'none', transform: 'none', translate: 'none', rotate: 'none', scale: 'none',
    zoom: '1', opacity: '1', filter: 'none', mixBlendMode: 'normal', overflowX: 'visible', overflowY: 'visible',
    objectFit: 'fill', objectPosition: '50% 50%', imageRendering: 'auto', alignSelf: 'stretch' };
  const outer = { x: 10, y: 20, width: 8, height: 6 };
  const image = { id: 'image', complete: true, naturalWidth: 96, naturalHeight: 64, parentElement: null,
    decode: async () => {}, getBoundingClientRect: () => outer, getClientRects: () => [outer], getAttribute: () => 'picture' };
  const saved = new Map(['document', 'window', 'ResizeObserver', 'getComputedStyle'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  try {
    Object.defineProperty(globalThis, 'document', { configurable: true, value: { images: [image], getElementById: () => image } });
    Object.defineProperty(globalThis, 'window', { configurable: true, value: { scrollX: 0, scrollY: 0, devicePixelRatio: 1 } });
    Object.defineProperty(globalThis, 'getComputedStyle', { configurable: true, value: () => defaults });
    Object.defineProperty(globalThis, 'ResizeObserver', { configurable: true, value: class {
      constructor(callback) { this.callback = callback; }
      observe(target, options) {
        assert.deepEqual(options, { box: 'border-box' });
        queueMicrotask(() => this.callback([{ target, contentRect: { x: 4, y: 3, width: 0, height: 0 }, contentBoxSize: [{}] }]));
      }
      disconnect() {}
    } });
    const metrics = await observeImageContent(['image']);
    assert.deepEqual(metrics.rectangles.image, outer);
    assert.deepEqual(metrics.images.image.contentBox, { x: 14, y: 23, width: 0, height: 0 });
    assert.deepEqual(metrics.images.image.unsupported, []);
    defaults.borderLeftWidth = '2px';
    const bordered = await observeImageContent(['image']);
    assert.equal(bordered.images.image.contentBox.x, 16);
    assert(bordered.images.image.unsupported.includes('Image border paint needs a separate blank-content control'));
    defaults.borderLeftWidth = '0px'; defaults.transform = 'matrix(1, 0, 0, 1, 3, 0)';
    const transformed = await observeImageContent(['image']);
    assert(transformed.images.image.unsupported.some(message => message.includes('Transformed')));
  } finally {
    for (const [key, descriptor] of saved) if (descriptor) Object.defineProperty(globalThis, key, descriptor); else delete globalThis[key];
  }
});

test('prototype-like image ids remain visible to receipt presence accounting', () => {
  const p = pair(); paint(p.reference, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  const original = p.metrics;
  p.metrics = { rectangles: Object.fromEntries([['__proto__', original.rectangles.image]]),
    images: Object.fromEntries([['__proto__', original.images.image]]) };
  const r = compare(p);
  assert.equal(Object.values(r.imagePresence).length, 1);
  assert.equal(r.imagePresence.__proto__.passed, false);
  assert(Object.hasOwn(r.metrics.regionMeanRgbError, '__proto__:content'));
});

test('out-of-outer observations and duplicate local-region names are not accepted', () => {
  const p = pair(metadata(undefined, { x: 4, y: 4, width: 14.01, height: 8 }));
  paint(p.reference, { x: 5, y: 5, width: 1, height: 1 }, [0, 0, 0]);
  assert(compare(p).imagePresence.image.unsupported.includes('Content observation falls outside its outer box'));
  p.metrics.rectangles['image:content'] = { x: 0, y: 0, width: 1, height: 1 };
  assert.throws(() => compare(p), /collides/);
});

function ancestorBackground(p, own, parent) {
  p.metrics.images.image.backgroundColor = own;
  p.metrics.images.image.backgroundChain = [
    { element: 'img', id: 'image', backgroundColor: own, backgroundImage: 'none', borderBox: p.metrics.rectangles.image },
    { element: 'div', id: 'parent', backgroundColor: parent, backgroundImage: 'none', borderBox: { x: 0, y: 0, width: 32, height: 32 } },
  ];
}

test('transparent own background resolves an opaque ancestor and still detects missing sparse ink', () => {
  const p = pair(); ancestorBackground(p, 'rgba(0, 0, 0, 0)', 'rgb(255, 237, 194)');
  paint(p.reference, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  const missing = compare(p).imagePresence.image;
  assert.deepEqual(missing.background.rgb, background);
  assert.equal(missing.background.chain.length, 2);
  assert.deepEqual(missing.unsupported, []);
  assert.equal(missing.actualInk, 0); assert.equal(missing.passed, false);
  paint(p.actual, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]);
  assert.equal(compare(p).imagePresence.image.passed, true);
});

test('translucent own background composes over the recorded parent without hiding a missing image', () => {
  const p = pair(); ancestorBackground(p, 'rgba(200, 100, 50, 0.5)', 'rgb(20, 40, 80)');
  for (const image of [p.reference, p.actual]) paint(image, { x: 2, y: 2, width: 16, height: 16 }, [110, 70, 65]);
  paint(p.reference, { x: 8, y: 8, width: 1, height: 1 }, [255, 255, 255]);
  let r = compare(p).imagePresence.image;
  assert.deepEqual(r.background.rgb, [110, 70, 65]);
  assert.equal(r.referenceInk, 1); assert.equal(r.actualInk, 0); assert.equal(r.passed, false);
  // Independent renderer color rounding can differ by one without becoming ink.
  paint(p.actual, { x: 2, y: 2, width: 16, height: 16 }, [111, 71, 66]);
  paint(p.actual, { x: 8, y: 8, width: 1, height: 1 }, [255, 255, 255]);
  r = compare(p).imagePresence.image;
  assert.equal(r.actualInk, 1); assert.equal(r.passed, true);
});

test('multilayer solid composition stops at an opaque backdrop and rejects partial or nonsolid ancestry', () => {
  const p = pair(); ancestorBackground(p, 'rgba(200, 100, 50, 0.5)', 'rgba(20, 40, 80, 0.5)');
  p.metrics.images.image.backgroundChain.push({ element: 'body', backgroundColor: 'rgb(255, 255, 255)', backgroundImage: 'none', borderBox: { x: 0, y: 0, width: 32, height: 32 } });
  for (const image of [p.reference, p.actual]) { paint(image, { x: 2, y: 2, width: 16, height: 16 }, [169, 124, 109]); paint(image, { x: 8, y: 8, width: 1, height: 1 }, [0, 0, 0]); }
  assert.deepEqual(compare(p).imagePresence.image.background.rgb, [169, 124, 109]);
  assert.equal(compare(p).imagePresence.image.passed, true);
  p.metrics.images.image.backgroundChain[1].borderBox.width = 8;
  assert(compare(p).imagePresence.image.unsupported.some(message => message.includes('does not cover')));
  p.metrics.images.image.backgroundChain[1].borderBox.width = 32;
  p.metrics.images.image.backgroundChain[1].backgroundImage = 'linear-gradient(red, blue)';
  assert(compare(p).imagePresence.image.unsupported.includes('Flat solid background ancestry required'));
});

test('blank-content continues to require own opaque image background', () => {
  const p = pair(metadata(undefined, { x: 6, y: 6, width: 0, height: 0 }));
  ancestorBackground(p, 'rgba(0, 0, 0, 0)', 'rgb(255, 237, 194)');
  const r = compare(p).imagePresence.image;
  assert.equal(r.passed, false);
  assert(r.unsupported.includes('Blank content requires an opaque solid image background'));
});
