"""Authored literal and responsive fractional edge controls; no measured inputs."""
import json,itertools
from pathlib import Path
P=Path(__file__).resolve().parent
cases=[];recipes={}
for route,axis,(offset,percent) in itertools.product(['layout','shape'],['y','x'],[(64.25,False),(64.5,False),(64.75,False),(65,False),(32.5,True)]):
 name=f'fractional-{route}-{axis}-{offset}-{("percent"if percent else"px")}'.replace('.', 'd')
 sizes=[[96,240],[160,320],[128,200],[96,240]]
 if percent:sizes=[[96,241],[160,321],[128,203],[96,241]]
 if axis=='x':sizes=[list(reversed(v))for v in sizes]
 offset_css=str(offset)+('%'if percent else'px')
 css='#p{width:100%;height:100%;flex-direction:'+('row'if axis=='x'else'column')+';}'
 css+='#spacer{width:'+ (offset_css if axis=='x'else'80px')+';height:'+('80px'if axis=='x'else offset_css)+';}'
 css+='#v{width:'+('45px'if axis=='x'else'80px')+';height:'+('80px'if axis=='x'else'45px')+';background:rgba(255,96,48,0.5019607843137255);}'
 # A semantically inert class distinguishes exact CSS requests for the two routes.
 html=f'<div id="p" class="{route}"><div id="spacer"></div><div id="v"></div></div>'
 cases.append(dict(name=name,html=html,css=css,compileViewport=sizes[0],viewports=sizes,features=['ordinary '+route+' paint','static rectangle path on native layout owner','fractional position; original and clone resize']))
 recipes[name]=dict(initialViewport=sizes[0],axis=axis,route=route,offset=offset,percent=percent)
for name,data in [('cases',cases),('recipes',recipes)]: (P/f'fractional-paint-{name}.json').write_text(json.dumps(data,indent=2)+'\n')
print(len(cases))
