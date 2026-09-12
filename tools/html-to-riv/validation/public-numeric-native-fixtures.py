"""Finite native numeric-consumer matrix; retain fractional and large-coordinate cases."""
import hashlib
import json
import pathlib

out=pathlib.Path(__file__).resolve().parent
html='<div id="p"><div id="a"></div><div id="b"></div></div>'
base='#p{width:200px;height:80px;background:#14233f}#a{width:40px;height:20px;background:#669933}#b{width:30px;height:20px;background:#cc6644}'
paint={'p':'rgb(20, 35, 63)','a':'rgb(102, 153, 51)','b':'rgb(204, 102, 68)'}
cases=[]
def add(name,css,markup=html):
    cases.append(dict(name='numeric-'+name,html=markup,css=base+css,classification='visible-numeric-consumer',expectedPaint=paint.copy()))
add('decimal-width','#a{width:100.71428680419922px}')
add('adjacent-f32-width','#a{width:1.00000011920928955078125px}')
add('fractional-height','#a{height:25.499998092651367px;width:180px}')
add('decimal-padding','#p{padding:4.123456954956055px 7.654321193695068px}#a{width:100%}')
add('decimal-minimum','#a{width:30px;min-width:100.71428680419922px}')
add('decimal-maximum','#a{width:180px;max-width:100.71428680419922px}')
add('inherited-minimum','#p{min-width:100.71428680419922px}#a{min-width:inherit}')
add('responsive-percentage','#p{width:100%}#a{width:33.333335876464844%}')
add('percentage-minimum','#p{width:100%}#a{width:10px;min-width:33.333335876464844%;max-width:66.66667175292969%}')
add('decimal-rem','#a{width:6.294642925262451rem}')
add('decimal-em-font','#p{font-size:12.345678329467773px}#a{width:8.125em}')
add('percentage-font','#p{font-size:18.76543426513672px}#a{font-size:66.66667175292969%;width:8em}')
add('variable-alias','#a{--width:100.71428680419922px;--alias:var(--width);width:var(--alias)}')
add('inherited-variable-alias','#p{--width:100.71428680419922px;--alias:var(--width)}#a{--width:40px;width:var(--alias)}')
add('escaped-fallback',r'#a{width:var(--missing,1.0071428680419922e2p\78)}')
add('inline-escaped-em','',markup=html.replace('<div id="a">',r'<div id="a" style="font-size:1.25e1p\78;width:8.0571429443359375em">'))
add('relative-padding','#p{font-size:20px;padding:.20576131343841553em .4783950746059418em}#a{width:100%}')
add('responsive-percentage-padding','#p{width:100%;padding:0 1.2345678806304932%}#a{width:100.71428680419922px}')
add('content-box-decimal-points','#a{box-sizing:content-box;width:100.71428680419922px;padding:4.123456954956055px}#b{width:100%}',markup='<div id="p"><div id="a"><div id="b"></div></div></div>')
add('content-box-decimal-rem','#a{box-sizing:content-box;width:6.294642925262451rem;padding:.2577160596847534rem}')

color_specs=[
    ('rgb-half-byte','rgb(127.49999 0 0)','rgb(127, 0, 0)','0xff7f0000'),
    ('rgb-percent-half-byte','rgb(49.99999% 0% 0%)','rgb(127, 0, 0)','0xff7f0000'),
    ('alpha-half-byte','rgba(255,0,0,.4999999)','rgba(255, 0, 0, 0.498)','0x7fff0000'),
    ('hue-small-angle','hsl(.11764706deg 100% 50%)','rgb(255, 1, 0)','0xffff0100'),
    # Pinned Chrome's HSL path differs by one channel from the current ordinary
    # color parser. Preserve both expectations instead of equating lexical
    # preservation with exact browser color equivalence.
    ('hsl-lightness-half-byte','hsl(0 0% 49.99999%)','rgb(128, 128, 128)','0xff7f7f7f'),
    ('variable-rgb-alpha','rgba(var(--red),0,0,var(--alpha))','rgba(127, 0, 0, 0.498)','0x7f7f0000'),
]
for name,color,expected,argb in color_specs:
    cases.append(dict(name='numeric-color-'+name,html='<div id="p"></div>',
                      css=f'#p{{--red:127.49999;--alpha:.4999999;width:160px;height:50px;background:{color}}}',
                      classification='visible-numeric-color-boundary',expectedPaint={'p':expected},
                      expectedOrdinaryArgb=argb))

large=[];bindings=[]
for name in ['content-box-rounding-repro-cases.json','content-box-rounding-controls.json']:
    source=out/name;bindings.append(dict(path=str(source),sha256=hashlib.sha256(source.read_bytes()).hexdigest()))
    for row in json.loads(source.read_text()):
        large.append(dict(**row,classification='retained-large-coordinate-diagnostic',originalFixture=name))

for name,data in [('cases',cases),('large-cases',large),('large-source-bindings',bindings)]:
    (out/f'public-numeric-native-{name}.json').write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(dict(visibleScenes=len(cases),largeCoordinateScenes=len(large))))
