import {SharedStore,createImports,utf8Encode,utf8Decode} from './shared.mjs';

export function createAudioHost(kernelURL,options={}) {
    const shared=new SharedStore(),processors=new Map(),stalledContexts=new WeakSet();
    let nextId=1,generation=0,context=options.audioContext??null,output=null,disposed=false,mainExports=null;
    const wake=id=>{if(id!==undefined)mainExports?.nuxie_audio_notify?.(id);options.onEvent?.();};
    // This is the authoritative per-lane lease, including worklet imports.
    // Revoke on host failure without requiring a stopped/end acknowledgment.
    function releaseHolds(p){if(p.holds){for(const h of p.holds)shared.release(h);p.holds=null;}}
    function fail(p,text){if(p.state==='stopped'||p.state==='failed')return;p.state='failed';p.error=String(text);releaseHolds(p);wake(p.id);}
    function ownContext() {
        const session=globalThis.navigator?.audioSession;
        if(session?.type==='auto')session.type='playback';
        const ctx=new AudioContext(),gestures=['pointerdown','pointerup','keydown','touchend'];
        const resume=()=>{ctx.resume().catch(()=>{});};
        const cleanup=()=>{
            if(ctx.state!=='running'&&ctx.state!=='closed')return;
            for(const gesture of gestures)globalThis.removeEventListener?.(gesture,resume,{capture:true});
            ctx.removeEventListener('statechange',cleanup);
        };
        for(const gesture of gestures)globalThis.addEventListener?.(gesture,resume,{capture:true});
        ctx.addEventListener('statechange',cleanup);cleanup();
        return ctx;
    }
    function retire(o,text,{stalled=false,close=true}={}) {
        if(o.ended)return;
        o.ended=true;clearInterval(o.timer);
        o.context.removeEventListener('statechange',o.closed);
        if(stalled)stalledContexts.add(o.context);
        if(o.node){o.node.port.onmessage=null;o.node.onprocessorerror=null;o.node.port.postMessage({kind:'end'});o.node.disconnect();}
        if(output===o){output=null;if(o.owned)context=null;}
        // Retirement precedes notifications, which may start a new generation.
        for(const p of [...processors.values()])if(p.generation===o.generation)fail(p,text);
        // Closing a stalled audio thread can itself wait indefinitely.
        if(close&&!stalled&&o.owned)o.context.close().catch(()=>{});
    }
    async function initialize(o) {
        const [module]=await Promise.all([
            options.module?Promise.resolve(options.module):fetch(kernelURL).then(r=>{if(!r.ok)throw Error(`audio kernel fetch failed: ${r.status}`);return r.arrayBuffer();}).then(bytes=>WebAssembly.compile(bytes)),
            o.context.audioWorklet.addModule(options.workletURL??new URL('./worklet.mjs',import.meta.url)),
        ]);
        if(disposed||o.ended||output!==o)return null;
        const node=new AudioWorkletNode(o.context,'nuxie-luau-audio',{numberOfInputs:0,numberOfOutputs:1,outputChannelCount:[2],processorOptions:{counter:o.counter.buffer}});
        o.node=node;
        node.port.onmessage=({data:m})=>{
            if(o.ended||output!==o)return;
            const p=processors.get(m.id);if(!p||p.generation!==o.generation)return;
            if(m.kind==='log'){
                if(p.logBytes<8192){
                    const bytes=utf8Encode(String(m.text));let end=Math.min(bytes.length,8192-p.logBytes),line;
                    for(;;){try{line=utf8Decode(bytes.subarray(0,end));break;}catch{end--;}}
                    p.logs.push(line);p.logBytes+=end;
                }
                wake(p.id);return;
            }
            if(m.kind==='failed'){fail(p,m.text);return;}
            if(m.kind==='running'&&p.state==='starting')p.state='running';
            if(m.kind==='stopped'&&p.state!=='failed'){p.state='stopped';releaseHolds(p);}
            wake(p.id);
        };
        node.onprocessorerror=()=>retire(o,'audio worklet failed');
        node.connect(options.destination??o.context.destination);
        let seen=-1,stalled=0;
        o.timer=setInterval(()=>{
            if(o.context.state==='closed'){retire(o,'script audio context closed',{close:false});return;}
            const active=[...processors.values()].some(p=>p.generation===o.generation&&(p.state==='starting'||p.state==='running'));
            const now=Atomics.load(o.counter,0);
            stalled=(active||stalled>0)&&o.context.state==='running'&&seen===now?stalled+1:0;seen=now;
            const limit=Atomics.load(o.counter,1)?30:20;
            if(stalled>=limit)retire(o,'processor stalled the audio thread',{stalled:true});
        },50);
        return module;
    }
    function ensure() {
        if(disposed)throw Error('audio host has been disposed');
        if(globalThis.crossOriginIsolated!==true)throw Error('real time audio needs cross origin isolation');
        if(output)return output;
        context=options.audioContext??context??ownContext();
        if(context.state==='closed')throw Error('real time audio cannot start on a closed AudioContext');
        if(stalledContexts.has(context))throw Error("real time audio stalled the host's AudioContext; supply a new AudioContext");
        const o={generation:++generation,context,owned:!options.audioContext,counter:new Int32Array(new SharedArrayBuffer(16)),node:null,timer:null,ended:false};
        output=o;
        o.closed=()=>{if(o.context.state==='closed')retire(o,'script audio context closed',{close:false});};
        o.context.addEventListener('statechange',o.closed);
        o.ready=initialize(o).catch(error=>{retire(o,String(error?.message??error));return null;});
        return o;
    }
    function processorFormat(o, requested={}) {
        const format={channels:2,maxFrames:1024,...requested,sampleRate:o.context.sampleRate};
        if(!Number.isInteger(format.channels)||format.channels<1||format.channels>32||!Number.isInteger(format.maxFrames)||format.maxFrames<1||format.maxFrames>1024)throw Error('invalid audio processor format');
        return format;
    }
    function call(r) {
        if(!r.op.startsWith('processor.'))return shared.call(r);
        const action=r.op.slice(10);
        if(action==='clock') {
            if(disposed)throw Error('audio host has been disposed');
            if(!context)context=options.audioContext??ownContext();
            return {sampleRate:context.sampleRate,frame:output?Atomics.load(new BigUint64Array(output.counter.buffer,8,1),0).toString():'0'};
        }
        if(action==='format') {
            try { return processorFormat(ensure()); }
            catch { return {sampleRate:0,channels:0,maxFrames:0}; }
        }
        if(action==='start') {
            const o=ensure();
            const format=processorFormat(o,r.format);
            const holds=r.input?.holds??[];holds.forEach(h=>shared.get(h));holds.forEach(h=>shared.retain(h));
            const id=nextId++,p={id,generation:o.generation,state:'starting',error:null,logs:[],logBytes:0,holds,owner:r.owner};processors.set(id,p);
            const request={...r,format};delete request.op;
            o.ready.then(module=>{
                if(!module||p.state!=='starting'||disposed||o.ended||output!==o)return;
                o.node.port.postMessage({kind:'start',id,module,shared:shared.export(holds),request});
            }).catch(error=>fail(p,error?.message??error));
            return {id,format};
        }
        const p=processors.get(r.id);if(!p)throw Error('invalid audio processor handle');
        if(action==='state'||action==='poll') {
            const value={state:p.state,error:p.error,logs:action==='poll'?p.logs.splice(0):[]};
            if(action==='poll')p.logBytes=0;
            return value;
        }
        if(action==='stop'||action==='release') {
            if(p.state!=='failed')p.state='stopped';
            // No synchronous reentry to the main VM from its imported call.
            if(output?.generation===p.generation)output.node?.port.postMessage({kind:'stop',id:r.id});
            releaseHolds(p);if(action==='release')processors.delete(r.id);return true;
        }
        throw Error('unknown processor operation');
    }
    return {
        call,shared,imports:memory=>({nuxie_audio:createImports(memory,call,shared)}),attach:exports=>{mainExports=exports;},
        resume:()=>{const o=ensure();return o.context.resume();},
        dispose:()=>{
            if(disposed)return;disposed=true;
            for(const p of processors.values()){p.state='stopped';releaseHolds(p);}processors.clear();
            if(output)retire(output,'audio host disposed');
            else if(!options.audioContext)context?.close().catch(()=>{});
            for(const owner of shared.owners.keys())shared.call({op:'owner.close',owner});
        },
    };
}
