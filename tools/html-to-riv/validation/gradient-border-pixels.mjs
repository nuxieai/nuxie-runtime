// Local border-repeat sentinel. Uses measured Chrome geometry, never native bounds.
// Pixel objects have the pngjs shape {width,height,data}; positions use CSS px at DPR1.
import assert from 'node:assert/strict';
export function compareGradientBorderPixels(browser,native,{viewport,box,borderLeftWidth,borderRightWidth}) {
  for(const image of [browser,native]) {
    assert.equal(image.width,viewport.width);assert.equal(image.height,viewport.height);
    assert.equal(image.data.length,image.width*image.height*4);
  }
  for(const axis of ['x','y','width','height'])assert(Number.isFinite(box?.[axis]),`Missing browser ${axis}`);
  assert(box.width>0 && box.height>0);
  assert.equal(borderLeftWidth,1);assert.equal(borderRightWidth,1);
  const xs=[Math.floor(box.x+.5),Math.floor(box.x+box.width+.5)-1],samples=[];
  const pixel=(image,x,y)=>Array.from(image.data.subarray((y*image.width+x)*4,(y*image.width+x)*4+4));
  for(const [index,x] of xs.entries())for(const fraction of [.25,.5,.75]) {
    const y=Math.floor(box.y+box.height*fraction);
    assert(x>=0 && x<viewport.width && y>=0 && y<viewport.height,'Border sample outside viewport');
    const expected=pixel(browser,x,y),actual=pixel(native,x,y);
    const maxChannelError=Math.max(...expected.map((channel,i)=>Math.abs(channel-actual[i])));
    samples.push({side:index===0?'left':'right',x,y,browser:expected,actual,maxChannelError,passed:maxChannelError<=2});
  }
  return {passed:samples.every(sample=>sample.passed),samples};
}
