"""Independent Chrome grammar observations and bounded painted recovery controls."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
HTML = '<div id="p"><div id="a"><div id="c"></div></div><div id="b"></div></div>'
BASE = '#p{width:200px;height:100px;background:teal;color:navy;font-size:24px}#a{width:80px;height:30px;background:coral;color:maroon;font-size:12px}#b{width:30px;height:20px;background:gold}#c{width:10px;height:10px;background:currentColor}'
VALUES = {
    'width': '40px', 'height': '40px', 'min-width': '40px', 'min-height': '40px',
    'max-width': '40px', 'max-height': '40px', 'box-sizing': 'border-box',
    'font-size': '20px', 'background': 'red', 'background-color': 'red', 'color': 'red',
    'display': 'flex', 'flex-direction': 'column', 'flex': 'none', 'flex-grow': '0',
    'flex-shrink': '0', 'flex-basis': 'auto', 'order': '-1', 'align-self': 'flex-end',
    'justify-content': 'space-evenly', 'margin': 'auto', 'margin-left': 'auto',
    'margin-top': 'auto', 'margin-right': 'auto', 'margin-bottom': 'auto',
    'padding': '4px', 'padding-left': '4px', 'padding-top': '4px',
    'padding-right': '4px', 'padding-bottom': '4px', 'gap': '4px',
    'row-gap': '4px', 'column-gap': '4px',
}
assert len(VALUES) == 33
forms = []


def form(name, prop, tokens, *, mode='primary', expected_support=False, label='grammar-characterization'):
    good = VALUES[prop]
    setup, ordinary = ('--v:' + tokens + ';', 'var(--v)') if mode == 'primary' else ('', 'var(--missing,' + tokens + ')')
    prefix = BASE + '#a{' + prop + ':' + good + '!important}'
    # Both declarations have the same importance: the winning invalid-at-computed
    # declaration must unset, never roll back to the preceding known value.
    css = prefix + '#a{' + setup + prop + ':' + ordinary + '!important}'
    control = prefix + '#a{' + prop + ':' + (tokens if expected_support else 'unset') + '!important}'
    direct = prefix + '#a{' + prop + ':' + tokens + '!important}'
    forms.append(dict(name=name, property=prop, tokens=tokens, form=mode, html=HTML,
                      css=css, controlCss=control, directCss=direct, prefixCss=prefix,
                      expectedLiteralSupport=expected_support, classification=label))


for prop, good in VALUES.items():
    variants = [
        ('unknown-ident', 'nuxie-invalid-value'),
        ('quoted-string', '"red"'),
        ('nbsp-prefix', '\u00a0' + good),
        ('nbsp-suffix', good + '\u00a0'),
        ('vertical-tab-boundary', '\u000b' + good + '\u000b'),
        ('em-space-boundary', '\u2003' + good + '\u2003'),
        ('escaped-nbsp-prefix', r'\a0 ' + good),
        ('unknown-function', 'nuxie-unknown()'),
    ]
    for name, tokens in variants:
        form(f'value-{prop}-{name}', prop, tokens)
    form(f'value-{prop}-fallback-nbsp', prop, '\u00a0' + good + '\u00a0', mode='fallback')
    form(f'value-{prop}-css-whitespace', prop, ' \t\r\n\f' + good + ' \t\r\n\f', expected_support=True, label='valid-css-whitespace-control')
    for keyword in ['inherit', 'initial', 'unset']:
        form(f'value-{prop}-fallback-{keyword}', prop, keyword, mode='fallback', expected_support=True, label='css-wide-fallback')

# Valid CSS beyond target admission is recorded against its literal computation,
# never rewritten to unset merely because this compiler rejects it.
valid_unsupported = [
    ('background-position-only', 'background', '20px'),
    ('background-repeat', 'background', 'no-repeat'),
    ('background-gradient', 'background', 'linear-gradient(red, blue)'),
    ('background-color-system', 'background-color', 'CanvasText'),
    ('width-calc', 'width', 'calc(20px + 10px)'),
    ('width-min-content', 'width', 'min-content'),
    ('width-vw', 'width', '10vw'),
    ('width-negative-calc-clamp', 'width', 'calc(-20px)'),
    ('font-keyword', 'font-size', 'large'),
    ('font-relative-keyword', 'font-size', 'larger'),
    ('font-calc', 'font-size', 'calc(10px + 2px)'),
    ('negative-margin', 'margin', '-4px'),
    ('negative-margin-left', 'margin-left', '-4px'),
    ('display-grid', 'display', 'grid'),
    ('display-none', 'display', 'none'),
    ('flex-factors', 'flex', '1 1 20px'),
    ('flex-grow', 'flex-grow', '.5'),
    ('flex-shrink', 'flex-shrink', '1'),
    ('flex-basis-content', 'flex-basis', 'content'),
    ('align-baseline', 'align-self', 'baseline'),
    ('justify-center', 'justify-content', 'center'),
    ('gap-calc', 'gap', 'calc(2px + 2px)'),
    ('padding-vw', 'padding', '1vw'),
    ('color-mix', 'color', 'color-mix(in srgb, red, blue)'),
    ('color-lab', 'color', 'lab(50% 20 30)'),
    ('css-wide-revert', 'color', 'revert'),
    ('css-wide-revert-layer', 'color', 'revert-layer'),
]
for name, prop, tokens in valid_unsupported:
    form('value-valid-unsupported-' + name, prop, tokens, mode='fallback', expected_support=True, label='valid-css-separate-target-admission')

# Wrong-type/function cases that require a specific grammar decision. Browser
# invalidity alone is not authorization to catch every compiler parse diagnostic.
for name, prop, tokens in [
    ('color-length', 'color', '20px'), ('background-color-length', 'background-color', '20px'),
    ('width-color', 'width', 'red'), ('height-color', 'height', 'red'),
    ('width-negative', 'width', '-20px'), ('padding-negative', 'padding', '-4px'),
    ('font-negative', 'font-size', '-1px'), ('grow-negative', 'flex-grow', '-1'),
    ('shrink-negative', 'flex-shrink', '-1'), ('order-fraction', 'order', '1.5'),
    ('padding-color', 'padding', 'red'), ('margin-color', 'margin', 'red'),
    ('gap-auto', 'gap', 'auto'), ('direction-number', 'flex-direction', '20px'),
    ('rgb-empty', 'background-color', 'rgb()'), ('color-quoted-empty', 'color', '""'),
    ('width-token-concatenation', 'width', '10 px'), ('width-unknown-unit', 'width', '20frob'),
    ('order-exponent', 'order', '1e0'), ('color-semantic-extra', 'color', 'red blue'),
]:
    form('value-typed-' + name, prop, tokens, mode='fallback', label='typed-invalid-browser-characterization')

paint = {'p': 'rgb(0, 128, 128)', 'a': 'rgb(255, 127, 80)', 'b': 'rgb(255, 215, 0)', 'c': 'rgb(128, 0, 0)'}
scenes = []


def scene(name, target, authored, literal, *, prefix=BASE, expected=None, markup=HTML):
    scenes.append(dict(name='value-' + name, html=markup, css=prefix + f'#{target}{{{authored}}}',
                       literalCss=prefix + f'#{target}{{{literal}}}', expectedPaint=paint.copy() if expected is None else expected,
                       classification='candidate-painted-recovery-control'))


scene('width-color-responsive', 'a', '--v:red;width:31px;width:var(--v)', 'width:31px;width:unset', prefix=BASE+'#p{width:100%}')
scene('height-color-intrinsic', 'a', '--v:red;height:80px;height:var(--v)', 'height:80px;height:unset')
scene('color-length-inherits', 'a', '--v:20px;color:red;color:var(--v);background:currentColor', 'color:red;color:unset;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('background-color-length-unsets', 'a', '--v:20px;background-color:red;background-color:var(--v)', 'background-color:red;background-color:unset', expected=dict(paint,a='rgba(0, 0, 0, 0)'))
scene('font-color-inherits', 'a', '--v:red;font-size:8px;font-size:var(--v);width:2em', 'font-size:8px;font-size:unset;width:2em')
scene('order-fraction-preserves-position', 'a', '--v:1.5;order:-4;order:var(--v)', 'order:-4;order:unset', prefix=BASE+'#b{order:-1}')
scene('direction-number-initial-row', 'p', '--v:20px;flex-direction:column;flex-direction:var(--v)', 'flex-direction:column;flex-direction:unset')
scene('self-alignment-number', 'a', '--v:20px;align-self:flex-end;align-self:var(--v)', 'align-self:flex-end;align-self:unset')
scene('justify-number', 'p', '--v:20px;justify-content:space-evenly;justify-content:var(--v)', 'justify-content:space-evenly;justify-content:unset')
scene('gap-wrong-shorthand', 'p', '--v:red;gap:8px 10px;gap:var(--v);column-gap:4px', 'gap:8px 10px;gap:unset;column-gap:4px', prefix=BASE+'#p{flex-direction:row}')
scene('padding-wrong-shorthand', 'p', '--v:red;padding:8px;padding:var(--v);padding-right:4px', 'padding:8px;padding:unset;padding-right:4px', prefix=BASE+'#a{width:auto}')
scene('margin-wrong-shorthand', 'a', '--v:red;margin:auto;margin:var(--v);margin-left:auto', 'margin:auto;margin:unset;margin-left:auto')
scene('min-width-color', 'a', '--v:red;min-width:120px;min-width:var(--v)', 'min-width:120px;min-width:unset')
scene('max-width-color', 'a', '--v:red;max-width:40px;max-width:var(--v)', 'max-width:40px;max-width:unset')
scene('box-sizing-color-initial-content', 'a', '--v:red;padding:4px;box-sizing:border-box;box-sizing:var(--v)', 'padding:4px;box-sizing:border-box;box-sizing:unset')
scene('nbsp-color-primary', 'a', '--v:\u00a0red\u00a0;color:var(--v);background:currentColor', 'color:unset;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('nbsp-color-fallback', 'a', 'color:var(--missing,\u00a0red\u00a0);background:currentColor', 'color:unset;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('vertical-tab-color', 'a', '--v:\u000bred\u000b;color:var(--v);background:currentColor', 'color:unset;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('em-space-width', 'a', '--v:\u200340px\u2003;width:var(--v)', 'width:unset')
scene('escaped-nbsp-width', 'a', r'--v:\a0 40px;width:var(--v)', 'width:unset')
scene('css-whitespace-positive', 'a', '--v: \t\r\n\f40px \t\r\n\f;width:var(--v)', 'width:40px')
scene('important-invalid-color', 'a', '--v:20px;color:var(--v)!important;color:red;background:currentColor', 'color:unset!important;color:red;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('losing-invalid-color', 'a', '--v:20px;color:var(--v);color:maroon', 'color:unset;color:maroon')
scene('shorthand-importance', 'p', '--v:red;padding:8px!important;padding:var(--v)!important;padding-right:4px', 'padding:8px!important;padding:unset!important;padding-right:4px', prefix=BASE+'#a{width:auto}')
scene('inherited-invalid-data', 'a', '--v:red;color:var(--alias,red);background:currentColor', 'color:unset;background:currentColor', prefix=BASE+'#p{--v:20px;--alias:var(--v)}', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('unused-invalid-fallback', 'a', '--v:maroon;color:var(--v,20px)', 'color:maroon')
scene('css-wide-fallback-inherit', 'a', 'color:var(--missing,inherit);background:currentColor', 'color:inherit;background:currentColor', expected=dict(paint,a='rgb(0, 0, 128)',c='rgb(0, 0, 128)'))
scene('negative-padding', 'a', '--v:-4px;padding:8px;padding:var(--v)', 'padding:8px;padding:unset')
scene('negative-width', 'a', '--v:-40px;width:var(--v)', 'width:unset')

# Copy the independent public underflow controls exactly. The column-parent
# reset makes recovered auto width visible; em consumers expose inherited font
# recovery. Positive tiny lengths and f64-underflow zero remain distinct.
token_source = HERE / 'public-value-token-cases.json'
token_cases = {case['name']: case for case in json.loads(token_source.read_text())}
selected = [
    'variable-f32-underflow-width-unitless',
    'variable-f32-underflow-font-unitless',
    'variable-f32-underflow-width-negative-length',
    'variable-f32-underflow-font-negative-length',
    'valid-variable-positive-length',
    'valid-variable-f64-underflow-negative-length',
]
for name in selected:
    source = token_cases[name]
    assert source['outcome'] == 'equivalent'
    scenes.append(dict(name='value-tiny-' + name, html=source['html'], css=source['css'],
                       literalCss=source['literalCss'],
                       expectedPaint={'p': 'rgb(0, 128, 128)', 'a': 'rgb(255, 127, 80)', 'b': 'rgb(255, 215, 0)'},
                       classification='exact-public-underflow-source-control', sourceCase=name))
(HERE / 'public-value-native-token-source-bindings.json').write_text(json.dumps({
    'path': str(token_source), 'sha256': hashlib.sha256(token_source.read_bytes()).hexdigest(),
    'selectedNames': selected,
}, indent=2) + '\n')

for name, data in [('forms', forms), ('cases', scenes)]:
    (HERE / f'public-value-native-{name}.json').write_text(json.dumps(data, indent=2) + '\n')
print(json.dumps({'properties': len(VALUES), 'browserForms': len(forms), 'paintedCandidates': len(scenes)}))
