import {createCompiler, LANGUAGE_VERSION} from '../js/index.mjs';
import type {Compiler, DesignDocument, ImageAssetInput} from '../js/index.mjs';

const pixels: readonly number[] = [137, 80, 78, 71];
const encoded = new Uint8Array(pixels);
const image: ImageAssetInput = {kind: 'image', bytes: pixels};
const document: DesignDocument = {
  languageVersion: LANGUAGE_VERSION, html: '<img src="logo">', css: '', width: 240, height: 160,
  assets: {logo: image, alternate: {kind: 'image', bytes: encoded}},
};
void createCompiler;
function verify(compiler: Compiler) {
  compiler.compile(document);
  compiler.compile({...document, assets: {}});
  compiler.compile({...document, assets: undefined});
  // @ts-expect-error image source bytes are readonly through the public input type
  image.bytes = encoded;
  // @ts-expect-error no font input is admitted by image assets
  compiler.compile({...document, assets: {font: {kind: 'font', bytes: encoded}}});
  // @ts-expect-error callers supply encoded bytes, not paths
  compiler.compile({...document, assets: {logo: {kind: 'image', path: 'logo.png'}}});
  // @ts-expect-error raw ArrayBuffer is not an image byte vector
  compiler.compile({...document, assets: {logo: {kind: 'image', bytes: new ArrayBuffer(4)}}});
  // @ts-expect-error image dimensions are derived by the compiler
  compiler.compile({...document, assets: {logo: {kind: 'image', bytes: encoded, width: 10}}});
}
void verify;
