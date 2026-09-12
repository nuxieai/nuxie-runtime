"""Authored public cases and independently specified border-box controls."""
from pathlib import Path
import json
module=Path(__file__).resolve().parent.parent
prior=json.loads((module/'validation/content-box-candidate-cases.json').read_text())
cases=[]
mins={
 'point-dimensions-asymmetric':'#p{min-width:18px;min-height:10px}',
 'point-minmax-clamps':'#p{min-height:10px}',
 'point-min-wins-max':'',
 'auto-main-intrinsic':'#p{min-width:20px;min-height:10px}',
 'auto-cross-responsive':'#p{min-width:20px;min-height:10px}',
 'nested-row-reverse-order':'#a{min-width:8px;min-height:4px}#b{min-width:10px;min-height:6px}',
 'font-relative-arithmetic':'#p{min-width:40px;min-height:20px}',
 'auto-with-point-max':'#p{min-width:20px;min-height:10px}',
 'point-fractional-paint-control':'#p{min-height:10px}',
}
for p in prior:
 if p['name'] in mins:
  cases.append(dict(name='content-'+p['name'],html=p['html'],css=p['css'],controlCss=p['nativeCss']+mins[p['name']],features=['public content-box point lowering'],classification='known-paint-limit' if 'fractional' in p['name'] else 'positive'))
def add(name,html,css,control,kind='positive'):
 cases.append(dict(name='content-'+name,html=html,css=css,controlCss=control,features=['public content-box '+name],classification=kind))
html='<div id="p"><div id="a"><div id="b"></div></div></div>'
base='#p{width:100px;height:60px;padding:8px;background:#ddeeff}#a{height:30px;padding:4px;background:#669933}#b{height:10px;background:#663399}'
add('mixed-inherited-sizes',html,base+'#p{box-sizing:content-box}#a,#b{width:inherit}',base+'#p{width:116px;height:76px;min-width:16px;min-height:16px}#a,#b{width:100px}')
add('content-inherited-sizes',html,base+'#p,#a{box-sizing:content-box}#a,#b{width:inherit}',base+'#p{width:116px;height:76px;min-width:16px;min-height:16px}#a{width:108px;height:38px;min-width:8px;min-height:8px}#b{width:100px}')
base='#p{width:100px;height:70px;padding:3px;background:#ddeeff}#a{width:50px;height:30px;background:#669933}#b{width:20px;height:10px;background:#663399}'
add('box-and-padding-inherit',html,base+'#p{box-sizing:content-box}#a{padding:inherit;box-sizing:inherit}',base+'#p{width:106px;height:76px;min-width:6px;min-height:6px}#a{padding:3px;width:56px;height:36px;min-width:6px;min-height:6px}')
html='<div id="p"><div id="a"></div><div id="b"></div></div>'
base='#p{width:100%;height:90px;background:#ddeeff}#a,#b{width:60px;height:20px;padding:3px;background:#669933}#b{background:#663399}'
for keyword in ['initial','unset','var(--missing)','var(--box, content-box)']:
 name={'var(--missing)':'invalid-substitution-unset','var(--box, content-box)':'variable-fallback'}.get(keyword,keyword)
 add(name,html,base+f'#a,#b{{box-sizing:{keyword}}}',base+'#a,#b{width:66px;height:26px;min-width:6px;min-height:6px}')
add('important-and-inline','<div id="p"><div id="a" style="box-sizing:content-box"></div><div id="b"></div></div>',base+'#a{box-sizing:border-box!important}#b{--box:CONTENT-BOX;box-sizing:var(--box)}',base+'#b{width:66px;height:26px;min-width:6px;min-height:6px}#a{box-sizing:border-box!important}')
for direction in ['row','row-reverse','column','column-reverse']:
 base=f'#p{{width:100%;height:100px;flex-direction:{direction};background:#ddeeff}}#a{{width:40px;height:20px;padding:3px 5px;background:#669933}}#b{{width:60px;height:30px;padding:4px 6px;background:#663399}}'
 add('direction-'+direction,html,base+'#a,#b{box-sizing:content-box}',base+'#a{width:50px;height:26px;min-width:10px;min-height:6px}#b{width:72px;height:38px;min-width:12px;min-height:8px}')
base='#p{width:100%;height:100px;flex-direction:row;background:#ddeeff}#a{width:40px;height:20px;padding:3px 5px;margin-left:auto;background:#669933}#b{width:60px;height:30px;padding:4px 6px;background:#663399}'
add('main-auto-margin',html,base+'#a,#b{box-sizing:content-box}',base+'#a{width:50px;height:26px;min-width:10px;min-height:6px}#b{width:72px;height:38px;min-width:12px;min-height:8px}')
for alignment in ['space-around','space-evenly']:
 base=f'#p{{width:100%;height:100px;flex-direction:row;justify-content:{alignment};background:#ddeeff}}#a,#b{{width:40px;height:20px;padding:3px 5px;background:#669933}}#b{{background:#663399}}'
 add('distribution-'+alignment,html,base+'#a,#b{box-sizing:content-box}',base+'#a,#b{width:50px;height:26px;min-width:10px;min-height:6px}')
base='#p{width:70%;height:40px;min-width:30%;padding:5px 0;background:#ddeeff}#a{width:100%;height:20px;background:#669933}#b{width:50%;height:20px;background:#663399}'
add('width-percent-zero-inset',html,base+'#p{box-sizing:content-box}',base+'#p{height:50px;min-height:10px}')
base='#p{width:100px;height:70%;padding:0 10px;background:#ddeeff}#a{width:100%;height:50%;background:#669933}#b{width:50%;height:50%;background:#663399}'
add('height-percent-zero-inset',html,base+'#p{box-sizing:content-box}',base+'#p{width:120px;min-width:20px}')
base='#p{width:100%;height:100px;background:#ddeeff}#a{width:70%;height:40px;padding:0% 0px 0em 0rem;background:#669933}#b{width:50%;height:30px;background:#663399}'
add('literal-zero-insets',html,base+'#a,#b{box-sizing:content-box}',base)
base='#p{width:0px;height:0px;padding:12px 16px;background:#ddeeff}#a{width:0px;height:0px;background:#669933}#b{width:0px;height:0px;background:#663399}'
add('zero-content-padding-floor',html,base+'#p{box-sizing:content-box}',base+'#p{width:32px;height:24px;min-width:32px;min-height:24px}')
base='#p{width:auto;height:auto;min-width:auto;min-height:auto;max-width:100px;padding:5px;background:#ddeeff}#a{width:60px;height:20px;background:#669933}#b{width:70px;height:30px;background:#663399}'
add('automatic-minima-and-max',html,base+'#p{box-sizing:content-box}',base+'#p{max-width:110px}')
for name,width,left,right,outer in [
 ('large-add-side-first','1000000','.03125','.03125','1000000.0625'),
 ('large-padding-cancellation','.03125','1000000','.03125','1000000'),
 ('large-rounded-outer','999999.875','1000000','1000000','3000000'),
 ('large-content-cancellation','999999.9375','1000000','1000000','3000000'),
 ('large-full-source-limits','1000000','1000000','1000000','3000000'),
]:
 base=f'#p{{width:{width}px;height:20px;padding:0 {right}px 0 {left}px;background:#14233f}}#c{{width:100%;height:1px;background:#669933}}'
 # Canonical native padding sum; these controls preserve native byte identities,
 # while Chrome is compared independently against the original content-box CSS.
 import struct
 f32=lambda x:struct.unpack('f',struct.pack('f',float(x)))[0]
 minimum=f32(f32(left)+f32(right))
 add(name,'<div id="p"><div id="c"></div></div>',base+'#p{box-sizing:content-box}',base+f'#p{{width:{outer}px;min-width:{minimum}px}}','numeric-boundary')
(module/'validation/public-content-box-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
(module/'validation/public-content-box-boundary-cases.json').write_text(json.dumps([c for c in cases if c['classification']=='numeric-boundary'],indent=2)+'\n')
rejections=[]
base='#p{width:160px;height:100px}#a{width:40px;height:20px}'
for name,css in [
 ('mixed-width', '#a{box-sizing:content-box;width:50%;padding:5px}'),
 ('mixed-height', '#a{box-sizing:content-box;height:50%;padding:5px}'),
 ('mixed-minimum','#a{box-sizing:content-box;min-width:50%;padding:5px}'),
 ('mixed-maximum','#a{box-sizing:content-box;max-height:50%;padding:5px}'),
 ('percentage-padding','#a{box-sizing:content-box;padding:5%}'),
 ('padded-alignment-wrapper','#a{box-sizing:content-box;padding:5px;align-self:center}'),
 ('padded-baseline','#p{flex-direction:row}#a{box-sizing:content-box;padding:5px;align-self:baseline}'),
 ('padded-gap','#p{gap:5px}#a{box-sizing:content-box;padding:5px}'),
 ('invalid-keyword','#a{box-sizing:padding-box}'),
 ('strict-overridden','#a{box-sizing:padding-box;box-sizing:border-box}'),
 ('strict-unmatched','#never{box-sizing:padding-box}'),
 ('strict-variable','#a{--bad:padding-box;box-sizing:var(--bad);box-sizing:border-box}'),
 ('invalid-unit','#a{box-sizing:0}'),
]:rejections.append(dict(name=name,html='<div id="p"><div id="a"></div></div>',css=base+css))
rejections.append(dict(name='lowered-content-overflow-chain',html='<div id="p">'+'<div>'*10+'</div>'*10+'</div>',css='div{width:1000000%;height:1px}#p{box-sizing:content-box;width:1px;padding:1px}'))
(module/'validation/public-content-box-rejections.json').write_text(json.dumps(rejections,indent=2)+'\n')
print(len(cases),len(rejections))
