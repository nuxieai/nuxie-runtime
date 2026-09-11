// Reuse the same font-aware pixel profile as static and full browser validation.
import assert from 'node:assert/strict';
export function lifecyclePixelProfile(fixture) {
  assert(fixture.font===undefined || typeof fixture.font==='boolean','Invalid fixture font metadata');
  assert(fixture.nativeGlyphs===undefined || typeof fixture.nativeGlyphs==='boolean','Invalid native glyph metadata');
  return {font:fixture.font===true,name:fixture.font===true?'font':'non-font',nativeGlyphs:fixture.nativeGlyphs??null};
}
