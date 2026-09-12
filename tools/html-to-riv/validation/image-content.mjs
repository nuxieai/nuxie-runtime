// Validation only: source observations and pixel controls never affect Rive bytes.
import { comparePixels, compareInkPresence } from './pixels.mjs';

// This function is self-contained so Playwright can serialize it into Chrome.
// ResizeObserver contentRect's x/y are padding offsets from the padding origin:
// https://www.w3.org/TR/resize-observer/#content-rect
export async function observeImageContent(ids) {
  const elements = [...document.images];
  await Promise.all(elements.map(image => image.decode()));
  const observed = await new Promise((resolve, reject) => {
    const entries = new Map();
    if (!elements.length) { resolve(entries); return; }
    const timer = setTimeout(() => { observer.disconnect(); reject(new Error('Image content-box observation timed out')); }, 2000);
    const observer = new ResizeObserver(changes => {
      for (const entry of changes) entries.set(entry.target, entry);
      if (entries.size === elements.length) { clearTimeout(timer); observer.disconnect(); resolve(entries); }
    });
    // A padded image can have a zero-by-zero content box. Observing its nonzero
    // border box still delivers the entry containing that empty contentRect.
    for (const image of elements) observer.observe(image, { box: 'border-box' });
  });
  const rectangles = Object.create(null), images = Object.create(null);
  const rect = r => ({ x: r.x, y: r.y, width: r.width, height: r.height });
  for (const id of ids) {
    const element = document.getElementById(id);
    if (!element) throw new Error(`Missing observed source id ${id}`);
    rectangles[id] = rect(element.getBoundingClientRect());
  }
  for (const image of elements) {
    if (!image.id || !Object.hasOwn(rectangles, image.id)) throw new Error('Every fixture image must have an observed authored id');
    const style = getComputedStyle(image), border = rectangles[image.id], entry = observed.get(image);
    const contentRect = rect(entry.contentRect);
    const borderWidths = [style.borderLeftWidth, style.borderTopWidth, style.borderRightWidth, style.borderBottomWidth].map(parseFloat);
    const unsupported = [], backgroundChain = [];
    if (window.scrollX !== 0 || window.scrollY !== 0) unsupported.push('Scrolled document requires a separate screenshot coordinate mapping');
    if (window.devicePixelRatio !== 1) unsupported.push('Image content sampling requires deviceScaleFactor1');
    if (image.getClientRects().length !== 1 || entry.contentBoxSize.length !== 1) unsupported.push('Fragmented image boxes need separate observation');
    if (style.backgroundImage !== 'none') unsupported.push('Image background is not a known solid color');
    if (borderWidths.some(value => !Number.isFinite(value) || value !== 0)) unsupported.push('Image border paint needs a separate blank-content control');
    for (let ancestor = image; ancestor; ancestor = ancestor.parentElement) {
      const s = getComputedStyle(ancestor);
      backgroundChain.push({ element: ancestor.localName ?? null, id: ancestor.id ?? '',
        backgroundColor: s.backgroundColor, backgroundImage: s.backgroundImage,
        borderBox: rect(ancestor.getBoundingClientRect()) });
      if (s.backgroundImage !== 'none') { unsupported.push('Image background ancestry contains nonsolid paint'); break; }
      if (s.transform !== 'none' || ['translate', 'rotate', 'scale'].some(key => s[key] && s[key] !== 'none') || !['normal', '1'].includes(s.zoom)) {
        unsupported.push('Transformed or zoomed image ancestry cannot use border-origin content coordinates'); break;
      }
      if (s.opacity !== '1' || s.filter !== 'none' || s.mixBlendMode !== 'normal') {
        unsupported.push('Image compositing requires a separate background/presence control'); break;
      }
      if (ancestor !== image && (s.overflowX !== 'visible' || s.overflowY !== 'visible')) {
        unsupported.push('Clipped image ancestry requires a separate visible-content control'); break;
      }
    }
    images[image.id] = {
      src: image.getAttribute('src'), complete: image.complete, naturalWidth: image.naturalWidth,
      naturalHeight: image.naturalHeight, objectFit: style.objectFit, objectPosition: style.objectPosition,
      imageRendering: style.imageRendering, alignSelf: style.alignSelf,
      contentRect, contentBox: { x: border.x + borderWidths[0] + contentRect.x,
        y: border.y + borderWidths[1] + contentRect.y, width: contentRect.width, height: contentRect.height },
      contentObservation: { method: 'ResizeObserver.contentRect', observedBox: 'border-box',
        coordinateMapping: 'unscrolled border origin + border inset + contentRect padding offset', borderWidths },
      backgroundColor: style.backgroundColor, backgroundImage: style.backgroundImage, backgroundChain, unsupported,
    };
  }
  return { rectangles, images };
}

function validRect(box) {
  return box && ['x', 'y', 'width', 'height'].every(key => Number.isFinite(box[key])) && box.width >= 0 && box.height >= 0;
}
export function fullyContainedPixels(box, width, height) {
  if (!validRect(box) || !Number.isInteger(width) || !Number.isInteger(height) || width <= 0 || height <= 0) throw new Error('Invalid image sampling geometry');
  const x = Math.max(0, Math.min(width, Math.ceil(box.x))), y = Math.max(0, Math.min(height, Math.ceil(box.y)));
  const right = Math.max(x, Math.min(width, Math.floor(box.x + box.width)));
  const bottom = Math.max(y, Math.min(height, Math.floor(box.y + box.height)));
  return { x, y, width: right - x, height: bottom - y };
}
function solidColor(css) {
  const match = /^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*(1(?:\.0*)?|0(?:\.\d+)?))?\s*\)$/.exec(css ?? '');
  if (!match) return null;
  const values = match.slice(1, 4).map(Number);
  return values.every(value => value >= 0 && value <= 255) ? { rgb: values, alpha: match[4] === undefined ? 1 : Number(match[4]) } : null;
}
function contentBackground(image, region, empty) {
  const result = { css: image.backgroundColor, rgb: null, source: empty ? 'computed opaque solid image background' : 'source-over solid background ancestry',
    chain: [], unsupported: [] };
  const own = solidColor(image.backgroundColor);
  if (empty) {
    if (image.backgroundImage !== 'none' || !own || own.alpha !== 1) result.unsupported.push('Blank content requires an opaque solid image background');
    else { result.rgb = own.rgb; result.chain = [{ css: image.backgroundColor, ...own }]; }
    return result;
  }
  const chain = image.backgroundChain ?? [{ backgroundColor: image.backgroundColor, backgroundImage: image.backgroundImage }];
  if (chain[0]?.backgroundColor !== image.backgroundColor) { result.unsupported.push('Background chain does not start with the observed image background'); return result; }
  let remaining = 1; const rgb = [0, 0, 0];
  for (const [index, layer] of chain.entries()) {
    const color = solidColor(layer.backgroundColor);
    if (!color || layer.backgroundImage !== 'none') { result.unsupported.push('Flat solid background ancestry required'); break; }
    result.chain.push({ css: layer.backgroundColor, ...color, element: layer.element ?? null, id: layer.id ?? null, borderBox: layer.borderBox ?? null });
    if (color.alpha > 0) {
      // The image's own containment is checked separately. An ancestor that
      // paints only part of the sample cannot provide one constant backdrop.
      if (index > 0 && (!validRect(layer.borderBox) || layer.borderBox.x > region.x || layer.borderBox.y > region.y
          || layer.borderBox.x + layer.borderBox.width < region.x + region.width
          || layer.borderBox.y + layer.borderBox.height < region.y + region.height)) {
        result.unsupported.push('A contributing ancestor background does not cover all sampled content pixels'); break;
      }
      for (let channel = 0; channel < 3; channel++) rgb[channel] += remaining * color.alpha * color.rgb[channel];
      remaining *= 1 - color.alpha;
    }
    if (remaining === 0) { result.rgb = rgb.map(Math.round); break; }
  }
  if (!result.rgb && !result.unsupported.length) result.unsupported.push('Background ancestry has no established opaque backdrop');
  result.computation = 'Front-to-back source-over of computed sRGB channels, rounded once to integer RGB; original ink threshold and RGB gates retained';
  return result;
}
function opaqueRegion(image, region) {
  for (let y = region.y; y < region.y + region.height; y++) for (let x = region.x; x < region.x + region.width; x++) {
    if (image.data[(y * image.width + x) * 4 + 3] !== 255) return false;
  }
  return true;
}

// Full, outer and sibling RGB gates are evaluated unchanged; only additional
// named content regions are supplied to the same comparePixels implementation.
export function compareImageContent(reference, actual, metrics) {
  if (reference.width !== actual.width || reference.height !== actual.height) throw new Error('Pixel dimensions differ');
  const regions = { ...metrics.rectangles }, imagePresence = Object.create(null), borderBoxImagePresence = Object.create(null);
  for (const [id, image] of Object.entries(metrics.images)) {
    const outer = metrics.rectangles[id], content = image.contentBox;
    if (!validRect(outer) || !validRect(content)) throw new Error(`Invalid observed image boxes: ${id}`);
    // Preserve the former control exactly, including its white background and
    // floor/ceil border-box sampling. It is evidence, not the content verdict.
    borderBoxImagePresence[id] = compareInkPresence(reference, actual, outer, [255, 255, 255]);
    const empty = content.width === 0 || content.height === 0;
    const region = fullyContainedPixels(empty ? outer : content, reference.width, reference.height);
    const background = contentBackground(image, region, empty);
    const control = { expectation: empty ? 'blank-content' : 'nonempty-ink',
      observedContentBox: content, outerBox: outer, sampledRegion: region,
      sampling: empty ? 'fully-contained outer pixel cells; zero content axis' : 'fully-contained content pixel cells',
      background,
      viewportClipped: (empty ? outer : content).x < 0 || (empty ? outer : content).y < 0
        || (empty ? outer : content).x + (empty ? outer : content).width > reference.width
        || (empty ? outer : content).y + (empty ? outer : content).height > reference.height,
      inkThresholdMeanRgbExclusive: 32, unsupported: [...(image.unsupported ?? [])], passed: false };
    control.unsupported.push(...background.unsupported);
    if (region.width === 0 || region.height === 0) control.unsupported.push('No fully contained visible pixels for this content expectation');
    if (content.x < outer.x || content.y < outer.y || content.x + content.width > outer.x + outer.width || content.y + content.height > outer.y + outer.height) {
      control.unsupported.push('Content observation falls outside its outer box');
    }
    if (!empty && region.width > 0 && region.height > 0) {
      const key = `${id}:content`;
      if (Object.hasOwn(regions, key)) throw new Error(`Image content region name collides: ${key}`);
      regions[key] = region; control.pixelRegion = key;
    } else control.pixelRegion = null;
    if (!control.unsupported.length) {
      if (!opaqueRegion(reference, region) || !opaqueRegion(actual, region)) control.unsupported.push('Presence sampling requires opaque screenshot pixels');
      const ink = compareInkPresence(reference, actual, region, background.rgb);
      Object.assign(control, { referenceInk: ink.referenceInk, actualInk: ink.actualInk,
        referenceBounds: ink.referenceBounds, actualBounds: ink.actualBounds });
      if (empty) {
        if (ink.referenceInk !== 0) control.unsupported.push('Zero-content reference is not blank against the known background');
        control.passed = control.unsupported.length === 0 && ink.actualInk === 0;
        control.failure = control.unsupported.length ? 'unsupported-control' : ink.actualInk ? 'unexpected-ink-in-empty-content' : null;
      } else {
        if (ink.referenceInk === 0) control.unsupported.push('Nonempty reference has no distinguishable ink for this control');
        control.passed = control.unsupported.length === 0 && ink.passed;
        control.failure = control.unsupported.length ? 'unsupported-control' : ink.passed ? null : 'missing-or-misaligned-content-ink';
      }
    } else control.failure = 'unsupported-control';
    imagePresence[id] = control;
  }
  const compared = comparePixels(reference, actual, regions, false);
  return { ...compared, imagePresence, borderBoxImagePresence, pixelRegions: regions };
}
