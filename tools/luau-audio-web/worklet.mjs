import {SharedStore,createImports,utf8Encode,utf8Decode} from './shared.mjs';

function request(exports,value) {
    const input=utf8Encode(JSON.stringify(value));
    const ptr=exports.nuxie_audio_alloc(input.length);
    if(!ptr)throw Error('audio kernel allocation failed');
    try {
        new Uint8Array(exports.memory.buffer,ptr,input.length).set(input);
        const result=exports.nuxie_audio_request(ptr,input.length);
        const length=exports.nuxie_audio_response_len();
        const response=JSON.parse(utf8Decode(new Uint8Array(exports.memory.buffer,result,length)));
        if(!response.ok)throw Error(response.error);
        return response.value;
    }finally{exports.nuxie_audio_free(ptr,input.length);}
}

export class LuauAudioMixer {
    constructor(post,counter) {this.post=post;this.counter=counter;this.lanes=new Map();this.order=[];this.frame=null;}
    relist(){this.order=[...this.lanes.entries()];}
    fail(id,error,initializingExports=null){
        const lane=this.lanes.get(id);
        const exports=lane?.exports??initializingExports;
        this.lanes.delete(id);this.relist();
        // Run Rust destruction while the lane's shared imports still exist.
        // Preserve the original failure even if a trapped VM cannot clean up.
        if(exports){try{request(exports,{op:'stop'});}catch{}}
        lane?.store.dispose();
        this.post({id,kind:'failed',text:String(error?.message??error)});
    }
    receive(message) {
        if(message.kind==='end') {for(const [id] of this.lanes)this.stop(id);return;}
        if(message.kind==='stop'){this.stop(message.id);return;}
        if(message.kind!=='start')return;
        const {id,module,shared,request:start}=message;
        Atomics.store(this.counter,1,1);
        let exports,store;
        try {
            store=new SharedStore('processor');store.import(shared);
            // decode_wire adopts one reference for each descriptor occurrence.
            const seen=new Set();
            for(const h of start.input?.holds??[]){const key=`${h.owner}:${h.id}`;if(seen.has(key))store.retain(h);else seen.add(key);}
            const imports=createImports(()=>exports.memory,r=>store.call(r),store);
            exports=new WebAssembly.Instance(module,{nuxie_audio:imports}).exports;
            request(exports,{...start,op:'start'});
            this.lanes.set(id,{exports,store,format:start.format});this.relist();
            this.post({id,kind:'running'});
            this.deliver(id,exports);
        }catch(error){this.fail(id,error,exports);store?.dispose();}
        finally{Atomics.add(this.counter,0,1);Atomics.store(this.counter,1,0);}
    }
    deliver(id,exports) {
        const status=request(exports,{op:'poll'});
        for(const text of status.logs??[])this.post({id,kind:'log',text});
        if(status.state==='failed'){this.fail(id,status.error??'audio processor failed');return false;}
        return true;
    }
    stop(id) {
        const lane=this.lanes.get(id);
        this.lanes.delete(id);this.relist();
        // Remove first: a failed stop must not recursively attempt destruction.
        if(lane){try{request(lane.exports,{op:'stop'});}catch(error){this.fail(id,error);return;}finally{lane.store.dispose();}}
        this.post({id,kind:'stopped'});
    }
    render(planes,firstFrame) {
        const frames=planes[0]?.length??0;
        if(this.frame===null)this.frame=firstFrame;
        const order=this.order;
        for(const [id,lane] of order) {
            const {exports,format}=lane;
            try {
                for(let offset=0;offset<frames;offset+=format.maxFrames) {
                    const count=Math.min(format.maxFrames,frames-offset),frame=BigInt(this.frame+offset);
                    const ptr=exports.nuxie_audio_render(count,Number(frame&0xffffffffn),Number(frame>>32n));
                    if(!ptr){this.deliver(id,exports);if(this.lanes.has(id))this.fail(id,'audio processor returned no output');break;}
                    const samples=new Float32Array(exports.memory.buffer,ptr,count*format.channels);
                    for(let c=0;c<planes.length;c++)for(let i=0;i<count;i++)planes[c][offset+i]+=samples[i*format.channels+Math.min(c,format.channels-1)];
                    if(!this.deliver(id,exports))break;
                }
            }catch(error){this.fail(id,error);}
        }
        this.frame+=frames;
        Atomics.store(new BigUint64Array(this.counter.buffer,8,1),0,BigInt(this.frame));
        Atomics.add(this.counter,0,1);
    }
}

if(typeof AudioWorkletProcessor!=='undefined') {
    class LuauAudioWorklet extends AudioWorkletProcessor {
        constructor(options){super();this.mixer=new LuauAudioMixer(message=>this.port.postMessage(message),new Int32Array(options.processorOptions.counter));this.port.onmessage=event=>this.mixer.receive(event.data);}
        process(_inputs,outputs){this.mixer.render(outputs[0],globalThis.currentFrame??0);return true;}
    }
    registerProcessor('nuxie-luau-audio',LuauAudioWorklet);
}
