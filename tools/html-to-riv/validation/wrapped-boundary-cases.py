"""Authored boundary/overlap recipes, independent of browser observations."""
from pathlib import Path
import itertools,json,struct
M=Path(__file__).resolve().parents[1]
def dim(v,unit='px'):return dict(value=v,unit=unit)
def cssdim(name,d):return f'{name}:{d["value"]}{d["unit"]};'
def adjacent(v,step):return struct.unpack('<f',struct.pack('<I',struct.unpack('<I',struct.pack('<f',v))[0]+step))[0]
colors=[0xbfffff00,0x80ff6030,0xc000bba0]
cases=[];recipes={}
def add(name,row,reverse,wrap,specs,sizes,main_align=0):
 axis='row'if row else'column';sizes=sizes if row else[[h,w]for w,h in sizes]
 parent={'width':dim(100,'%'),'height':dim(100,'%')};slots=[]
 css='#p{width:100%;height:100%;'+f'flex-direction:{axis}{"-reverse"if reverse else""};flex-wrap:{"wrap"if wrap==1 else"wrap-reverse"};align-content:flex-start;}}'
 html='<div id="p">'
 for i,(main,cross,vm,vc,a)in enumerate(specs):
  pa=a if wrap==1 else 1-a
  h,v=(main_align,pa)if row else(pa,main_align)
  slot={'name':f'v{i}','main':dim(main),'cross':dim(cross),'visibleMain':dim(vm),'visibleCross':dim(vc),'slotAlignment':int(2*h+6*v),'alignment':a,'color':colors[i%3]}
  slots.append(slot);html+=f'<div class="s{i}"><div id="v{i}"></div></div>'
  css+=f'.s{i}{{'+cssdim('width'if row else'height',slot['main'])+cssdim('height'if row else'width',slot['cross'])
  css+=f'align-self:{["flex-start","center","flex-end"][int(a*2)]};flex-direction:column;'
  css+=f'align-items:{["flex-start","center","flex-end"][int(h*2)]};justify-content:{["flex-start","center","flex-end"][int(v*2)]};}}'
  color=slot['color'];r=(color>>16)&255;g=(color>>8)&255;b=color&255;alpha=(color>>24)/255
  css+=f'#v{i}{{'+cssdim('width'if row else'height',slot['visibleMain'])+cssdim('height'if row else'width',slot['visibleCross'])+f'background:rgba({r},{g},{b},{alpha:.16g});}}'
 html+='</div>';name='boundary-'+axis+'-'+name
 recipes[name]=dict(initialViewport=sizes[0],row=row,reverseMain=reverse,wrap=wrap,lineFraction=0,mainFraction=0,parent=parent,slots=slots)
 cases.append(dict(name=name,html=html,css=css,compileViewport=sizes[0],viewports=sizes,features=['actual owned Derived','saturation/thin/accumulation/overlap boundary','full declared viewport domain','no browser inputs']))
for row in [True,False]:
 for name,reverse,slotmain,visiblemain,alignment in [
  ('negative-saturation',True,20000,20,0),('positive-saturation',False,20000,20,1),
  ('spanning',False,60,40000,0),('negative-spanning',True,20000,40000,0)]:
  add(name,row,reverse,1,[(slotmain,80,visiblemain,80,0)],[[96,96],[160,120],[80,100],[96,96]],alignment)
 for name,width in [('zero',0),('thin-limit',1/16),('thin-next-above',adjacent(1/16,1)),('thin-next-below',adjacent(1/16,-1)),('thin-above',5/64),('half-extent',.5)]:
  add(name,row,False,1,[(60,80,width,80,0)],[[96,96],[128,96],[80,96],[96,96]])
 for name,first,second in [('accumulated-exact',64.25,.25),('accumulated-decimal',64.249,.251)]:
  add(name,row,False,1,[(first,50,first,50,0),(second,50,second,50,0),(30,50,30,50,0)],[[96,96],[160,96],[80,96],[96,96]])
 for reverse,wrap in itertools.product([False,True],[1,2]):
  add(f'overlap-r{int(reverse)}-w{wrap}',row,reverse,wrap,[(60,50,110,90,0),(50,80,100,120,.5),(40,40,100,90,1)],[[160,160],[96,200],[64,240],[160,160]])
(M/'validation/wrapped-boundary-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
(M/'validation/wrapped-boundary-recipes.json').write_text(json.dumps(recipes,indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),recipes=len(recipes))))
