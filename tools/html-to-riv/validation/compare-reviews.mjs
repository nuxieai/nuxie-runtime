// Compare authoritative gallery metadata without treating numbers as visual review.
import fs from 'node:fs';
import path from 'node:path';
const [baselinePath,candidatePath]=process.argv.slice(2);
if(!baselinePath||!candidatePath)throw new Error('Expected baseline and candidate review.json paths');
const baseline=JSON.parse(fs.readFileSync(baselinePath)),candidate=JSON.parse(fs.readFileSync(candidatePath));
const key=c=>`${c.project}:${c.name}:${c.width}`;
const before=new Map(baseline.cases.map(c=>[key(c),c]));
if(before.size!==baseline.cases.length)throw new Error('Duplicate baseline cases');
const seen=new Set(),added=[],removed=[],regressions=[],improvements=[],unchangedFailures=[],sourceChanges=[],metricChanges=[];
for(const c of candidate.cases){
 const id=key(c);if(seen.has(id))throw new Error('Duplicate candidate case');seen.add(id);
 const b=before.get(id);if(!b){added.push(id);continue;}
 if(b.html!==c.html||b.css!==c.css)sourceChanges.push(id);
 if(b.status==='passed'&&c.status!=='passed')regressions.push(id);
 else if(b.status!=='passed'&&c.status==='passed')improvements.push(id);
 else if(c.status!=='passed')unchangedFailures.push(id);
 if(JSON.stringify(b.metrics)!==JSON.stringify(c.metrics))metricChanges.push(id);
}
for(const id of before.keys())if(!seen.has(id))removed.push(id);
const report={baseline:path.resolve(baselinePath),candidate:path.resolve(candidatePath),
 baselineCount:before.size,candidateCount:seen.size,added,removed,sourceChanges,regressions,improvements,unchangedFailures,metricChanges,
 candidateFailures:candidate.cases.filter(c=>c.status!=='passed').map(c=>({key:key(c),errors:c.errors})),
 visualReviewComplete:false};
const output=path.join(path.dirname(candidatePath),'baseline-comparison.json');
fs.writeFileSync(output,JSON.stringify(report,null,2));
console.log(JSON.stringify(Object.fromEntries(Object.entries(report).map(([k,v])=>[k,Array.isArray(v)?{count:v.length,...(['regressions','improvements','unchangedFailures','sourceChanges'].includes(k)?{cases:v}:{})}:v])),null,2));
