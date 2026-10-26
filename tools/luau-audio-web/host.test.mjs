import test from 'node:test';
import assert from 'node:assert/strict';
import {createAudioHost} from './host.mjs';

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
