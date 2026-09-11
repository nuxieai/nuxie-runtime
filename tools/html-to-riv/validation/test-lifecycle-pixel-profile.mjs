import {test} from 'node:test';
import assert from 'node:assert/strict';
import {lifecyclePixelProfile} from './lifecycle-pixel-profile.mjs';
test('authored font selects existing font profile independently of native glyph adapter',()=>{
 for(const nativeGlyphs of [false,true])assert.deepEqual(lifecyclePixelProfile({font:true,nativeGlyphs}),{font:true,name:'font',nativeGlyphs});
});
test('font-free and legacy metadata retain non-font profile',()=>{
 assert.deepEqual(lifecyclePixelProfile({}),{font:false,name:'non-font',nativeGlyphs:null});
 assert.deepEqual(lifecyclePixelProfile({font:false,nativeGlyphs:true}),{font:false,name:'non-font',nativeGlyphs:true});
});
test('malformed classification metadata rejects instead of changing gates',()=>{
 for(const font of ['true',1,null])assert.throws(()=>lifecyclePixelProfile({font}));
 for(const nativeGlyphs of ['true',1,null])assert.throws(()=>lifecyclePixelProfile({nativeGlyphs}));
});
