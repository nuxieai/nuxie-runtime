import {test,expect} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels,compareInkPresence,compareRedDecoration} from './pixels.mjs';

function picture(dx=0,color=[30,70,160,255],missing=false) {
  const png=new PNG({width:200,height:200}); png.data.fill(255);
  if(!missing) for(let y=20;y<28;y++) for(let x=20+dx;x<28+dx;x++) png.data.set(color,(y*200+x)*4);
  return png;
}
const boxes={small:{x:20,y:20,width:8,height:8}};
test('pixel gate accepts identical independently supplied images',()=>{
  expect(comparePixels(picture(),picture(),boxes).failures).toEqual([]);
});
for(const text of [false,true]) {
  test(`local gate catches a missing small object even on a mostly empty page (text=${text})`,()=>{
    expect(comparePixels(picture(),picture(0,undefined,true),boxes,text).failures).toContain('small: local RGB error');
  });
  test(`pixel gate catches displaced content (text=${text})`,()=>{
    expect(comparePixels(picture(),picture(3),boxes,text).failures.length).toBeGreaterThan(0);
  });
  test(`pixel gate catches incorrect color (text=${text})`,()=>{
    expect(comparePixels(picture(),picture(0,[240,150,30,255]),boxes,text).failures.length).toBeGreaterThan(0);
  });
}
test('pixel dimensions must agree',()=>{
  expect(()=>comparePixels(picture(),new PNG({width:100,height:100}),boxes)).toThrow('dimensions');
});

test('sparse ink presence rejects a missing mark despite low average error',()=>{
  const reference=picture(0,undefined,true), missing=picture(0,undefined,true);
  for(let x=20;x<28;x++) reference.data.set([90,110,150,255],(24*200+x)*4);
  const box={x:20,y:20,width:8,height:30};
  expect(comparePixels(reference,missing,{mark:box},true).failures).toEqual([]);
  expect(compareInkPresence(reference,missing,box,[255,255,255]).passed).toBe(false);
  expect(compareInkPresence(reference,reference,box,[255,255,255]).passed).toBe(true);
  expect(compareInkPresence(missing,missing,box,[255,255,255]).passed).toBe(false);
  const shifted=picture(0,undefined,true);
  for(let x=24;x<32;x++) shifted.data.set([90,110,150,255],(24*200+x)*4);
  expect(compareInkPresence(reference,shifted,box,[255,255,255]).passed).toBe(false);
});


test('red decoration gate catches sparse missing, displaced and overlong lines',()=>{
  const blank=()=>picture(0,undefined,true);
  const draw=(image,start,end,y)=>{for(let x=start;x<end;x++)image.data.set([255,0,0,255],(y*200+x)*4);return image;};
  const reference=draw(blank(),20,45,30),box={x:10,y:10,width:100,height:80};
  const missing=blank(),shifted=draw(blank(),20,45,33),long=draw(blank(),20,60,30);
  expect(comparePixels(reference,missing,{a:box},true).failures).toEqual([]);
  expect(compareRedDecoration(reference,reference,box).passed).toBe(true);
  for(const actual of [missing,shifted,long])expect(compareRedDecoration(reference,actual,box).passed).toBe(false);
  expect(compareRedDecoration(missing,missing,box).passed).toBe(false);
  const two=draw(draw(blank(),20,45,30),20,45,50);
  expect(compareRedDecoration(two,reference,box).passed).toBe(false);
});
