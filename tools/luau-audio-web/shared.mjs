// Explicit shared storage only: a processor's Luau VM heap is never shared.
export const PROCESSOR_HOLDS_ERROR = "a processor cannot pass shared objects through rings or snapshots; pass them in Audio.start's argument";
const MAX_BYTES = 256 << 20;
const HEADER = 128;
const CLOSED = 0, LOCK = 1, HEAD = 2, USED = 3, PUBLISHED = 4, POISON = 5;
const VERSION = [8, 16], SIZE = [9, 17], HAS_HOLDS = [10, 18];
const SEQUENCE = [6, 10]; // indices into the u64 header
const key = h => `${h.owner}:${h.id}`;
const handle = o => ({ owner: o.owner, id: o.id, kind: o.kind });
const integer = (n, label) => {
    if (!Number.isSafeInteger(n) || n < 0) throw Error(`invalid ${label}`);
    return n;
};
const bytes = a => {
    if (!Array.isArray(a) || a.some(n => !Number.isInteger(n) || n < 0 || n > 255)) throw Error('invalid shared bytes');
    return Uint8Array.from(a);
};
function locked(o, action) {
    while (Atomics.compareExchange(o.words, LOCK, 0, 1) !== 0) {}
    try { return action(); } finally { Atomics.store(o.words, LOCK, 0); }
}
function object(descriptor) {
    return { ...descriptor, words: new Int32Array(descriptor.buffer, 0, HEADER / 4),
        sequences: new BigUint64Array(descriptor.buffer, 0, HEADER / 8),
        data: new Uint8Array(descriptor.buffer, HEADER), holds: new Map(), refs: 1, locked: false };
}

export class SharedStore {
    constructor(lane = 'main') { this.lane = lane; this.owners = new Map(); this.objects = new Map(); this.nextOwner = 1; this.nextId = 1; }
    import(descriptors) {
        for (const d of descriptors) {
            if (!(d.buffer instanceof SharedArrayBuffer)) throw Error('shared storage must be a SharedArrayBuffer');
            if (!this.objects.has(key(d))) this.objects.set(key(d), object(d));
        }
    }
    export(handles) {
        return [...new Map(handles.map(h => { const o = this.get(h); return [key(h), { ...handle(o), capacity: o.capacity, buffer: o.buffer }]; })).values()];
    }
    get(h, kind) {
        const o = this.objects.get(key(h));
        if (!o || o.kind !== h.kind || (kind && kind !== o.kind)) throw Error('invalid shared handle');
        return o;
    }
    retain(h) { this.get(h).refs++; }
    release(h) {
        const o = this.get(h);
        if (--o.refs !== 0) return;
        this.objects.delete(key(h));
        for (const held of o.holds.values()) for (const h of held) this.release(h);
        const budget = this.owners.get(o.owner);
        if (budget) {
            budget.bytes -= o.charge; budget.objects -= o.count;
            if (budget.closed && budget.objects===0) this.owners.delete(o.owner);
        }
    }
    create(kind, request) {
        if (this.lane !== 'main') throw Error('processors cannot create shared objects');
        const budget = this.owners.get(request.owner);
        if (!budget || budget.closed) throw Error('shared owner has been released');
        const initial = kind === 'blob' ? bytes(request.bytes) : null;
        const capacity = integer(initial?.length ?? request.capacity ?? request.length, 'shared capacity');
        const charge = kind === 'snapshot' ? capacity * 2 : capacity;
        const count = kind === 'ring' ? 3 : 1;
        if (capacity > MAX_BYTES || (kind === 'snapshot' && charge > MAX_BYTES) || ((kind === 'ring' || kind === 'snapshot' || kind === 'region') && capacity === 0)) throw Error('invalid shared capacity');
        if (charge > budget.maximum - budget.bytes) throw Error('shared memory budget exceeded');
        if (count > 65536 - budget.objects) throw Error('shared object limit exceeded');
        // Region atomics need a complete final word, but size remains exact.
        const storage = kind === 'region' ? Math.ceil(charge / 4) * 4 : charge;
        const o = object({owner: request.owner, id: this.nextId++, kind, capacity, buffer: new SharedArrayBuffer(HEADER + storage)});
        o.charge = charge; o.count = count;
        if (initial) o.data.set(initial);
        this.objects.set(key(o), o); budget.bytes += charge; budget.objects += count;
        return handle(o);
    }
    encoded(value) {
        const payload = bytes(value.bytes), holds = value.holds ?? [];
        holds.forEach(h => this.get(h));
        if (this.lane === 'processor' && holds.length) throw Error(PROCESSOR_HOLDS_ERROR);
        return {payload, holds, size: payload.length + (holds.length ? 4 + holds.length * 4 : 0)};
    }
    call(r) {
        const [kind, action] = r.op.split('.');
        if (r.op === 'owner.create') {
            if (this.lane !== 'main') throw Error('processors cannot create owners');
            const owner = this.nextOwner++;
            this.owners.set(owner, { maximum: integer(r.maximumBytes ?? (512 << 20), 'shared budget'), bytes: 0, objects: 0, closed: false });
            return owner;
        }
        if (r.op === 'owner.close') {
            const budget = this.owners.get(r.owner); if (budget) budget.closed = true;
            const retired = [];
            for (const o of this.objects.values()) if (o.owner === r.owner) locked(o, () => {
                Atomics.store(o.words, CLOSED, 1); Atomics.store(o.words, USED, 0); Atomics.store(o.words, PUBLISHED, 0);
                retired.push(...o.holds.values()); o.holds.clear();
            });
            for (const holds of retired) for (const h of holds) this.release(h);
            if (budget && budget.objects===0) this.owners.delete(r.owner);
            return true;
        }
        if (action === 'create') return this.create(kind, r);
        if (r.op === 'shared.retain') { this.retain(r.handle); return true; }
        if (r.op === 'shared.release') { this.release(r.handle); return true; }
        const o = this.get(r.handle, r.op==='size'||r.op==='capacity'?undefined:kind);
        if (action === 'size' || action === 'capacity' || r.op==='size'||r.op==='capacity') {
            if ((o.kind==='blob'||o.kind==='region') && Atomics.load(o.words,CLOSED)) return 0;
            return o.capacity;
        }
        if (kind === 'ring') return this.ring(o, action, r);
        if (kind === 'snapshot') return this.snapshot(o, action, r);
        if (kind === 'blob') {
            if (action !== 'read') throw Error('unknown blob operation');
            const offset = integer(r.offset, 'blob offset'), length = integer(r.length, 'blob length');
            return Atomics.load(o.words, CLOSED) ? [] : Array.from(o.data.slice(Math.min(offset, o.capacity), Math.min(offset + length, o.capacity)));
        }
        if (kind === 'region') return this.region(o, action, r);
        throw Error('unknown shared operation');
    }
    ring(o, action, r) {
        if (action === 'used') return Atomics.load(o.words, USED);
        const encoded = action === 'send' ? this.encoded(r.encoded) : null;
        return locked(o, () => {
            const head = Atomics.load(o.words, HEAD), used = Atomics.load(o.words, USED);
            const read = (at, length) => Uint8Array.from({length}, (_, i) => o.data[(at + i) % o.capacity]);
            const word = at => new DataView(read(at, 4).buffer).getUint32(0, true);
            const write = (at, value) => { for (let i=0;i<value.length;i++) o.data[(at+i)%o.capacity] = value[i]; };
            const putWord = (at, n) => { const b=new Uint8Array(4); new DataView(b.buffer).setUint32(0,n,true); write(at,b); };
            if (action === 'send') {
                if (Atomics.load(o.words,CLOSED) || encoded.size+4 > o.capacity-used) return false;
                const tail=(head+used)%o.capacity;
                putWord(tail,encoded.size | (encoded.holds.length ? 0x80000000 : 0)); write(tail+4,encoded.payload);
                if(encoded.holds.length) {
                    encoded.holds.forEach((h,i)=>{this.retain(h);putWord(tail+4+encoded.payload.length+i*4,i);});
                    putWord(tail+4+encoded.payload.length+encoded.holds.length*4,encoded.holds.length); o.holds.set(tail,encoded.holds);
                }
                Atomics.store(o.words,USED,used+encoded.size+4); return true;
            }
            if(action !== 'receive') throw Error('unknown ring operation');
            if(!used) return null;
            const header=word(head), size=header&0x7fffffff, hasHolds=(header&0x80000000)!==0;
            if(hasHolds && (this.lane==='processor'||r.processor)) throw Error(PROCESSOR_HOLDS_ERROR);
            const holds=o.holds.get(head)??[], count=hasHolds?word(head+size):0;
            const payload=read(head+4,size-(hasHolds?4+count*4:0));
            o.holds.delete(head); Atomics.store(o.words,HEAD,(head+4+size)%o.capacity); Atomics.store(o.words,USED,used-4-size);
            // Queue-owned hold references transfer to the decoder.
            return {bytes:Array.from(payload),holds};
        });
    }
    snapshot(o, action, r) {
        if(action==='publish') {
            const e=this.encoded(r.encoded); if(e.size>o.capacity)return false;
            let retired=[];
            const result=locked(o,()=>{
                if(Atomics.load(o.words,CLOSED))return false;
                const previous=Atomics.load(o.words,PUBLISHED)-1;
                if((this.lane==='processor'||r.processor) && previous>=0 && Atomics.load(o.words,HAS_HOLDS[previous]))throw Error(PROCESSOR_HOLDS_ERROR);
                const index=previous===0?1:0;
                Atomics.add(o.words,VERSION[index],1);
                for(let i=0;i<e.payload.length;i++)Atomics.store(o.data,index*o.capacity+i,e.payload[i]);
                Atomics.store(o.words,SIZE[index],e.payload.length); Atomics.store(o.words,HAS_HOLDS[index],e.holds.length?1:0);
                const seq=previous<0?1n:Atomics.load(o.sequences,SEQUENCE[previous])+1n;
                Atomics.store(o.sequences,SEQUENCE[index],seq);
                e.holds.forEach(h=>this.retain(h));
                retired=[...(o.holds.get(index)??[]),...(previous>=0?(o.holds.get(previous)??[]):[])];
                o.holds.clear();o.holds.set(index,e.holds);
                Atomics.add(o.words,VERSION[index],1);Atomics.store(o.words,PUBLISHED,index+1);return true;
            });
            retired.forEach(h=>this.release(h));return result;
        }
        if(action!=='read' && action!=='seq')throw Error('unknown snapshot operation');
        for(;;) {
            const index=Atomics.load(o.words,PUBLISHED)-1;if(index<0)return action==='seq'?'0':null;
            const version=Atomics.load(o.words,VERSION[index]);if(version&1)continue;
            const sequence=Atomics.load(o.sequences,SEQUENCE[index]);
            const size=Atomics.load(o.words,SIZE[index]), hasHolds=Atomics.load(o.words,HAS_HOLDS[index]);
            if(action==='seq') {if(Atomics.load(o.words,VERSION[index])===version)return sequence.toString();continue;}
            if(hasHolds) {
                if(Atomics.load(o.words,VERSION[index])!==version)continue;
                if(this.lane==='processor'||r.processor)throw Error(PROCESSOR_HOLDS_ERROR);
                return locked(o,()=>{
                    const current=Atomics.load(o.words,PUBLISHED)-1;if(current<0)return null;
                    const holds=o.holds.get(current)??[];holds.forEach(h=>this.retain(h));
                    return {sequence:Atomics.load(o.sequences,SEQUENCE[current]).toString(),encoded:{bytes:Array.from(o.data.slice(current*o.capacity,current*o.capacity+Atomics.load(o.words,SIZE[current]))),holds}};
                });
            }
            const payload=Array.from({length:size},(_,i)=>Atomics.load(o.data,index*o.capacity+i));
            if(Atomics.load(o.words,VERSION[index])===version)return {sequence:sequence.toString(),encoded:{bytes:payload,holds:[]}};
        }
    }
    region(o, action, r) {
        if(action==='poisoned')return !Atomics.load(o.words,CLOSED) && !!Atomics.load(o.words,POISON);
        if(action==='unlock') {
            if(!o.locked)throw Error('region is not locked by this lane');
            if(r.poison)Atomics.store(o.words,POISON,1);
            o.locked=false;Atomics.store(o.words,LOCK,0);return true;
        }
        if(Atomics.load(o.words,CLOSED))throw Error('shared owner has been released');
        if(action==='lock') {
            if(this.lane!=='main')throw Error('processors cannot lock regions');
            if(o.locked)throw Error('shared region is already locked by this lane');
            const timeout=r.timeoutMs??0;
            if(!Number.isFinite(timeout)||timeout<0)throw Error('invalid region timeout');
            const deadline=Date.now()+timeout;
            do {if(Atomics.compareExchange(o.words,LOCK,0,1)===0){o.locked=true;return true;}}while(Date.now()<deadline);
            return false;
        }
        const offset=integer(r.offset,'region offset');
        if(action==='read'||action==='write') {
            if(this.lane!=='main'||!o.locked)throw Error('region byte access requires exclusive main-lane lock');
            const data=action==='write'?bytes(r.bytes):null, length=data?.length??integer(r.length,'region length');
            if(offset>o.capacity||length>o.capacity-offset)throw Error('shared region access out of bounds');
            if(data){o.data.set(data,offset);return true;}return Array.from(o.data.slice(offset,offset+length));
        }
        if(offset%4||offset>o.capacity||4>o.capacity-offset)throw Error('shared region atomic access out of bounds or unaligned');
        const words=new Uint32Array(o.buffer,HEADER),at=offset/4;
        if(action==='load')return Atomics.load(words,at);
        if(action==='store'){Atomics.store(words,at,r.value>>>0);return 0;}
        if(action==='add')return Atomics.add(words,at,r.value>>>0);
        if(action==='cas')return Atomics.compareExchange(words,at,r.expected>>>0,r.desired>>>0);
        throw Error('unknown region operation');
    }
}

// Two-step response draining never reruns a mutating operation.
export function createImports(memory, dispatch) {
    let pending=null;
    return {host_call(op,ptr,length,out,capacity){
        if(op===1) {
            try {pending=utf8Encode(JSON.stringify({ok:true,value:dispatch(JSON.parse(utf8Decode(new Uint8Array(memory().buffer,ptr>>>0,length>>>0))))}));}
            catch(error){pending=utf8Encode(JSON.stringify({ok:false,error:String(error?.message??error)}));}
        }else if(op!==2||pending===null)return -1;
        if(capacity>>>0<pending.length)return -pending.length;
        new Uint8Array(memory().buffer,out>>>0,pending.length).set(pending);const written=pending.length;pending=null;return written;
    },now_ms(){return globalThis.performance !== undefined ? globalThis.performance.now() : Date.now();}};
}

// AudioWorkletGlobalScope need not supply TextEncoder/TextDecoder.
export function utf8Encode(text) {
    const out=[];
    for(const character of text) {
        let c=character.codePointAt(0);if(c>=0xd800&&c<=0xdfff)c=0xfffd;
        if(c<0x80)out.push(c);
        else if(c<0x800)out.push(0xc0|(c>>6),0x80|(c&63));
        else if(c<0x10000)out.push(0xe0|(c>>12),0x80|((c>>6)&63),0x80|(c&63));
        else out.push(0xf0|(c>>18),0x80|((c>>12)&63),0x80|((c>>6)&63),0x80|(c&63));
    }
    return Uint8Array.from(out);
}
export function utf8Decode(data) {
    let text='';
    for(let i=0;i<data.length;) {
        const first=data[i++];let n,c,min;
        if(first<128){text+=String.fromCharCode(first);continue;}
        if(first>=0xc2&&first<=0xdf){n=1;c=first&31;min=0x80;}
        else if(first>=0xe0&&first<=0xef){n=2;c=first&15;min=0x800;}
        else if(first>=0xf0&&first<=0xf4){n=3;c=first&7;min=0x10000;}
        else throw Error('invalid UTF-8');
        if(i+n>data.length)throw Error('truncated UTF-8');
        while(n--){const next=data[i++];if((next&0xc0)!==0x80)throw Error('invalid UTF-8');c=(c<<6)|(next&63);}
        if(c<min||c>0x10ffff||(c>=0xd800&&c<=0xdfff))throw Error('invalid UTF-8');
        text+=String.fromCodePoint(c);
    }
    return text;
}
