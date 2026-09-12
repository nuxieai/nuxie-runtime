import assert from 'node:assert/strict';
export const languageVersion='nuxie-html-immutable-v1';
export const SOURCE_LIMIT=1_048_576;
export const control={html:'<div id="box"></div>',css:'#box{width:20px;height:10px;background:teal}',width:240,height:160};
const plain='div{width:8px;height:1px}',padded='div{box-sizing:content-box;width:8px;height:1px;padding:1px}';
export const serialize=(value,escaped=false)=>{
  const text=JSON.stringify(value);
  return escaped?text.replace(/[^\x00-\x7f]/g,char=>'\\u'+char.charCodeAt(0).toString(16).padStart(4,'0')):text;
};
export function stats(request) {
  return {htmlUtf8Bytes:Buffer.byteLength(request.html),cssUtf8Bytes:Buffer.byteLength(request.css),
    sourceUtf8Bytes:Buffer.byteLength(request.html)+Buffer.byteLength(request.css),
    sourceUtf16Units:request.html.length+request.css.length,
    sourceUnicodeScalars:[...request.html].length+[...request.css].length};
}
function comment(bytes,kind,scalar) {
  const [start,end]=kind==='html'?['<!--','-->']:['/*','*/'];
  const available=bytes-start.length-end.length,unit=Buffer.byteLength(scalar);assert(available>=0);
  const result=start+scalar.repeat(Math.floor(available/unit))+'x'.repeat(available%unit)+end;
  assert.equal(Buffer.byteLength(result),bytes);return result;
}
function fill(request,total,htmlFraction,scalar) {
  const extra=total-stats(request).sourceUtf8Bytes;assert(extra>20);
  const htmlBytes=Math.floor(extra*htmlFraction),cssBytes=extra-htmlBytes;
  const result={...request,html:request.html+(htmlBytes?comment(htmlBytes,'html',scalar):''),css:request.css+(cssBytes?comment(cssBytes,'css',scalar):'')};
  assert.equal(stats(result).sourceUtf8Bytes,total);return result;
}
function nested(count) {return Array.from({length:count},(_,i)=>`<div id="n${i}">`).join('')+'</div>'.repeat(count);}
function flat(count) {return Array.from({length:count},(_,i)=>`<div id="n${i}"></div>`).join('');}
function grouped(count) {
  const split=Math.floor(count/2);
  return `<div id="n0">`+Array.from({length:split-1},(_,i)=>`<div id="n${i+1}"></div>`).join('')+'</div>'+
    `<div id="n${split}">`+Array.from({length:count-split-1},(_,i)=>`<div id="n${split+i+1}"></div>`).join('')+'</div>';
}
const structural=(count,shape,css)=>({html:shape==='nested'?nested(count):shape==='grouped'?grouped(count):flat(count),css,width:390,height:160});
export function cases() {
  const rows=[];
  const add=(id,family,request,expect,extra={})=>rows.push({id,family,request,expect,stats:stats(request),...extra});
  add('control','control',control,'success',{controlRequest:control,identities:{shape:'box',count:1}});
  for(const [name,fraction,scalar] of [['html-ascii',1,'a'],['css-ascii',0,'a'],['split-ascii',.5,'a'],['html-utf8-2',1,'é'],['css-utf8-3',0,'雪'],['split-utf8-4',.5,'😀']])for(const delta of [-1,0,1]) {
    const request=fill(control,SOURCE_LIMIT+delta,fraction,scalar);
    add(`source-${name}-${delta+1}`,'source-bytes',request,delta>0?'input-limit':'success',
      {sourceBoundaryOffset:delta,padding:{htmlFraction:fraction,scalar},controlRequest:delta<=0?control:undefined,identities:{shape:'box',count:1}});
  }
  for(const delta of [-1,0,1])add(`source-escaped-json-${delta+1}`,'serialized-vs-source-bytes',fill(control,SOURCE_LIMIT+delta,0,'😀'),delta>0?'input-limit':'success',
    {sourceBoundaryOffset:delta,escapedJson:true,controlRequest:delta<=0?control:undefined,identities:{shape:'box',count:1}});
  for(const [name,css] of [['plain',plain],['content-owner',padded]])for(const depth of [127,128,129]) {
    const request=structural(depth,'nested',css);
    add(`depth-${name}-${depth}`,'nesting',request,depth>128?'depth-limit':'success',
      {depth,identities:{shape:'nested',count:depth},controlRequest:depth<=128?{...request,css:css+(name==='plain'?'div{padding:0}':'/*noop*/')}:undefined});
  }
  for(const [name,shape,css] of [['flat-plain','flat',plain],['grouped-content-owner','grouped',padded]])for(const count of [8191,8192,8193]) {
    const request=structural(count,shape,css);
    add(`objects-${name}-${count}`,'authored-elements',request,count>8192?'object-limit':'success',
      {authoredElements:count,identities:{shape,count},controlRequest:count<=8192?{...request,css:css+(shape==='flat'?'div{padding:0}':'/*noop*/')}:undefined});
  }
  for(const [name,shape,count,sourceDelta,expect] of [
    ['depth-at-both','nested',128,0,'success'],['depth-over-at-source','nested',129,0,'depth-limit'],['depth-and-source-over','nested',129,1,'input-limit'],
    ['objects-at-both','flat',8192,0,'success'],['objects-over-at-source','flat',8193,0,'object-limit'],['objects-and-source-over','flat',8193,1,'input-limit'],
  ]) {
    const original=structural(count,shape,plain);
    add(`combined-${name}`,'combined-limits',fill(original,SOURCE_LIMIT+sourceDelta,.5,'é'),expect,
      {sourceBoundaryOffset:sourceDelta,identities:{shape,count},controlRequest:expect==='success'?original:undefined});
  }
  assert.equal(rows.length,40);assert.equal(new Set(rows.map(row=>row.id)).size,40);
  for(const row of rows) {
    row.serializedCliBytes=Buffer.byteLength(serialize(row.request,row.escapedJson));
    row.serializedAbiBytes=Buffer.byteLength(serialize({languageVersion,input:row.request},row.escapedJson));
    assert(row.serializedAbiBytes<4*1024*1024,'this finite campaign must stay far below the ABI allocation limit');
  }
  return rows;
}
export function identities(map,{shape,count}) {
  assert.equal(map.length,count,'only authored elements have source nodes');
  assert.equal(new Set(map.map(node=>node.object_id)).size,count);
  if(shape==='box'){assert.equal(map[0].id,'box');assert.equal(map[0].path,'/0');return;}
  const split=Math.floor(count/2);
  for(let index=0;index<count;index++) {
    assert.equal(map[index].id,`n${index}`);
    const path=shape==='nested'?'/0'.repeat(index+1):shape==='flat'?`/${index}`:
      index===0?'/0':index<split?`/0/${index-1}`:index===split?'/1':`/1/${index-split-1}`;
    assert.equal(map[index].path,path);
  }
}
