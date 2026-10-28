import test from 'node:test';
import assert from 'node:assert/strict';
import {createAudioHost} from './host.mjs';
import {SharedStore,createImports} from './shared.mjs';

// Transport lifecycle coverage; actual DSP is qualified by the browser probe.
const contexts=[],nodes=[];
class Context {
    constructor(){this.state='running';this.sampleRate=48000;this.destination={};this.listeners=new Set();this.audioWorklet={addModule:async()=>{}};this.closeCalls=0;contexts.push(this);}
    addEventListener(_,fn){this.listeners.add(fn);}
    removeEventListener(_,fn){this.listeners.delete(fn);}
    async resume(){this.state='running';}
    async close(){this.closeCalls++;this.state='closed';for(const fn of [...this.listeners])fn();}
}
class Node {
    constructor(context){this.context=context;this.messages=[];this.port={postMessage:m=>this.messages.push(m),onmessage:null};nodes.push(this);}
    connect(){}
    disconnect(){this.disconnected=true;}
}
const settle=()=>new Promise(resolve=>setImmediate(resolve));
const start=host=>host.call({op:'processor.start',name:'tone',modules:{tone:[1]},input:{bytes:[0],holds:[]},format:{channels:2,maxFrames:128},hardBlocks:2,memoryLimit:1000000});

test('owned output recovers and stale events cannot mutate the new generation',async()=>{
    const original={AudioContext:globalThis.AudioContext,AudioWorkletNode:globalThis.AudioWorkletNode,crossOriginIsolated:globalThis.crossOriginIsolated};
    Object.assign(globalThis,{AudioContext:Context,AudioWorkletNode:Node,crossOriginIsolated:true});
    const host=createAudioHost('unused',{module:{}});
    try {
        const first=start(host);await settle();
        const old=nodes.at(-1),oldMessage=old.port.onmessage;
        old.onprocessorerror();
        assert.equal(host.call({op:'processor.state',id:first.id}).state,'failed');
        assert.equal(old.context.closeCalls,1);
        const second=start(host);await settle();
        const fresh=nodes.at(-1);assert.notEqual(fresh.context,old.context);
        oldMessage({data:{id:second.id,kind:'failed',text:'stale'}});
        assert.equal(host.call({op:'processor.state',id:second.id}).state,'starting');
        fresh.port.onmessage({data:{id:second.id,kind:'running'}});
        assert.equal(host.call({op:'processor.state',id:second.id}).state,'running');
    }finally{host.dispose();Object.assign(globalThis,original);}
});

test('provided AudioContext survives output recovery and disposal',async()=>{
    const original={AudioContext:globalThis.AudioContext,AudioWorkletNode:globalThis.AudioWorkletNode,crossOriginIsolated:globalThis.crossOriginIsolated};
    Object.assign(globalThis,{AudioContext:Context,AudioWorkletNode:Node,crossOriginIsolated:true});
    const context=new Context(),host=createAudioHost('unused',{module:{},audioContext:context});
    try {
        start(host);await settle();nodes.at(-1).onprocessorerror();
        assert.equal(context.closeCalls,0);
        start(host);await settle();assert.equal(nodes.at(-1).context,context);
        host.dispose();assert.equal(context.closeCalls,0);
    }finally{host.dispose();Object.assign(globalThis,original);}
});


test('processor format matches startup and reports zero when unavailable',async()=>{
    const original={AudioContext:globalThis.AudioContext,AudioWorkletNode:globalThis.AudioWorkletNode,crossOriginIsolated:globalThis.crossOriginIsolated};
    Object.assign(globalThis,{AudioContext:Context,AudioWorkletNode:Node,crossOriginIsolated:true});
    const context=new Context();context.sampleRate=44100;
    const host=createAudioHost('unused',{module:{},audioContext:context});
    try {
        const format=host.call({op:'processor.format'});
        assert.deepEqual(format,{sampleRate:44100,channels:2,maxFrames:1024});
        const processor=host.call({op:'processor.start',name:'tone',modules:{tone:[1]},input:{bytes:[0],holds:[]}});
        assert.deepEqual(processor.format,format);
        await settle();
        host.dispose();
        assert.deepEqual(host.call({op:'processor.format'}),{sampleRate:0,channels:0,maxFrames:0});
    }finally{host.dispose();Object.assign(globalThis,original);}
});


test('host retirement revokes imported PCM leases without a worklet acknowledgment',async()=>{
    const original={AudioContext:globalThis.AudioContext,AudioWorkletNode:globalThis.AudioWorkletNode,crossOriginIsolated:globalThis.crossOriginIsolated};
    Object.assign(globalThis,{AudioContext:Context,AudioWorkletNode:Node,crossOriginIsolated:true});
    const host=createAudioHost('unused',{module:{}}),lane=new SharedStore('processor');
    try {
        const owner=host.call({op:'owner.create'}),ring=host.call({op:'ring.create',owner,capacity:64});
        const memory=new WebAssembly.Memory({initial:1}),bridge=createImports(()=>memory,host.call,host.shared);
        const references=()=>bridge.shared_raw(5,ring.owner,ring.id,0,0,0,0);
        const p=host.call({op:'processor.start',name:'tone',modules:{tone:[1]},input:{bytes:[0],holds:[ring,ring]}});
        assert.equal(references(),3); // owner plus both queued input occurrences
        await settle();
        const node=nodes.at(-1),message=node.messages.find(m=>m.kind==='start');
        lane.import(message.shared);lane.retain(ring); // duplicate input occurrence
        assert.equal(references(),3); // lane borrows, never adds an irrevocable hold
        node.port.onmessage({data:{id:p.id,kind:'running'}});
        // No worklet stopped/end response is ever sent.
        node.onprocessorerror();
        assert.equal(host.call({op:'processor.state',id:p.id}).state,'failed');
        assert.equal(references(),1);
        lane.release(ring);lane.dispose();lane.dispose(); // delayed or repeated cleanup
        assert.equal(references(),1);
        host.call({op:'processor.release',id:p.id});
        assert.equal(references(),1);
    }finally{lane.dispose();host.dispose();Object.assign(globalThis,original);}
});
