"""Independently authored live paint-edge cases; no browser measurements."""
from pathlib import Path
import itertools,json
P=Path(__file__).resolve().parent;cases=[];recipes={}
profiles=[('quarter',64.25,False,45,False,0,0),('half',64.5,False,45,False,0,0),('three-quarter',64.75,False,45,False,0,0),('integer',65,False,45,False,0,0),('responsive-offset',32.5,True,45,False,0,0),('responsive-extent',64.5,False,25,True,0,0),('thin-limit',64,False,1/16,False,0,0),('thin-above',64,False,5/64,False,0,0),('thin-phase',64.25,False,5/64,False,0,0),('half-extent',64.25,False,.5,False,0,0),('fractional-extent',64.25,False,45.5,False,0,0),('negative-half',0,False,45,False,-.5,0),('negative-three-quarter',0,False,45,False,-.75,0),('cumulative-half',.25,False,45,False,0,.25)]
for axis,profile in itertools.product(['x','y'],profiles):
 label,offset,percent,extent,extent_percent,margin,inset=profile;name=f'rounded-paint-{axis}-{label}'
 sizes=[[96,240],[160,320],[128,200],[96,240]]
 if percent or extent_percent:sizes=[[96,241],[160,321],[128,203],[96,241]]
 if axis=='x':sizes=[list(reversed(s))for s in sizes]
 moving='width'if axis=='x'else'height';other='height'if axis=='x'else'width';side='left'if axis=='x'else'top'
 css=f'#p{{width:100%;height:100%;flex-direction:{"row"if axis=="x"else"column"};padding-{side}:{inset}px;}}'
 css+=f'#spacer{{{moving}:{offset}{"%"if percent else"px"};{other}:80px;}}'
 css+=f'#v{{{moving}:{extent}{"%"if extent_percent else"px"};{other}:80px;margin-{side}:{margin}px;background:rgba(255,96,48,0.5019607843137255);}}'
 cases.append(dict(name=name,html='<div id="p"><div id="spacer"></div><div id="v"></div></div>',css=css,compileViewport=sizes[0],viewports=sizes,features=['private live rounded clip paint','native layout owner preserved','signed and thin and cumulative edges']))
 recipes[name]=dict(axis=axis,initialViewport=sizes[0],offset=offset,percent=percent,extent=extent,extentPercent=extent_percent,margin=margin,inset=inset)
for suffix,data in [('cases',cases),('recipes',recipes)]: (P/f'rounding-paint-{suffix}.json').write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(dict(cases=len(cases))))
