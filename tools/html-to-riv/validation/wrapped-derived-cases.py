"""Authored CSS/record recipes; never reads Chrome geometry or a rendered artifact."""
from pathlib import Path
import itertools,json
M=Path(__file__).resolve().parents[1]
def dim(value,unit='px',minimum=None,maximum=None):
    x={'value':value,'unit':unit}
    if minimum:x['min']=minimum
    if maximum:x['max']=maximum
    return x
def css_dim(name,x):
    text=f"{name}:{x['value']}{x['unit']};"
    for bound in ['min','max']:
        if bound in x:text+=f"{bound}-{name}:{x[bound]['value']}{x[bound]['unit']};"
    return text
cases=[];recipes={}
for row,reverse,wrap,level,profile in itertools.product([True,False],[False,True],[1,2],[0,.5,1],['equal','responsive-visible']):
    axis='row'if row else'column'
    name=f"derived-{axis}-reverse{int(reverse)}-wrap{wrap}-level{int(level*2)}-{profile}"
    sizes=[[320,200],[160,320],[96,240],[320,200]]
    if not row:sizes=[[h,w]for w,h in sizes]
    parent={'width':dim(75 if row else 60,'%'),'height':dim(60 if row else 75,'%')}
    slots=[];html='<div id="p">';css='#p{'+css_dim('width',parent['width'])+css_dim('height',parent['height'])
    css+=f"flex-direction:{axis}{'-reverse'if reverse else''};flex-wrap:{'wrap'if wrap==1 else'wrap-reverse'};align-content:{['flex-start','center','flex-end'][int(level*2)]};}}"
    colors=[0xbfffff00,0x80ff6030,0xc000bba0]
    for i in range(3):
        main=dim([60,60,40][i]);cross=dim([50,30,70][i])
        if profile=='responsive-visible' and i==0:cross=dim(50,'%',dim(24),dim(72))
        visible_main=dim([60,80,40][i]) # One deliberate overlap, including reverse-main paint.
        visible_cross=dim(100,'%') if profile=='equal' else dim([50,150,75][i],'%',dim(12),dim([60,80,90][i]))
        a=i/2;physical_a=a if wrap==1 else 1-a
        slots.append({'name':f'v{i}','main':main,'cross':cross,'visibleMain':visible_main,'visibleCross':visible_cross,'slotAlignment':0,'alignment':a,'color':colors[i]})
        html+=f'<div class="s{i}"><div id="v{i}"></div></div>'
        css+=f'.s{i}{{'+css_dim('width'if row else'height',main)+css_dim('height'if row else'width',cross)
        css+=f"align-self:{['flex-start','center','flex-end'][i]};flex-direction:column;"
        # Reference wrapper is itself aligned in its flex line. Its child uses
        # the same PHYSICAL cross fraction, yielding visible alignment even when
        # child and wrapper have different used sizes. Wrappers carry no paint.
        css+=f"{'justify-content'if row else'align-items'}:{['flex-start','center','flex-end'][int(physical_a*2)]};"
        css+=f"{'align-items'if row else'justify-content'}:flex-start;}}"
        color=colors[i];r=(color>>16)&255;g=(color>>8)&255;b=color&255;a8=(color>>24)&255
        css+=f'#v{i}{{'+css_dim('width'if row else'height',visible_main)+css_dim('height'if row else'width',visible_cross)+f'background:rgba({r},{g},{b},{a8}/255);}}'
        # CSS alpha uses an authored exact decimal, not an invalid division token.
        css=css.replace(f',{a8}/255)',f',{a8/255:.16g})')
    html+='</div>'
    recipe={'initialViewport':sizes[0],'row':row,'reverseMain':reverse,'wrap':wrap,'lineFraction':level,'mainFraction':0,'parent':parent,'slots':slots}
    cases.append({'name':name,'html':html,'css':css,'compileViewport':sizes[0],'viewports':sizes,'features':['private actual compose_with_bounds','derived epsilon over full viewport domain','anchored visible size','overlap and repeated original/clone resize']})
    recipes[name]=recipe
(M/'validation/wrapped-derived-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
(M/'validation/wrapped-derived-recipes.json').write_text(json.dumps(recipes,indent=2)+'\n')
print(json.dumps({'cases':len(cases),'recipes':len(recipes)}))
