import { PNG } from 'pngjs';
import pixelmatch from 'pixelmatch';

// Differences caused by independent glyph/curve rasterizers are bounded, not
// silently accepted. Geometry is checked separately at 0.1 CSS px.
export function comparePixels(reference, actual, boxes, text = false) {
  if (reference.width !== actual.width || reference.height !== actual.height) throw new Error('Pixel dimensions differ');
  const {width,height}=reference;
  const diff=new PNG({width,height});
  const mismatched=pixelmatch(reference.data,actual.data,diff.data,width,height,{threshold:0.1,includeAA:!text});
  let sum=0;
  for(let i=0;i<actual.data.length;i++) sum+=Math.abs(actual.data[i]-reference.data[i]);
  const regions={}, interiors={};
  for(const [id,box] of Object.entries(boxes)) {
    let total=0,count=0,innerTotal=0,innerCount=0;
    for(let y=Math.max(0,Math.floor(box.y));y<Math.min(height,Math.ceil(box.y+box.height));y++) {
      for(let x=Math.max(0,Math.floor(box.x));x<Math.min(width,Math.ceil(box.x+box.width));x++) {
        for(let c=0;c<3;c++){
          const i=(y*width+x)*4+c, delta=Math.abs(actual.data[i]-reference.data[i]);
          total+=delta; count++;
          if(x>=Math.ceil(box.x+1) && x<Math.floor(box.x+box.width-1) && y>=Math.ceil(box.y+1) && y<Math.floor(box.y+box.height-1)) {innerTotal+=delta;innerCount++;}
        }
      }
    }
    regions[id]=count ? total/count : 0;
    interiors[id]=innerCount ? innerTotal/innerCount : regions[id];
  }
  const metrics={mismatchedPixels:mismatched,mismatchRatio:mismatched/(width*height),meanChannelError:sum/actual.data.length,regionMeanRgbError:regions,interiorMeanRgbError:interiors,antialiasingExcluded:text};
  const failures=[];
  if(metrics.mismatchRatio>(text ? 0.01 : 0.005)) failures.push('mismatch ratio');
  if(metrics.meanChannelError>1) failures.push('mean channel error');
  // A one-pixel edge band admits half-pixel coverage differences. Interior
  // colors retain the tighter bound; tiny objects never disappear in the
  // whole-page average. Curved corners still count in both local metrics.
  for(const [id,error] of Object.entries(regions)) if(error>10 || interiors[id]>6) failures.push(`${id}: local RGB error`);
  return {metrics,failures,diff};
}

// Presence control for sparse ink that can disappear inside an average-error
// budget. Only fixtures with a known solid background opt in. The regular
// geometry, whole-image and regional error limits continue to apply.
export function compareInkPresence(reference, actual, box, background) {
  if (background.length !== 3 || background.some(v => !Number.isInteger(v) || v < 0 || v > 255)) throw new Error('Invalid ink background');
  const count = image => {
    let ink = 0, left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
    for (let y = Math.max(0, Math.floor(box.y)); y < Math.min(image.height, Math.ceil(box.y + box.height)); y++) {
      for (let x = Math.max(0, Math.floor(box.x)); x < Math.min(image.width, Math.ceil(box.x + box.width)); x++) {
        const i = (y * image.width + x) * 4;
        if (background.reduce((sum, value, c) => sum + Math.abs(image.data[i + c] - value), 0) / 3 > 32) { ink++; left=Math.min(left,x); top=Math.min(top,y); right=Math.max(right,x); bottom=Math.max(bottom,y); }
      }
    }
    return {count:ink,bounds:ink ? [left,top,right,bottom] : null};
  };
  const expected = count(reference), observed = count(actual);
  const aligned = expected.bounds && observed.bounds && expected.bounds.every((v,i)=>Math.abs(v-observed.bounds[i])<=1);
  return { referenceInk:expected.count, actualInk:observed.count, referenceBounds:expected.bounds, actualBounds:observed.bounds,
    passed: Boolean(expected.count > 0 && observed.count >= expected.count / 2 && aligned) };
}

// Saturated red decoration controls isolate underlines from dark glyph ink.
// Require nearby support for every painted pixel in both directions, so missing
// lines and overlong tails cannot hide inside the ordinary whole-image average.
export function compareRedDecoration(reference, actual, box) {
  if(reference.width!==actual.width || reference.height!==actual.height) throw new Error('Pixel dimensions differ');
  const {width,height}=reference;
  const collect=image=>{
    const ink=new Set();
    for(let y=Math.max(0,Math.floor(box.y));y<Math.min(height,Math.ceil(box.y+box.height));y++)
      for(let x=Math.max(0,Math.floor(box.x));x<Math.min(width,Math.ceil(box.x+box.width));x++) {
        const index=y*width+x,i=index*4;
        if(image.data[i]>image.data[i+1]+64 && image.data[i]>image.data[i+2]+64) ink.add(index);
      }
    return ink;
  };
  const expected=collect(reference),observed=collect(actual);
  const unmatched=(a,b)=>{
    let count=0;
    for(const index of a) {
      const x=index%width,y=Math.floor(index/width);
      let found=false;
      for(let dy=-1;dy<=1&&!found;dy++)for(let dx=-1;dx<=1;dx++)
        if(x+dx>=0&&x+dx<width&&y+dy>=0&&y+dy<height&&b.has((y+dy)*width+x+dx)){found=true;break;}
      if(!found)count++;
    }
    return count;
  };
  const missing=unmatched(expected,observed),extra=unmatched(observed,expected);
  return {passed:expected.size>0&&missing===0&&extra===0&&observed.size>=expected.size*.85&&observed.size<=expected.size*1.15,
    referencePixels:expected.size,actualPixels:observed.size,missing,extra};
}
