"""Independent source corpus for the private ordinary content-owner experiment.

This file creates authored HTML/CSS only. No compiler lowering, browser-derived
dimensions, public admission change or runtime mutation is performed here.
"""
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent
old = json.loads((OUT/'public-content-box-cases.json').read_text())
rejected = json.loads((OUT/'public-content-box-rejections.json').read_text())
cases = []


def add(name, html, css, purpose, classification='core', origin=None):
    case = dict(name=name, html=html, css=css, purpose=purpose,
                classification=classification,
                baselineExpected='diagnostic' if classification == 'proposed-diagnostic' else 'success',
                candidateProfile='outside-initial-point-profile' if classification == 'proposed-diagnostic'
                else 'point-content-owner-experiment')
    if origin:
        case['retainedSource'] = origin
    cases.append(case)


for original in old:
    if original['classification'] == 'numeric-boundary' or original['name'] == 'content-point-fractional-paint-control':
        add(original['name'], original['html'], original['css'],
            'Exact original public source; preserve known numeric/paint limitations without claiming qualification.',
            'retained-large' if original['classification'] == 'numeric-boundary' else 'retained-fractional',
            dict(file='public-content-box-cases.json', name=original['name']))

html = '<div id="p"><div id="a"><div id="leaf"></div></div><div id="b"></div></div>'
for direction in ['row', 'row-reverse', 'column', 'column-reverse']:
    add('owner-direction-'+direction, html,
        f'#p{{box-sizing:content-box;width:128px;height:70px;padding:7px 13px 11px 5px;flex-direction:{direction};background:#dbeafe}}'
        '#a{width:40px;height:20px;background:#dc2626}#leaf{width:100%;height:10px;background:#fbbf24}#b{width:60px;height:26px;background:#3b82f6}',
        'Original child flow and paint order must survive outer normal-column packing and an unpainted inner owner.')

pair = '<div id="p"><div id="c"></div></div>'
add('owner-small-padding-dominant', pair,
    '#p{box-sizing:content-box;width:8px;height:24px;padding:0 2px 0 96px;background:#14233f}#c{width:100%;height:16px;background:#669933}',
    'Visible point-sized analogue: padding dominates the authored content width.')
add('owner-small-fractional-outer', pair,
    '#p{box-sizing:content-box;width:127.875px;height:24px;padding:0 32px;background:#14233f}#c{width:100%;height:16px;background:#669933}',
    'Visible fractional analogue; retain actual paint differences rather than widening gates.', 'retained-fractional')
add('owner-zero-content-padding-floor', pair,
    '#p{box-sizing:content-box;width:0px;height:0px;padding:8px 12px;background:#14233f}#c{width:100%;height:0px;background:#669933}',
    'A zero content box still has a painted padding floor; an inner owner must not double the inset.')

for name, dimensions, purpose in [
    ('fixed-max-clamp', 'width:120px;min-width:20px;max-width:72px;height:50px;max-height:32px',
     'Preferred point width/height are clamped by authored content maxima before percentages consume them.'),
    ('minimum-wins-maximum', 'width:10px;min-width:84px;max-width:48px;height:10px;min-height:36px;max-height:20px',
     'Both axes retain CSS minimum precedence over conflicting maxima.'),
    ('auto-width-point-maximum', 'width:auto;max-width:140px;height:30px',
     'Auto cross-axis width with a point maximum remains a falsifiable candidate; viewport resize must stay live.'),
    ('fixed-height-minimum', 'width:90px;height:8px;min-height:46px',
     'A point minimum changes content height without counting padding as part of that authored minimum.'),
]:
    add('owner-'+name, pair,
        '#p{box-sizing:content-box;'+dimensions+';padding:5px 9px;background:#dbeafe}#c{width:100%;height:100%;background:#669933}',
        purpose)

siblings = '<div id="p"><div id="a"></div><div id="b"></div></div>'
add('owner-auto-height-column', siblings,
    '#p{box-sizing:content-box;width:118px;height:auto;padding:6px 10px;background:#dbeafe}#a{width:100%;height:17px;background:#dc2626}#b{width:100%;height:23px;background:#3b82f6}',
    'Column content height is the intrinsic sum of fixed child heights, with padding added only by the outer owner.')
add('owner-auto-height-row', siblings,
    '#p{box-sizing:content-box;width:118px;height:auto;padding:6px 10px;flex-direction:row;background:#dbeafe}#a{width:40px;height:17px;background:#dc2626}#b{width:46px;height:31px;background:#3b82f6}',
    'Row content height is the intrinsic cross-axis child maximum; an inner owner must not acquire definite height accidentally.')
add('owner-intrinsic-auto-width-in-row', '<div id="stage">'+siblings+'<div id="s"></div></div>',
    '#stage{width:230px;height:90px;flex-direction:row;background:#e2e8f0}#p{box-sizing:content-box;width:auto;height:48px;padding:6px 10px;flex-direction:row;background:#dbeafe}'
    '#a{width:40px;height:17px;background:#dc2626}#b{width:46px;height:31px;background:#3b82f6}#s{width:60px;height:60px;background:#669933}',
    'Intrinsic main-axis width must be measured from authored children, not the outer padding box or viewport.')

nested = '<div id="p"><div id="a"><div id="b"></div></div></div>'
add('owner-nested-percentage-descendants', nested,
    '#p{box-sizing:content-box;width:160px;height:80px;padding:8px 10px;background:#dbeafe}#a{width:70%;height:50%;background:#669933}#b{width:50%;height:50%;background:#663399}',
    'Nested percentage children consume the correct content owner at each depth on both axes.')
add('owner-percentage-border-box-descendant', nested,
    '#p{box-sizing:content-box;width:160px;height:80px;padding:7px 11px;background:#dbeafe}#a{width:100%;height:100%;padding:3px;background:#669933}#b{width:100%;height:50%;background:#663399}',
    'The authored percentage border-box child retains its own padding and a separate descendant percentage basis.')
add('owner-inherited-content-width', nested,
    '#p{box-sizing:content-box;width:96px;height:64px;padding:8px;background:#dbeafe}#a{box-sizing:content-box;width:inherit;height:28px;padding:4px;background:#669933}#b{width:inherit;height:8px;background:#663399}',
    'Width inheritance copies authored computed content dimensions, never the emitted outer dimension.')
add('owner-inherited-box-and-padding', nested,
    '#p{box-sizing:content-box;width:112px;height:68px;padding:3px 7px;background:#dbeafe}#a{box-sizing:inherit;padding:inherit;width:44px;height:24px;background:#669933}#b{width:100%;height:50%;background:#663399}',
    'Inherited box sizing and physical padding remain authored style before each owner lowers independently.')
add('owner-inherited-content-minmax', nested,
    '#p{box-sizing:content-box;width:90px;min-width:80px;max-width:120px;height:48px;padding:4px;background:#dbeafe}'
    '#a{box-sizing:content-box;width:20px;min-width:inherit;max-width:inherit;height:24px;padding:2px;background:#669933}#b{width:100%;height:10px;background:#663399}',
    'Inherited content bounds stay separate from outer padding-adjusted bounds.')

add('owner-overflow-across-alpha-sibling', '<div id="stage"><div id="p"><div id="a"></div></div><div id="s"></div></div>',
    '#stage{width:100%;height:100px;flex-direction:row;background:#e2e8f0}#p{box-sizing:content-box;width:80px;height:40px;padding:8px 10px;background:rgba(20,80,160,.5)}'
    '#a{width:140px;height:24px;background:rgba(240,80,30,.55)}#s{width:80px;height:64px;background:rgba(30,150,90,.6)}',
    'An overflowing child crosses a sibling boundary; helper ownership must retain paint ordering and alpha composition.')
add('owner-nested-alpha-backgrounds', nested,
    '#p{box-sizing:content-box;width:100px;height:50px;padding:8px 12px;background:rgba(20,80,160,.5)}'
    '#a{width:100%;height:32px;background:rgba(240,80,30,.5)}#b{width:40px;height:40px;background:rgba(30,150,90,.5)}',
    'Outer padding/background paints exactly once; the content owner is unpainted even with overflowing translucent descendants.')
add('owner-ordered-overflow-siblings', '<div id="stage"><div id="p"><div id="a"></div></div><div id="q"></div></div>',
    '#stage{width:100%;height:100px;flex-direction:row;background:#e2e8f0}#p{box-sizing:content-box;width:60px;height:40px;padding:6px;order:1;background:#dbeafe}'
    '#a{width:120px;height:28px;background:rgba(220,38,38,.7)}#q{width:50px;height:56px;order:2;background:rgba(59,130,246,.7)}',
    'Source IDs, child attachment, order sorting and overflow paint remain observable across an adjacent authored sibling.')

assert len(cases) == 28, len(cases)
for original in rejected:
    if original['name'] in ['mixed-width', 'mixed-height', 'mixed-minimum', 'mixed-maximum', 'percentage-padding']:
        add('proposed-'+original['name'], original['html'], original['css'],
            'Exact currently diagnostic source. Percentage owner dimensions/insets require a separate containing-block composition and are outside the initial point-owner experiment.',
            'proposed-diagnostic', dict(file='public-content-box-rejections.json', name=original['name']))

assert len(cases) == 33
assert len({c['name'] for c in cases}) == len(cases)
(OUT/'content-owner-cases.json').write_text(json.dumps(cases, indent=2)+'\n')
print(json.dumps({'coreCases':28, 'proposedDiagnosticCases':5, 'total':len(cases)}))
