"""Exact authoring-field expectations; native/Chrome qualification is separate."""
from pathlib import Path
import json
import struct

root=Path(__file__).parent
cases=[]
def field(node,record,name,decimal=None,integer=None):
    result=dict(node=node,record=record,property=name)
    if decimal is not None: result['decimal']=str(decimal)
    else: result['integer']=integer
    return result
def case(name,group,css,fields,html='<div id=p></div>'):
    cases.append(dict(name='numeric-token-'+name,group=group,html=html,css=css,expectedFields=fields))
width=lambda value,node='p':field(node,'component','width',value)
style=lambda name,value,node='p':field(node,'style',name,value)
paint=lambda value,node='p':field(node,'paint','colorValue',integer=int(value,16))
for name,value in [('literal','999999.875px'),('signed-exponent','+9.99999875e5px'),('escaped-unit',r'999999.875p\78 '),('uppercase-unit','999999.875PX'),('one-ulp','1.00000011920928955078125px')]:
    expected='1.00000011920928955078125' if name=='one-ulp' else '999999.875'
    case(name,'ordinary',f'#p{{width:{value};height:20px}}',[width(expected)])
case('inline','ordinary','',[width('999999.875')],'<div id=p style="width:999999.875px;height:20px"></div>')
case('negative-zero','ordinary','#p{width:-0px;height:20px}',[width('-0')])
for name,value in [('variable','--w:999999.875px;width:var(--w)'),('alias','--w:999999.875px;--alias:var(--w);width:var(--alias)'),('fallback','width:var(--missing,+9.99999875e5px)'),('variable-escaped-unit',r'--w:999999.875p\78 ;width:var(--w)')]:
    case(name,'variables',f'#p{{{value};height:20px}}',[width('999999.875')])
case('inherited-variable','variables','#p{--w:999999.875px;height:20px}#c{width:var(--w);height:1px}',[width('999999.875','c')],'<div id=p><div id=c></div></div>')
case('inherited-computed-width','variables','#p{width:999999.875px;height:20px}#c{width:inherit;height:1px}',[width('999999.875'),width('999999.875','c')],'<div id=p><div id=c></div></div>')
case('percentage','ordinary','#p{width:99.9999875%;height:25.123456789%}',[width('99.9999875'),field('p','style','widthUnitsValue',integer=2),field('p','component','height','25.123456789'),field('p','style','heightUnitsValue',integer=2)])
case('variable-percentage','variables','#p{--w:+9.99999875e1%;width:var(--w);height:20px}',[width('99.9999875'),field('p','style','widthUnitsValue',integer=2)])
case('em','font','#p{font-size:1000px;width:999.999875em;height:1px}',[width('999999.875')])
case('rem','font','#p{width:62499.9921875rem;height:1px}',[width('999999.875')])
case('font-size-number','font','#p{font-size:999.999875px;width:1000em;height:1px}',[width('999999.875')])
case('font-before-inheritance','font','#p{font-size:999.999875px;height:20px}#c{font-size:inherit;width:1000em;height:1px}',[width('999999.875','c')],'<div id=p><div id=c></div></div>')
case('padding-shorthand','bounds','#p{width:200px;height:100px;padding:1.00000011920928955078125px 2.123456789px 3.123456789px 4.123456789px}',[style('paddingTop','1.00000011920928955078125'),style('paddingRight','2.123456789'),style('paddingBottom','3.123456789'),style('paddingLeft','4.123456789')])
case('variable-padding-percentage','bounds','#p{width:200px;height:100px;--pad:12.123456789%;padding-left:var(--pad)}',[style('paddingLeft','12.123456789'),field('p','style','paddingLeftUnitsValue',integer=2)])
case('minmax','bounds','#p{width:100px;height:30px;min-width:99.9999875px;max-width:100.71428680419922px;min-height:12.123456789%;max-height:99.9999875%}',[style('minWidth','99.9999875'),style('maxWidth','100.71428680419922'),style('minHeight','12.123456789'),style('maxHeight','99.9999875'),field('p','style','minHeightUnitsValue',integer=2),field('p','style','maxHeightUnitsValue',integer=2)])
f32=lambda n:struct.unpack('<f',struct.pack('<f',float(n)))[0]
outer=f32(f32('100.71428680419922')+f32(f32('5.123456478118896')+f32('5.123456478118896')))
case('content-box-derived','bounds','#p{box-sizing:content-box;width:100.71428680419922px;height:20px;padding:0 5.123456478118896px}',[width(outer),style('paddingLeft','5.123456478118896'),style('paddingRight','5.123456478118896')])
colors=[('rgb-number','rgb(127.49999 0 0)','ff7f0000'),('rgb-percentage','rgb(49.99999% 0% 0%)','ff7f0000'),('rgb-alpha','rgb(255 0 0 / .4999999)','7fff0000'),('hsl-angle','hsl(.11764706deg 100% 50%)','ffff0100'),('hsl-lightness','hsl(0 0% 49.99999%)','ff7f7f7f'),('hsl-alpha','hsl(0 100% 50% / .4999999)','7fff0000')]
for name,value,color in colors:case(name,'color',f'#p{{width:100px;height:20px;background:{value}}}',[paint(color)])
case('color-variable','color','#p{width:100px;height:20px;--r:127.49999;--a:.4999999;color:rgb(var(--r) 0 0 / var(--a));background:currentColor}',[paint('7f7f0000')])
case('color-inline','color','',[paint('ff7f0000')],'<div id=p style="width:100px;height:20px;background:rgb(127.49999 0 0)"></div>')

rejected=[]
def reject(name,css,html='<div id=p></div>',category='token-category'):
    rejected.append(dict(name='numeric-token-reject-'+name,group=category,html=html,css=css))
for i,value in enumerate([r'1\65 2px',r'1\45 2px',r'1\65-2px',r'1\45-2px',r'1\65 2']):
    for context in ['literal','variable','inline']:
        css=f'#p{{width:{value}}}' if context=='literal' else f'#p{{--w:{value};width:var(--w)}}'
        html='<div id=p></div>'
        if context=='inline':css='';html=f'<div id=p style="width:{value}"></div>'
        reject(f'exponent-unit-{i}-{context}',css,html)
reject('color-exponent-unit',r'#p{background:hsl(1\65 2 100% 50%)}')
for name,css in [('number-unit-fragments','#p{--n:10;width:var(--n)px}'),('variable-unit-fragments','#p{--n:10;--u:px;width:var(--n)var(--u)}'),('exponent-fragments','#p{--n:1;--u:e2px;width:var(--n)var(--u)}'),('percentage-fragments','#p{--n:10;width:var(--n)%}')]:reject(name,css)
for i,value in enumerate(['1.0','1e2','2147483648','-2147483649','1px','1%','+ 1']):
    for context in ['literal','variable']:
        css=f'#p{{order:{value};order:0}}' if context=='literal' else f'#p{{--o:{value};order:var(--o);order:0}}'
        reject(f'order-{i}-{context}',css,category='integer-grammar')
for i,value in enumerate(['1e999px','1e999%','1e999em']):reject(f'nonfinite-{i}',f'#p{{width:{value}}}',category='nonfinite')
for file,data in [('public-numeric-token-cases.json',cases),('public-numeric-token-rejections.json',rejected)]:
    (root/file).write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),rejections=len(rejected))))
