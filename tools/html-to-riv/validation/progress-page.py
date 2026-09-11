"""Generate the local progress page from BACKLOG.md and explicit current evidence.

Run with --watch to regenerate when either input changes. No tests or qualification
claims are inferred from process names, elapsed time, or the existence of a file.
"""
from pathlib import Path
from collections import Counter
from html import escape
import argparse
import json
import os
import re
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
MODULE = ROOT / 'tools/html-to-riv'
STATE = MODULE / 'validation/progress-state.json'
BACKLOG = MODULE / 'BACKLOG.md'
OUTPUT = MODULE / 'test-results/progress.html'
LABELS = {'A':'Authoring & basic typography','S':'Selectors & cascade','L':'Layout & positioning',
          'P':'Painting','I':'Image assets','T':'Rich typography','R':'Responsive CSS','Q':'Compiler & validation quality'}


def build():
    state = json.loads(STATE.read_text())
    rows = []
    for line in BACKLOG.read_text().splitlines():
        cells = [v.strip() for v in line.split('|')[1:-1]]
        if len(cells) >= 4 and re.fullmatch(r'[ASLPITRQ]\d{2}[a-z]?', cells[0]):
            bucket = 'qualified' if 'qualified' in cells[2] else 'pending' if cells[2] == 'pending' else 'partial'
            rows.append((*cells[:4], bucket))
    counts = Counter(r[4] for r in rows)
    def link(path, label):
        target = ROOT / path
        if not target.is_file():
            return f'<span>{escape(label)} · evidence unavailable</span>'
        return f'<a href="{escape(os.path.relpath(target, OUTPUT.parent), quote=True)}">{escape(label)}</a>'
    groups = []
    for prefix, title in LABELS.items():
        items = [r for r in rows if r[0].startswith(prefix)]
        tally = Counter(r[4] for r in items)
        bars = ''.join(f'<span class="{key}" style="flex:{tally[key]}"></span>' for key in ('qualified','partial','pending') if tally[key])
        entries = ''.join(f'<details><summary><span class="id">{escape(i)}</span><strong>{escape(name)}</strong><span class="badge {bucket}">{escape(status)}</span></summary><p>{escape(note)}</p></details>' for i,name,status,note,bucket in items)
        groups.append(f'<section><div class="group-title"><h2>{escape(title)}</h2><span>{tally["qualified"]} qualified · {tally["partial"]} partial · {tally["pending"]} pending</span></div><div class="bar" aria-hidden="true">{bars}</div>{entries}</section>')
    html = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="refresh" content="60"><title>HTML → Rive · Compiler progress</title><link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%2310232c'/%3E%3Cpath d='M7 16h18m-7-7 7 7-7 7' stroke='%2372ddbb' stroke-width='3' fill='none'/%3E%3C/svg%3E"><style>
*{box-sizing:border-box}body{margin:0;background:#f3f5f4;color:#172f37;font:15px/1.55 system-ui,sans-serif}main{max-width:1120px;margin:auto;padding:38px 24px 70px}header{display:flex;justify-content:space-between;gap:20px;align-items:baseline}h1{font-size:30px;letter-spacing:-1px;margin:0}h2{font-size:18px;margin:0}.muted,small{color:#586c73}.stats{display:flex;gap:1px;background:#dce3e1;border:1px solid #dce3e1;border-radius:10px;overflow:hidden;margin:25px 0}.stat{flex:1;background:white;padding:18px}.stat b{display:block;font-size:34px;line-height:1.2}.current{background:#102b34;color:#f4faf7;padding:25px;border-radius:10px}.current small{color:#91cdbb}.current h2{font-size:23px;margin:4px 0}.current p{max-width:900px}.current .next{color:#b6e3d6}.baseline{border-left:3px solid #cf954b;padding:5px 15px;margin:22px 0}.evidence{display:flex;flex-wrap:wrap;gap:9px 24px;margin:22px 0}a{color:#14634e;text-underline-offset:3px}section{background:white;border:1px solid #dce3e1;border-radius:10px;padding:20px;margin-top:20px}.group-title{display:flex;justify-content:space-between;align-items:baseline;gap:12px}.group-title>span{color:#586c73;font-size:13px}.bar{height:5px;display:flex;gap:2px;margin:15px 0}.qualified{background:#d9eee5;color:#215c47}.partial{background:#fff0ce;color:#785115}.pending{background:#e9edef;color:#52646c}.bar .qualified{background:#438e71}.bar .partial{background:#d8a345}.bar .pending{background:#cbd5d9}details{border-top:1px solid #e9eeec}summary{cursor:pointer;display:flex;align-items:center;gap:12px;padding:12px 0}.id{font:12px ui-monospace,monospace;color:#687b80;width:38px}summary strong{font-weight:500;flex:1}.badge{font-size:11px;border-radius:4px;padding:3px 8px;white-space:nowrap}details p{font-size:13px;color:#4d6269;margin:0 0 18px 50px;overflow-wrap:anywhere}footer{margin-top:28px;color:#586c73;font-size:13px}@media(max-width:650px){main{padding:22px 14px}header,.group-title{display:block}h1{font-size:25px}.stat{padding:12px 8px;font-size:12px}.stat b{font-size:27px}.badge{max-width:110px;white-space:normal}section{padding:14px}.current{padding:18px}}
</style><main>'''
    html += f'<header><h1>HTML → Rive <span class="muted">/ progress</span></h1><small>Evidence updated {escape(state["updated"])}</small></header>'
    html += '<div class="stats">'+''.join(f'<div class="stat"><b>{n}</b>{label}</div>' for n,label in [(len(rows),'Tracked items'),(counts['qualified'],'Qualified in stated scope'),(counts['partial'],'Partial / active'),(counts['pending'],'Pending')])+'</div>'
    html += '<p class="muted">Item counts are not an estimate of remaining effort. Native-only qualifications and unresolved renderer differences remain explicit below.</p>'
    html += f'<article class="current"><small>{escape(state["status"])}</small><h2>{escape(state["focus"])}</h2><p>{escape(state["current"])}</p><p class="next">Next: {escape(state["next"])}</p></article>'
    html += f'<p class="baseline">{escape(state["baseline"])}</p><nav class="evidence" aria-label="Validation evidence">'+''.join(link(e['path'], e['label']) for e in state['evidence'])+'</nav>'
    html += '<section><h2>Parallel work</h2>' + ''.join('<p><strong>' + escape(item['name']) + '</strong> · ' + escape(item['work']) + '</p>' for item in state.get('activeWork', [])) + '</section>'
    html += ''.join(groups)
    html += '<footer>Reference: pinned Chrome. Excluded: CSS Grid, editor integration, scripting, interactions, bindings and animation.<br>Generated from BACKLOG.md and progress-state.json. This local page reloads every 60 seconds; the evidence date changes only when the status is updated.</footer></main></html>'
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', dir=OUTPUT.parent,
            prefix='progress-', suffix='.html.tmp', delete=False) as handle:
        temp=Path(handle.name)
        handle.write(html)
    try:
        temp.replace(OUTPUT)
    finally:
        temp.unlink(missing_ok=True)
    print(json.dumps({'page':str(OUTPUT),'total':len(rows),**counts}),flush=True)


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--watch',action='store_true');args=parser.parse_args()
    stamp=None
    while True:
        current=(BACKLOG.stat().st_mtime_ns,STATE.stat().st_mtime_ns)
        if current != stamp: build();stamp=current
        if not args.watch: break
        time.sleep(2)
