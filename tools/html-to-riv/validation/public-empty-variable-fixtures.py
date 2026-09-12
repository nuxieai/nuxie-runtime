"""Generate independent unset controls and observable empty-variable compositions."""
import json
import pathlib

out = pathlib.Path(__file__).resolve().parent
html = '<div id="p"><div id="a"></div><div id="b"></div></div>'
base = '#p{width:200px;height:80px;background:teal;color:navy;font-size:24px}#a{width:40px;height:20px;background:coral}#b{width:20px;height:20px;background:gold}'
paint = {'p': 'rgb(0, 128, 128)', 'a': 'rgb(255, 127, 80)', 'b': 'rgb(255, 215, 0)'}
fallbacks = {
    'width': '30px', 'height': '30px', 'min-width': '30px', 'min-height': '30px',
    'max-width': '30px', 'max-height': '30px', 'box-sizing': 'border-box',
    'font-size': '12px', 'background': 'red', 'background-color': 'red', 'color': 'red',
    'display': 'flex', 'flex-direction': 'column', 'flex': 'none', 'flex-grow': '0',
    'flex-shrink': '0', 'flex-basis': 'auto', 'order': '-1', 'align-self': 'flex-end',
    'justify-content': 'space-evenly', 'margin': 'auto', 'margin-left': 'auto',
    'margin-top': 'auto', 'margin-right': 'auto', 'margin-bottom': 'auto',
    'padding': '4px', 'padding-left': '4px', 'padding-top': '4px',
    'padding-right': '4px', 'padding-bottom': '4px', 'gap': '4px',
    'row-gap': '4px', 'column-gap': '4px',
}
assert len(fallbacks) == 33
forms = []
for prop, fallback in fallbacks.items():
    variants = [
        ('fallback', '', 'var(--missing,)'),
        ('primary', '--empty:;', f'var(--empty,{fallback})'),
        ('whitespace', '--empty: \t\r\n\f ;', f'var(--empty,{fallback})'),
        ('comment', '--empty:/**/;', f'var(--empty,{fallback})'),
        ('mixed', '--empty: /* first */\t/**/ ;', f'var(--empty,{fallback})'),
        ('nested-fallback', '', 'var(--missing,var(--also-missing,))'),
        ('inherited-alias', '--empty:8px;', f'var(--alias,{fallback})'),
    ]
    for form, custom, value in variants:
        prefix = base + ('#p{--empty:;--alias:var(--empty)}' if form == 'inherited-alias' else '')
        forms.append(dict(name=f'empty-{prop}-{form}', property=prop, form=form, html=html,
                          css=prefix + f'#a{{{custom}{prop}:{value}}}',
                          literalCss=prefix + f'#a{{{custom}{prop}:unset}}',
                          accepted=prop not in ['display', 'flex', 'flex-shrink']))

cases = []
def add(name, target, authored, literal, expected=None, markup=html, prefix=base):
    cases.append(dict(name=name, html=markup, css=prefix + f'#{target}{{{authored}}}',
                      literalCss=prefix + f'#{target}{{{literal}}}',
                      expectedPaint=paint.copy() if expected is None else expected,
                      features=['S09', 'S10', 'successful-empty-substitution-to-unset']))

add('empty-responsive-width', 'a', 'width:31px;width:var(--missing,)', 'width:31px;width:unset', prefix=base+'#p{width:100%}')
add('empty-intrinsic-height', 'a', '--e:;height:60px;height:var(--e,30px)', 'height:60px;height:unset', markup='<div id="p"><div id="a"><div id="b"></div></div></div>')
add('empty-inherited-color', 'a', '--e:/**/;color:red;color:var(--e,coral);background:currentColor', 'color:red;color:unset;background:currentColor', dict(paint,a='rgb(0, 0, 128)'))
add('empty-background-no-rollback', 'a', 'background:red;background:var(--missing,)', 'background:red;background:unset', dict(paint,a='rgba(0, 0, 0, 0)'))
add('empty-inherited-font', 'a', '--e: \t/**/ ;font-size:11px;font-size:var(--e,8px);width:2em', 'font-size:11px;font-size:unset;width:2em')
add('empty-order-before-emission', 'a', '--e:;order:-4;order:var(--e,2)', 'order:-4;order:unset', prefix=base+'#b{order:-1}')
add('empty-direction-css-initial', 'p', 'flex-direction:column;flex-direction:var(--missing,)', 'flex-direction:column;flex-direction:unset')
add('empty-self-alignment', 'a', '--e:;align-self:flex-end;align-self:var(--e,center)', 'align-self:flex-end;align-self:unset')
add('empty-main-spacing', 'p', '--e:;justify-content:space-evenly;justify-content:var(--e,space-around)', 'justify-content:space-evenly;justify-content:unset')
add('empty-gap-shorthand', 'p', 'gap:8px 10px;gap:var(--missing,);column-gap:4px', 'gap:8px 10px;gap:unset;column-gap:4px', prefix=base+'#p{flex-direction:row}')
add('empty-padding-shorthand', 'p', '--e:;padding:8px;padding:var(--e,2px);padding-right:4px', 'padding:8px;padding:unset;padding-right:4px', prefix=base+'#a{width:auto}')
add('empty-margin-shorthand', 'a', '--e:;margin:auto;margin:var(--e,auto);margin-left:auto', 'margin:auto;margin:unset;margin-left:auto')
add('empty-min-width', 'a', 'min-width:60px;min-width:var(--missing,)', 'min-width:60px;min-width:unset')
add('empty-max-width', 'a', '--e:;max-width:30px;max-width:var(--e,20px)', 'max-width:30px;max-width:unset')
add('empty-losing-declaration', 'a', 'background:var(--missing,);background:coral', 'background:unset;background:coral')
add('empty-important-declaration', 'a', '--e:;background:var(--e,red)!important;background:coral', 'background:unset!important;background:coral', dict(paint,a='rgba(0, 0, 0, 0)'))
add('empty-nested-fallback', 'a', 'width:var(--missing,var(--also-missing,))', 'width:unset')
add('empty-inherited-alias-suppresses-fallback', 'a', '--e:red;background:var(--alias,coral)', 'background:unset', dict(paint,a='rgba(0, 0, 0, 0)'), prefix=base+'#p{--e:;--alias:var(--e)}')
add('empty-unused-fallback', 'a', '--good:30px;width:var(--good,)', 'width:30px')
add('empty-concatenated-literal', 'a', '--e:/**/;width:var(--e)30px', 'width:30px')
add('empty-color-function-component', 'a', '--e:;background:rgb(var(--e)255 0 0)', 'background:red', dict(paint,a='rgb(255, 0, 0)'))
add('empty-order-dom-selector', 'a', '--e:;order:-4;order:var(--e,9)', 'order:-4;order:unset', prefix=base+'#b{order:-1}#a:first-child{background:navy}', expected=dict(paint,a='rgb(0, 0, 128)'))
add('empty-content-box-css-initial', 'a', 'padding:4px;box-sizing:border-box;box-sizing:var(--missing,)', 'padding:4px;box-sizing:border-box;box-sizing:unset')
add('empty-shorthand-longhand-importance', 'p', '--e:;padding:8px!important;padding:var(--e)!important;padding-right:4px', 'padding:8px!important;padding:unset!important;padding-right:4px', prefix=base+'#a{width:auto}')

# Preserve exact pre-change rejected sources as newly positive controls.
old = json.loads((out/'public-variable-recovery-rejections.json').read_text())
for name in ['typed-empty-fallback', 'typed-empty-primary']:
    source = next(item for item in old if item['name'] == name)
    value = 'var(--missing,)' if name == 'typed-empty-fallback' else 'var(--empty,25px)'
    cases.append(dict(name='empty-former-'+name, html=source['html'], css=source['css'],
                      literalCss=source['css'].replace(value,'unset'), expectedPaint=paint.copy(),
                      features=['S09','S10','preserved-former-rejection']))

rejects = [
    ('nonempty-wrong-type', '--e:20px;background:var(--e)'),
    ('nonempty-token-boundary', '--e:10;width:var(--e)px'),
    ('quoted-empty-string', '--e:"";width:var(--e)'),
    ('empty-function', '--e:rgb();background:var(--e)'),
    ('empty-block', '--e:();width:var(--e)'),
    ('comma-token', '--e:,;width:var(--e)'),
    ('non-css-nbsp', '--e:\u00a0;width:var(--e)'),
    ('non-css-vertical-tab', '--e:\u000b;width:var(--e)'),
    ('fallback-non-css-nbsp', 'width:var(--missing,\u00a0)'),
    ('fallback-non-css-vertical-tab', 'width:var(--missing,\u000b)'),
    ('escaped-non-css-nbsp', r'--e:\a0;width:var(--e)'),
    ('fallback-escaped-non-css-nbsp', r'width:var(--missing,\a0)'),
    ('unsupported-property', '--e:;opacity:var(--e)'),
    ('unsupported-grid', '--e:grid;display:var(--e,)'),
    ('unsupported-calc-after-empty', '--e:;width:var(--e)calc(20px + 10px)'),
    ('over-limit-after-empty', '--e:;width:var(--e)1000001px'),
    ('literal-empty', 'width:'),
    ('literal-comment', 'width:/**/'),
    ('malformed-var', 'width:var(empty)'),
    ('malformed-unused-fallback', '--e:30px;width:var(--e,var(bad))'),
    ('nonempty-invalid-loser', '--e:teal;width:var(--e);width:40px'),
    ('nonempty-suffix-after-empty', '--e:;width:var(--e)px'),
    ('nonfinite-token', '--e:1e999px;width:var(--e)'),
    ('content-box-percent-size', 'box-sizing:var(--missing,);width:50%;padding:4px'),
    ('content-box-percent-padding', 'box-sizing:var(--missing,);padding:2%'),
]
rejections=[dict(name='empty-reject-'+name,html=html,css=base+f'#a{{{css}}}') for name,css in rejects]
rejections.append(dict(name='empty-reject-context-auto-height',html=html,css=base+'#p{height:var(--missing,)}#a{height:50%}'))
rejections.append(dict(name='empty-reject-font-overflow',html=html,css=base+'#p{font-size:1000000px}#a{font-size:var(--missing,);width:2em}'))

for name, data in [('forms',forms),('cases',cases),('rejections',rejections)]:
    (out/f'public-empty-variable-{name}.json').write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(dict(properties=len(fallbacks),forms=len(forms),acceptedForms=sum(item['accepted'] for item in forms),cases=len(cases),rejections=len(rejections))))
