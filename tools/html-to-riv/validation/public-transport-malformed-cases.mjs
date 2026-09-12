// Fixed, reproducible transport cases. Expectations come from the named-field
// object contract, Rust/JS validation boundaries, and explicit ABI operations.
export const languageVersion='nuxie-html-immutable-v1';
export const control={html:'<div id="box"></div>',css:'#box{width:20px;height:10px;background:teal}',width:240,height:160};
const text=JSON.stringify(control),head=`{"languageVersion":${JSON.stringify(languageVersion)},"input":`;
const hex=value=>Buffer.isBuffer(value)?value.toString('hex'):Buffer.from(value).toString('hex');
export function cases() {
  const rows=[];
  function input(id,raw,expect='invalid-request',options={}) {
    const bytes=Buffer.isBuffer(raw)?raw:Buffer.from(raw);
    let js;
    try {JSON.parse(bytes.toString('utf8'));if(!options.invalidUtf8)js={kind:'input-json',text:bytes.toString('utf8')};}catch{}
    rows.push({id,family:options.family??'input-schema',cliHex:hex(bytes),abiHex:hex(Buffer.concat([Buffer.from(head),bytes,Buffer.from('}')])),
      ...(js?{js}:{}),expect:{cli:expect,abi:expect,...(js?{js:options.jsExpect??expect}:{})},
      comparison:options.comparison??'diagnostic-class',note:options.note??null});
  }
  input('valid-object',text,'success',{comparison:'exact',family:'control'});
  input('valid-pretty-object',JSON.stringify(control,null,2),'success',{comparison:'exact',family:'control'});
  input('valid-escaped-key',text.replace('"html"','"\\u0068tml"'),'success',{comparison:'exact',family:'control'});
  for(let position=0;position<Buffer.byteLength(text);position++)input(`json-truncate-${String(position).padStart(3,'0')}`,Buffer.from(text).subarray(0,position),'invalid-request',{family:'json-syntax'});
  for(const [name,raw] of [
    ['trailing-comma',text.slice(0,-1)+',}'],['trailing-object',text+'{}'],['trailing-null',text+'null'],
    ['comment-prefix','/*comment*/'+text],['comment-suffix',text+'//comment'],['single-quotes',text.replace('"html"',"'html'")],
    ['unquoted-key',text.replace('"html"','html')],['missing-colon',text.replace('"html":','"html"')],
    ['bom','\ufeff'+text],['nul-prefix','\0'+text],['nbsp-prefix','\u00a0'+text],
    ['bad-escape',text.replace('teal','\\q')],['unfinished-unicode',text.replace('teal','\\u12')],
    ['bad-hex-escape',text.replace('teal','\\uGGGG')],['literal-newline',text.replace('teal','te\nal')],
    ['nan',text.replace('240','NaN')],['infinity',text.replace('240','Infinity')],['leading-zero',text.replace('240','0240')],
    ['leading-plus',text.replace('240','+240')],['unfinished-exponent',text.replace('240','2e+')],
  ])input(`json-${name}`,raw,'invalid-request',{family:'json-syntax'});
  for(const field of Object.keys(control)) {
    const { [field]:removed,...rest}=control;input(`missing-${field}`,JSON.stringify(rest));
    const values=typeof control[field]==='string'?[null,true,false,0,240,[],{}]:[null,true,false,'240','',[],{}];
    for(const [index,value] of values.entries())input(`type-${field}-${index}`,JSON.stringify({...control,[field]:value}));
    const duplicate=text.slice(0,-1)+','+JSON.stringify(field)+':'+JSON.stringify(control[field])+'}';
    input(`duplicate-${field}`,duplicate,'invalid-request',{jsExpect:'success',family:'duplicate-key',
      note:'serde rejects duplicate input fields; JSON.parse used by this JS recipe has already collapsed identical duplicate keys.'});
  }
  for(const name of ['assets','runtimeRequirements','input','html2','HTML','widthUnits','__proto__','constructor'])
    input(`unknown-${name}`,text.slice(0,-1)+','+JSON.stringify(name)+':{}}');
  input('positional-input-array',JSON.stringify(Object.values(control)),'invalid-request',{family:'object-contract',
    note:'The public input uses named html/css/width/height fields; positional arrays must not be admitted.'});
  for(const [name,value] of [['null',null],['array',[]],['number',42],['boolean',true],['string','input']])input(`root-${name}`,JSON.stringify(value));
  for(const [name,value] of [['zero',0],['negative',-1],['too-large',16385],['negative-zero',-0],['f32-overflow',1e39],['f32-underflow',1e-46]])
    input(`viewport-${name}`,JSON.stringify({...control,width:value}),'invalid-viewport',{family:'numeric-transport',comparison:'exact'});
  for(const [name,raw] of [['f64-overflow','1e400'],['huge-negative','-1e400']])
    input(`number-${name}`,text.replace('240',raw),'invalid-request',{family:'numeric-transport'});
  for(const [name,escape] of [['high','\\ud800'],['low','\\udfff'],['mismatched','\\ud800\\u0041']])
    input(`surrogate-${name}`,text.replace('teal',escape),'invalid-request',{family:'json-unicode'});
  for(const [name,bytes] of [['invalid-lead',[255]],['continuation',[128]],['overlong',[192,175]],['truncated',[194]],['encoded-surrogate',[237,160,128]],['above-scalar-range',[244,144,128,128]]]) {
    const split=text.indexOf('teal');input(`utf8-${name}`,Buffer.concat([Buffer.from(text.slice(0,split)),Buffer.from(bytes),Buffer.from(text.slice(split+4))]),'invalid-request',{family:'invalid-utf8',invalidUtf8:true});
  }
  function envelope(id,value,expect,js,jsExpect=expect,note=null) {
    rows.push({id,family:'abi-envelope',abiHex:hex(typeof value==='string'?value:JSON.stringify(value)),...(js?{js}:{}),
      expect:{abi:expect,...(js?{js:jsExpect}:{})},comparison:'declared-contract',note});
  }
  envelope('envelope-missing-language',{input:control},'invalid-request',{kind:'document-json',text:JSON.stringify(control)},'unsupported-language-version','JS checks its language field before ABI deserialization.');
  envelope('envelope-missing-input',{languageVersion},'invalid-request');
  for(const [name,version] of [['old','nuxie-html-v1'],['empty',''],['null',null],['number',2],['boolean',true],['array',[]],['object',{}]]) {
    const expected=typeof version==='string'?'unsupported-language-version':'invalid-request';
    envelope(`envelope-language-${name}`,{languageVersion:version,input:control},expected,{kind:'document-json',text:JSON.stringify({...control,languageVersion:version})},'unsupported-language-version','JS rejects any nonmatching version before typed ABI request parsing.');
  }
  for(const name of ['assets','runtimeRequirements','extra'])envelope(`envelope-unknown-${name}`,{languageVersion,input:control,[name]:{}},'invalid-request');
  envelope('envelope-positional-array',[languageVersion,control],'invalid-request',null,undefined,'The ABI Request is a named-field object, not a positional array.');
  envelope('envelope-and-input-positional-arrays',[languageVersion,Object.values(control)],'invalid-request');
  envelope('envelope-duplicate-language',`{"languageVersion":${JSON.stringify(languageVersion)},"languageVersion":${JSON.stringify(languageVersion)},"input":${text}}`,'invalid-request');
  envelope('envelope-duplicate-input',`{"languageVersion":${JSON.stringify(languageVersion)},"input":${text},"input":${text}}`,'invalid-request');
  envelope('envelope-empty-bytes','','invalid-request');
  for(const name of ['undefined','null','array','boolean','number','string','bigint','symbol','function','width-nan','width-infinity','height-negative-infinity','width-bigint','width-boxed','html-boxed','html-undefined','unknown-cyclic','getter-error','getter-string','proxy-keys-error','revoked-proxy','inherited-input'])
    rows.push({id:`js-${name}`,family:'js-values',js:{kind:'special',name},expect:{js:'invalid-request'},comparison:'js-only'});
  for(const name of ['date','boxed-document','missing-version'])rows.push({id:`js-${name}`,family:'js-values',js:{kind:'special',name},expect:{js:'unsupported-language-version'},comparison:'js-only'});
  for(const name of ['frozen-document','null-prototype','inherited-version'])rows.push({id:`js-${name}`,family:'js-values',js:{kind:'special',name},expect:{js:'success'},comparison:'js-only',note:'Fields retained by the public wrapper satisfy its object/spread contract.'});
  rows.push({id:'js-getter-unprintable-throw',family:'js-host-error',js:{kind:'special',name:'getter-unprintable-throw'},expect:{js:'invalid-request'},comparison:'js-only',note:'An unreadable request getter throws a null-prototype value; public validation catch should remain a structured failure.'});
  for(const [name,length] of [['zero',0],['over-limit',192*1024*1024+1],['u32-max',0xffffffff]])
    rows.push({id:`abi-allocate-${name}`,family:'abi-protocol',abiOperation:{kind:'allocate-rejected',length},expect:{abi:'invalid-request'},comparison:'abi-only'});
  for(const kind of ['compile-without-request','compile-twice','reset-twice'])rows.push({id:`abi-${kind}`,family:'abi-protocol',abiOperation:{kind},expect:{abi:'invalid-request'},comparison:'abi-only'});
  if(new Set(rows.map(row=>row.id)).size!==rows.length)throw new Error('duplicate case id');
  return rows;
}

export function jsValue(recipe) {
  const document={languageVersion,...control};
  if(recipe.kind==='document-json')return JSON.parse(recipe.text);
  if(recipe.kind==='input-json') {const value=JSON.parse(recipe.text);return value && typeof value==='object' && !Array.isArray(value)?{languageVersion,...value}:value;}
  const rejectGetter=value=>Object.defineProperty({...document},'html',{enumerable:true,get(){throw value;}});
  switch(recipe.name) {
    case 'undefined':return undefined;case 'null':return null;case 'array':return [];case 'boolean':return true;
    case 'number':return 42;case 'string':return 'input';case 'bigint':return 1n;case 'symbol':return Symbol('input');case 'function':return ()=>{};
    case 'width-nan':return {...document,width:NaN};case 'width-infinity':return {...document,width:Infinity};case 'height-negative-infinity':return {...document,height:-Infinity};
    case 'width-bigint':return {...document,width:240n};case 'width-boxed':return {...document,width:new Number(240)};case 'html-boxed':return {...document,html:new String(control.html)};
    case 'html-undefined':return {...document,html:undefined};
    case 'unknown-cyclic':{const extra={};extra.self=extra;return {...document,assets:extra};}
    case 'getter-error':return rejectGetter(new Error('request getter rejected'));
    case 'getter-string':return rejectGetter('request getter rejected');
    case 'getter-unprintable-throw':return rejectGetter(Object.create(null));
    case 'proxy-keys-error':return new Proxy(document,{ownKeys(){throw new Error('request ownKeys rejected');}});
    case 'revoked-proxy':{const {proxy,revoke}=Proxy.revocable(document,{});revoke();return proxy;}
    case 'inherited-input':return Object.assign(Object.create(control),{languageVersion});
    case 'date':return new Date(0);case 'boxed-document':return new String('input');case 'missing-version':return {...control};
    case 'frozen-document':return Object.freeze(document);case 'null-prototype':return Object.assign(Object.create(null),document);
    case 'inherited-version':return Object.assign(Object.create({languageVersion}),control);
    default:throw new Error(`unknown JS recipe ${recipe.name}`);
  }
}
