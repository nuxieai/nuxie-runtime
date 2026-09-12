"""Public token-boundary controls; no browser-derived compiler inputs.

`equivalent` compares the public result with separately authored literal CSS.
`unset-diagnostic` preserves a CSS initial value outside the immutable target.
Opaque functions and valid unsupported CSS remain diagnostic, not recovered.
"""
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent
HTML = '<div id="p"><div id="a"></div><div id="b"></div></div>'
BASE = '#p{width:200px;height:80px;background:teal;color:navy;font-size:24px}#a{width:40px;height:20px;background:coral}#b{width:20px;height:20px;background:gold}'
UNSUPPORTED_UNSET = {'display', 'flex', 'flex-shrink'}
cases = []


def add(name, group, css, *, literal=None, outcome='diagnostic', prop=None,
        raw=None, html=HTML, note='', rollback=None):
    case = dict(name=name, group=group, html=html, css=css, outcome=outcome,
                note=note)
    if literal is not None:
        case['literalCss'] = literal
    if prop is not None:
        case['property'] = prop
    if raw is not None:
        case['rawValue'] = raw
    if rollback is not None:
        case['rollbackCss'] = rollback
    cases.append(case)


def invalid(name, prop, value, group='boundary', *, note=''):
    add('literal-' + name, 'literal-' + group, BASE + f'#a{{{prop}:{value}}}',
        prop=prop, raw=value, note=note)
    environment = f'--bad:{value};'
    add('variable-' + name, 'variable-' + group,
        BASE + f'#a{{{environment}{prop}:var(--bad)}}',
        literal=BASE + f'#a{{{environment}{prop}:unset}}', prop=prop, raw=value,
        outcome='unset-diagnostic' if prop in UNSUPPORTED_UNSET else 'equivalent', note=note)


# These characters are identifier content or delimiters, never CSS whitespace.
noncss = [('nbsp', '\u00a0'), ('vertical-tab', '\u000b'), ('em-space', '\u2003'),
          ('narrow-nbsp', '\u202f'), ('ideographic-space', '\u3000')]
edge_values = [('width-number', 'width', '10px'), ('width-keyword', 'width', 'auto'),
               ('width-unit', 'width', '2em'), ('color-name', 'color', 'red'),
               ('background-hex', 'background-color', '#f00'),
               ('direction', 'flex-direction', 'row'), ('box-sizing', 'box-sizing', 'border-box'),
               ('order', 'order', '2'), ('padding', 'padding', '4px'), ('gap', 'gap', '4px')]
for space_name, space in noncss:
    for label, prop, value in edge_values:
        for edge, text in [('leading', space + value), ('trailing', value + space)]:
            invalid(f'{space_name}-{label}-{edge}', prop, text,
                    note=f'{space_name} is not a CSS whitespace token.')
    for label, prop, value in [
        ('dimension', 'width', '10' + space + 'px'),
        ('padding', 'padding', '1px' + space + '2px'),
        ('gap', 'gap', '1px' + space + '2px'),
        ('margin', 'margin', '0' + space + 'auto'),
        ('alignment', 'align-self', 'safe' + space + 'center'),
        ('flex', 'flex', '0' + space + '0' + space + 'auto'),
    ]:
        invalid(f'{space_name}-{label}-interior', prop, value,
                note='A non-CSS separator must not manufacture multiple components.')

for name, prop, value in [
    ('escaped-nbsp-keyword', 'color', r'\a0 red'),
    ('escaped-nbsp-unit', 'width', r'10px\a0'),
    ('escaped-em-space-keyword', 'flex-direction', r'row\2003'),
    ('escaped-narrow-nbsp-number', 'width', r'\202f 10px'),
    ('escaped-vt-keyword', 'color', r'red\b '),
    ('escaped-ascii-space-keyword', 'color', r'r\20 ed'),
    ('escaped-ascii-space-unit', 'width', r'10p\20 x'),
    ('escaped-ascii-tab-keyword', 'width', r'a\9 uto'),
    ('escaped-single-ident-flex', 'flex', r'\30 \20 \30 \20 auto'),
    ('escaped-single-ident-alignment', 'align-self', r'safe\20 center'),
    ('escaped-single-ident-padding', 'padding', r'\31 px\20 2px'),
    ('escaped-single-ident-gap', 'gap', r'\31 px\20 2px'),
    ('comment-splits-dimension', 'width', '1/**/px'),
    ('comment-splits-number', 'width', '1/**/0px'),
]:
    invalid(name, prop, value, note='Escapes decode within their original token; comments separate tokens.')

# Literal parsing must remain strict even where the classifier conservatively
# leaves a function unclassified. Variable functions are separate retained gates.
for name, prop, value in [
    ('rgb-vt-separators', 'color', 'rgb(255\u000b0\u000b0)'),
    ('rgb-nbsp-component', 'color', 'rgb(255,\u00a00,0)'),
    ('rgba-nbsp-alpha', 'background', 'rgba(255,0,0,\u00a01)'),
    ('hsl-vt-separators', 'background-color', 'hsl(0\u000b100%\u000b50%)'),
    ('hsl-nbsp-hue', 'color', 'hsl(\u00a00,100%,50%)'),
]:
    add('literal-' + name, 'literal-function-boundary', BASE + f'#a{{{prop}:{value}}}', prop=prop, raw=value)
    add('variable-' + name, 'unclassified-function', BASE + f'#a{{--bad:{value};{prop}:var(--bad)}}', prop=prop, raw=value,
        note='No broad function recovery is asserted by this conservative classifier checkpoint.')

# All currently admitted receiving property names prohibit a string value.
properties = ['width', 'height', 'min-width', 'min-height', 'max-width', 'max-height',
              'box-sizing', 'font-size', 'background', 'background-color', 'color',
              'display', 'flex-direction', 'flex', 'flex-grow', 'flex-shrink',
              'flex-basis', 'order', 'align-self', 'justify-content', 'margin',
              'margin-left', 'margin-top', 'margin-right', 'margin-bottom',
              'padding', 'padding-left', 'padding-top', 'padding-right',
              'padding-bottom', 'gap', 'row-gap', 'column-gap']
assert len(properties) == 33
for prop in properties:
    invalid('string-' + prop, prop, '"wrong"', 'primitive', note='A string token is a proven wrong primitive type.')

for name, prop, value in [
    ('width-color', 'width', 'red'), ('width-nonzero-number', 'width', '2'),
    ('height-angle', 'height', '2deg'), ('color-length', 'color', '12px'),
    ('background-number', 'background', '12'), ('order-length', 'order', '1px'),
    ('order-fraction', 'order', '1.0'), ('order-exponent', 'order', '1e2'),
    ('width-block', 'width', '()'), ('padding-brackets', 'padding', '[1px]'),
    ('gap-comma', 'gap', ','), ('width-punctuation', 'width', '/'),
    ('min-width-none', 'min-width', 'none'), ('max-width-auto', 'max-width', 'auto'),
    ('padding-too-many', 'padding', '1px 2px 3px 4px 5px'),
    ('gap-too-many', 'gap', '1px 2px 3px'), ('row-gap-too-many', 'row-gap', '1px 2px'),
    ('flex-too-many', 'flex', '0 0 auto extra'), ('order-too-many', 'order', '1 2'),
    ('alignment-too-many', 'align-self', 'safe center center'),
    ('width-too-many', 'width', '1px 2px'), ('color-too-many', 'color', 'red blue'),
    ('css-wide-extra', 'width', 'initial 1px'),
    ('padding-css-wide-mixed', 'padding', 'initial 1px'),
    ('gap-css-wide-mixed', 'gap', 'unset 1px'),
]:
    invalid(name, prop, value, 'primitive')
for prop in ['width', 'height', 'min-width', 'min-height', 'max-width', 'max-height',
             'font-size', 'flex-basis', 'padding', 'padding-left', 'gap', 'row-gap']:
    invalid('negative-' + prop, prop, '-1px', 'primitive', note='Negative length is prohibited by this receiving grammar.')
for prop in ['flex-grow', 'flex-shrink']:
    invalid('negative-' + prop, prop, '-1', 'primitive')

# This is invalid declaration syntax, rather than a value eligible for receiving-
# grammar recovery: CSS Variables 1 §2.1 excludes top-level ! delimiters.
# Preserve the exact sources from r2 as strict diagnostics after replacing the
# intended primitive punctuation probe above with a valid custom-property '/'.
for name, declarations in [('literal', 'width:!'), ('variable', '--bad:!;width:var(--bad)')]:
    add('syntax-'+name+'-bare-important-delimiter', 'retained-diagnostic',
        BASE+f'#a{{{declarations}}}', prop='width', raw='!',
        note='https://www.w3.org/TR/css-variables-1/#syntax prohibits top-level ! in declaration-value.')

# f32 token values alone cannot prove CSS zero or the sign of an authored value.
# Chrome153 grammar-probes-r2 distinguishes these nonzero f64 coefficients from
# the zero-rounded 1e-500 controls below. The column parent makes width:unset
# stretch to 200px, so incorrectly accepting native zero is observable.
for name, value in [('unitless', '1e-50'), ('negative-length', '-1e-50px')]:
    invalid('f32-underflow-width-'+name, 'width', value, 'primitive',
            note='Chrome rejects this nonzero authored coefficient even though cssparser rounds it to f32 zero.')
    add('literal-f32-underflow-font-'+name, 'literal-primitive',
        BASE+f'#a{{font-size:{value};width:2em}}', prop='font-size', raw=value)
    add('variable-f32-underflow-font-'+name, 'variable-primitive',
        BASE+f'#a{{--bad:{value};font-size:var(--bad);width:2em}}',
        literal=BASE+f'#a{{--bad:{value};font-size:unset;width:2em}}',
        outcome='equivalent', prop='font-size', raw=value,
        note='An em consumer distinguishes inherited 24px font size from wrongly accepted zero.')

for name, value, literal in [('positive-length', '1e-50px', '0px'),
                            ('f64-underflow-negative-length', '-1e-500px', '-0px')]:
    for form, declarations in [('literal', f'width:{value}'),
                               ('variable', f'--v:{value};width:var(--v)')]:
        custom = '' if form == 'literal' else f'--v:{value};'
        add('valid-'+form+'-'+name, 'valid-boundary', BASE+f'#a{{{declarations}}}',
            literal=BASE+f'#a{{{custom}width:{literal}}}', outcome='equivalent', prop='width', raw=value,
            note='Valid CSS zero/native-zero control; this does not admit arbitrary nonzero unitless lengths.')

for name, text in [('nbsp', '\u00a0'), ('nbsp-entity', '&nbsp;'),
                   ('vertical-tab', '\u000b'), ('vertical-tab-entity', '&#11;')]:
    for position, markup in [('nested', f'<div id="a">{text}</div>'), ('root', text)]:
        add(f'html-non-whitespace-{name}-{position}', 'literal-boundary', BASE,
            html=markup, note='Non-HTML whitespace is text and requires intentionally unsupported text rendering.')

# Recovery must retain the winning declaration, priority, inherited environment,
# shorthand resets and order prepass, rather than roll back the CSS cascade.
def recovery(name, authored, literal, *, prefix=BASE, target='a', rollback=None):
    add(name, 'cascade-recovery', prefix + f'#{target}{{{authored}}}',
        literal=prefix + f'#{target}{{{literal}}}', outcome='equivalent',
        rollback=prefix + f'#{target}{{{rollback}}}' if rollback is not None else None)

recovery('invalid-winner-width', '--bad:red;width:31px;width:var(--bad)',
         '--bad:red;width:31px;width:unset', rollback='--bad:red;width:31px')
recovery('invalid-important-background', '--bad:"wrong";background:var(--bad)!important;background:coral',
         '--bad:"wrong";background:unset!important;background:coral', rollback='--bad:"wrong";background:coral')
recovery('invalid-order-prepass', '--bad:1px;order:-4;order:var(--bad,2)',
         '--bad:1px;order:-4;order:unset', prefix=BASE+'#b{order:-1}', rollback='--bad:1px;order:-4')
recovery('invalid-order-important', '--bad:1px;order:var(--bad)!important;order:-4',
         '--bad:1px;order:unset!important;order:-4', prefix=BASE+'#b{order:-1}', rollback='--bad:1px;order:-4')
recovery('invalid-inherited-color', '--bad:2px;color:red;color:var(--bad);background:currentColor',
         '--bad:2px;color:red;color:unset;background:currentColor', rollback='color:red;background:currentColor')
recovery('invalid-inherited-font-size', '--bad:red;font-size:10px;font-size:var(--bad);width:2em',
         '--bad:red;font-size:10px;font-size:unset;width:2em', rollback='font-size:10px;width:2em')
recovery('invalid-inherited-alias', '--bad:30px;width:var(--alias,20px)', '--bad:30px;width:unset',
         prefix=BASE+'#p{--bad:red;--alias:var(--bad)}', rollback='width:20px')
recovery('invalid-nested-fallback', 'width:var(--missing,var(--also-missing,red))', 'width:unset', rollback='width:20px')
recovery('invalid-gap-shorthand-longhand', '--bad:red;gap:8px;gap:var(--bad);column-gap:4px',
         '--bad:red;gap:8px;gap:unset;column-gap:4px', target='p', rollback='gap:8px;column-gap:4px')
recovery('invalid-padding-important-shorthand', '--bad:red;padding:8px!important;padding:var(--bad)!important;padding-right:4px',
         '--bad:red;padding:8px!important;padding:unset!important;padding-right:4px', target='p', rollback='padding:8px!important;padding-right:4px')
recovery('invalid-losing-variable', '--bad:red;width:var(--bad);width:40px', '--bad:red;width:unset;width:40px')
recovery('valid-primary-unused-invalid-fallback', '--good:30px;width:var(--good,red)', '--good:30px;width:30px')
recovery('valid-primary-unused-unclassified-fallback', '--good:30px;width:var(--good,calc(5px + 6px))', '--good:30px;width:30px')

# Actual CSS whitespace and comments remain accepted around and between tokens.
for name, space in [('space', ' '), ('tab', '\t'), ('lf', '\n'), ('cr', '\r'), ('ff', '\f'), ('comment', '/**/')]:
    for label, prop, value, literal in [
        ('width', 'width', space+'10px'+space, '10px'),
        ('padding', 'padding', '1px'+space+'2px', '1px 2px'),
        ('gap', 'gap', '1px'+space+'2px', '1px 2px'),
        ('flex', 'flex', '0'+space+'0'+space+'auto', '0 0 auto'),
        ('alignment', 'align-self', 'safe'+space+'center', 'safe center'),
        ('rgb', 'background', 'rgb(255'+space+'0'+space+'0)', 'red'),
    ]:
        add(f'valid-{name}-{label}', 'valid-boundary', BASE+f'#a{{{prop}:{value}}}',
            literal=BASE+f'#a{{{prop}:{literal}}}', outcome='equivalent', prop=prop, raw=value)

for name, prop, value, literal in [
    ('ascii-color', 'color', r'r\65 d', 'red'),
    ('ascii-unit', 'width', r'10p\78', '10px'),
    ('ascii-keyword', 'flex-direction', r'r\6f w', 'row'),
    ('ascii-alignment', 'align-self', r'safe c\65 nter', 'safe center'),
    ('ascii-flex', 'flex', r'0 0 a\75 to', '0 0 auto'),
    ('ascii-padding-unit', 'padding', r'1p\78  2px', '1px 2px'),
]:
    add('valid-escaped-'+name, 'valid-boundary', BASE+f'#a{{{prop}:{value}}}',
        literal=BASE+f'#a{{{prop}:{literal}}}', outcome='equivalent', prop=prop, raw=value)
    add('valid-variable-escaped-'+name, 'valid-boundary', BASE+f'#a{{--v:{value};{prop}:var(--v)}}',
        literal=BASE+f'#a{{--v:{value};{prop}:{literal}}}', outcome='equivalent', prop=prop, raw=value)

for name, ident, reference in [
    ('nbsp', '--x\u00a0', '--x\u00a0'), ('em-space', '--x\u2003', '--x\u2003'),
    ('narrow-nbsp', '--x\u202f', '--x\u202f'), ('ideographic-space', '--x\u3000', '--x\u3000'),
    ('accent', '--café', '--café'), ('escaped-nbsp', r'--x\a0', '--x\u00a0'),
    ('escaped-ascii-space', r'--x\20', r'--x\20'),
]:
    custom = f'--x:20px;{ident}:30px;'
    add('valid-unicode-name-'+name, 'unicode-custom-name', BASE+f'#a{{{custom}width:var({reference},50px)}}',
        literal=BASE+f'#a{{{custom}width:30px}}', outcome='equivalent',
        note='Custom-property identifiers retain non-ASCII and escaped character identity.')

# Percentage font sizing is already supported. Keep substitution equivalent to
# an independently authored percentage, with a font-relative width consumer.
add('valid-variable-font-percentage', 'valid-boundary',
    BASE+'#a{--v:120%;font-size:var(--v);width:2em}',
    literal=BASE+'#a{--v:120%;font-size:120%;width:2em}',
    outcome='equivalent', prop='font-size', raw='120%')

# Syntactically valid CSS beyond target capabilities must remain diagnostic even
# through substitution; invalid-at-computed-value recovery is not feature support.
unsupported = [
    ('viewport-unit', 'width', '1vw'), ('absolute-unit', 'width', '1cm'),
    ('intrinsic-width', 'width', 'min-content'), ('intrinsic-max', 'max-width', 'max-content'),
    ('font-keyword', 'font-size', 'larger'),
    ('negative-margin', 'margin', '-1px'), ('positive-margin', 'margin-left', '2px'),
    ('grid', 'display', 'grid'), ('flex-growth', 'flex', '1 1 0px'),
    ('background-position', 'background', 'left top red'),
    ('background-position-length', 'background', '2px'),
    ('background-text-clip', 'background', 'text'),
    ('background-border-area-clip', 'background', 'border-area'),
    ('calc-width', 'width', 'calc(10px + 5px)'), ('calc-gap', 'gap', 'calc(1px + 2px)'),
    ('unclassified-function', 'width', 'mystery(1px)'),
    ('modern-color', 'color', 'lab(50% 0 0)'),
    ('resource-length', 'width', '1000001px'), ('resource-order', 'order', '2147483648'),
    ('resource-nonfinite', 'width', '1e999px'),
    ('f64-underflow-unitless-zero', 'width', '1e-500'),
    ('revert-keyword', 'width', 'revert'), ('revert-layer-keyword', 'width', 'revert-layer'),
    ('revert-rule-keyword', 'width', 'revert-rule'),
]
for name, prop, value in unsupported:
    for form, declarations in [('literal', f'{prop}:{value}'), ('variable', f'--v:{value};{prop}:var(--v)')]:
        add(f'unsupported-{form}-{name}', 'retained-diagnostic', BASE+f'#a{{{declarations}}}', prop=prop, raw=value,
            note='Valid unsupported, unclassified grammar, or resource boundary; must not become unset.')

for name, declarations in [
    ('unused-custom', '--v:revert-rule;width:30px'),
    ('variable-fallback', 'width:var(--missing,revert-rule)'),
]:
    add('unsupported-revert-rule-'+name, 'retained-diagnostic', BASE+f'#a{{{declarations}}}',
        prop='width', raw='revert-rule',
        note='CSS-wide revert-rule requires cascade behavior not implemented by this target, including custom declarations.')

for value in ['text', 'border-area']:
    add('unsupported-background-clip-fallback-'+value, 'retained-diagnostic',
        BASE+f'#a{{background:var(--missing,{value})}}', prop='background', raw=value,
        note='Valid background clipping syntax is outside the solid-paint target and must not recover to unset.')

# Historical sources remain untouched. These hand-authored replacements describe
# only explicitly qualified changed outcomes, independently of the production
# classifier. Existing tests verify source identity before using these controls.
historical = {
    'public-content-box-rejections.json': {
        'strict-variable': ('box-sizing:var(--bad)', 'box-sizing:unset'),
    },
    'public-variable-recovery-rejections.json': {
        'typed-empty-fallback': ('width:var(--missing,)', 'width:unset'),
        'typed-empty-primary': ('width:var(--empty,25px)', 'width:unset'),
        'typed-token-boundary': ('width:var(--x)px', 'width:unset'),
    },
    'public-empty-variable-rejections.json': {
        'empty-reject-nonempty-token-boundary': ('width:var(--e)px', 'width:unset'),
        'empty-reject-quoted-empty-string': ('width:var(--e)', 'width:unset'),
        'empty-reject-empty-block': ('width:var(--e)', 'width:unset'),
        'empty-reject-comma-token': ('width:var(--e)', 'width:unset'),
        'empty-reject-non-css-nbsp': ('width:var(--e)', 'width:unset'),
        'empty-reject-non-css-vertical-tab': ('width:var(--e)', 'width:unset'),
        'empty-reject-fallback-non-css-nbsp': ('width:var(--missing,\u00a0)', 'width:unset'),
        'empty-reject-fallback-non-css-vertical-tab': ('width:var(--missing,\u000b)', 'width:unset'),
        'empty-reject-escaped-non-css-nbsp': ('width:var(--e)', 'width:unset'),
        'empty-reject-fallback-escaped-non-css-nbsp': (r'width:var(--missing,\a0)', 'width:unset'),
        'empty-reject-nonempty-invalid-loser': ('width:var(--e)', 'width:unset'),
        'empty-reject-nonempty-suffix-after-empty': ('width:var(--e)px', 'width:unset'),
    },
    'public-numeric-token-rejections.json': {
        'numeric-token-reject-number-unit-fragments': ('width:var(--n)px', 'width:unset'),
        'numeric-token-reject-variable-unit-fragments': ('width:var(--n)var(--u)', 'width:unset'),
        'numeric-token-reject-exponent-fragments': ('width:var(--n)var(--u)', 'width:unset'),
        'numeric-token-reject-percentage-fragments': ('width:var(--n)%', 'width:unset'),
        'numeric-token-reject-order-0-variable': ('order:var(--o)', 'order:unset'),
        'numeric-token-reject-order-1-variable': ('order:var(--o)', 'order:unset'),
        'numeric-token-reject-order-4-variable': ('order:var(--o)', 'order:unset'),
        'numeric-token-reject-order-5-variable': ('order:var(--o)', 'order:unset'),
        'numeric-token-reject-order-6-variable': ('order:var(--o)', 'order:unset'),
    },
}
for filename, changes in historical.items():
    originals = {c['name']: c for c in json.loads((OUT/filename).read_text())}
    for name, (before, after) in changes.items():
        original = originals[name]
        assert original['css'].count(before) == 1, (filename, name)
        add('historical-'+filename.removeprefix('public-').removesuffix('.json')+'-'+name,
            'historical-recovery', original['css'], html=original['html'],
            literal=original['css'].replace(before, after), outcome='equivalent',
            note='Exact historical source retained; explicit unset is the independently authored changed expectation.')
        cases[-1].update(historyFile=filename, historyName=name)

assert len({c['name'] for c in cases}) == len(cases)
(OUT/'public-value-token-cases.json').write_text(json.dumps(cases, indent=2, ensure_ascii=True)+'\n')
counts = {group: sum(c['group'] == group for c in cases) for group in sorted({c['group'] for c in cases})}
print(json.dumps({'total': len(cases), 'groups': counts}, indent=2))
