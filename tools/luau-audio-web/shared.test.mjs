import test from 'node:test';
import assert from 'node:assert/strict';
import { SharedStore, createImports, PROCESSOR_HOLDS_ERROR } from './shared.mjs';

function setup() {
    const main = new SharedStore();
    const owner = main.call({op:'owner.create'});
    return {main, owner, create:(kind, capacity)=>main.call({op:`${kind}.create`,owner,capacity})};
}
test('region requires positive capacity while an empty blob is valid',()=>{
    const {main,owner,create}=setup();
    assert.throws(()=>create('region',0),{message:'invalid shared capacity'});
    const blob=main.call({op:'blob.create',owner,bytes:[]});
    assert.equal(main.call({op:'size',handle:blob}),0);
});
test('ring byte framing wraps and is visible to an independent lane',()=>{
    const {main,create}=setup(),handle=create('ring',13),audio=new SharedStore('processor');
    audio.import(main.export([handle]));
    const send=bytes=>main.call({op:'ring.send',handle,encoded:{bytes,holds:[]}});
    const receive=()=>audio.call({op:'ring.receive',handle});
    assert.equal(send([1,2,3,4]),true);
    assert.equal(send([5,6]),false);
    assert.deepEqual(receive(),{bytes:[1,2,3,4],holds:[]});
    assert.equal(send([7,8,9,10,11,12,13,14,15]),true);
    assert.deepEqual(receive(),{bytes:[7,8,9,10,11,12,13,14,15],holds:[]});
    assert.equal(receive(),null);
});
test('processor cannot consume held handles or change the queue',()=>{
    const {main,create}=setup(),handle=create('ring',64),held=create('snapshot',16),audio=new SharedStore('processor');
    audio.import(main.export([handle,held]));
    assert.equal(main.call({op:'ring.send',handle,encoded:{bytes:[6,0,0,0,0],holds:[held]}}),true);
    const used=main.call({op:'ring.used',handle});
    assert.throws(()=>audio.call({op:'ring.receive',handle}),{message:PROCESSOR_HOLDS_ERROR});
    assert.equal(main.call({op:'ring.used',handle}),used);
    assert.deepEqual(main.call({op:'ring.receive',handle}),{bytes:[6,0,0,0,0],holds:[held]});
    main.call({op:'shared.release',handle:held}); // consumed ownership
});
test('snapshot oversize does not replace coherent value or sequence',()=>{
    const {main,create}=setup(),handle=create('snapshot',4),audio=new SharedStore('processor');
    audio.import(main.export([handle]));
    assert.equal(audio.call({op:'snapshot.publish',handle,encoded:{bytes:[1,2,3,4],holds:[]}}),true);
    assert.equal(audio.call({op:'snapshot.publish',handle,encoded:{bytes:[5,6,7,8,9],holds:[]}}),false);
    assert.deepEqual(main.call({op:'snapshot.read',handle}),{sequence:'1',encoded:{bytes:[1,2,3,4],holds:[]}});
    assert.equal(main.call({op:'snapshot.seq',handle}),'1');
});
test('snapshot with handles rejects processor reads and publication',()=>{
    const {main,create}=setup(),handle=create('snapshot',64),held=create('ring',8),audio=new SharedStore('processor');
    audio.import(main.export([handle,held]));
    main.call({op:'snapshot.publish',handle,encoded:{bytes:[6,0,0,0,0],holds:[held]}});
    assert.throws(()=>audio.call({op:'snapshot.read',handle}),{message:PROCESSOR_HOLDS_ERROR});
    assert.throws(()=>audio.call({op:'snapshot.publish',handle,encoded:{bytes:[0],holds:[]}}),{message:PROCESSOR_HOLDS_ERROR});
});
test('blob read clamps; region lock, atomics and poisoning cross lanes',()=>{
    const {main,owner,create}=setup(),blob=main.call({op:'blob.create',owner,bytes:[10,20,30]}),region=create('region',9),audio=new SharedStore('processor');
    audio.import(main.export([blob,region]));
    assert.deepEqual(audio.call({op:'blob.read',handle:blob,offset:1,length:99}),[20,30]);
    assert.deepEqual(audio.call({op:'blob.read',handle:blob,offset:9,length:1}),[]);
    assert.equal(main.call({op:'region.lock',handle:region,timeoutMs:0}),true);
    main.call({op:'region.write',handle:region,offset:0,bytes:[255,255,255,255]});
    main.call({op:'region.unlock',handle:region});
    assert.equal(audio.call({op:'region.add',handle:region,offset:0,value:1}),0xffffffff);
    assert.equal(main.call({op:'region.load',handle:region,offset:0}),0);
    assert.throws(()=>audio.call({op:'region.lock',handle:region,timeoutMs:0}));
    assert.throws(()=>audio.call({op:'region.load',handle:region,offset:8}));
    assert.throws(()=>main.call({op:'region.read',handle:region,offset:0,length:1}));
    main.call({op:'region.lock',handle:region,timeoutMs:0});
    main.call({op:'region.unlock',handle:region,poison:true});
    assert.equal(audio.call({op:'region.poisoned',handle:region}),true);
    assert.equal(audio.call({op:'region.load',handle:region,offset:0}),0);
    assert.equal(main.call({op:'region.lock',handle:region,timeoutMs:0}),true);
    main.call({op:'region.unlock',handle:region});
});
test('close retires shared storage and releases cycles',()=>{
    const {main,owner,create}=setup(),handle=create('ring',64),audio=new SharedStore('processor');
    audio.import(main.export([handle]));
    main.call({op:'ring.send',handle,encoded:{bytes:[6,0,0,0,0],holds:[handle]}});
    main.call({op:'shared.release',handle});
    main.call({op:'owner.close',owner});
    assert.equal(main.objects.size,0);
    assert.equal(audio.call({op:'ring.receive',handle}),null);
});
test('retired blob and region observables agree for retained and imported handles',()=>{
    const {main,owner,create}=setup();
    const blob=main.call({op:'blob.create',owner,bytes:[1,2,3]}),region=create('region',8);
    const ring=create('ring',16),snapshot=create('snapshot',16),audio=new SharedStore('processor');
    audio.import(main.export([blob,region,ring,snapshot]));
    main.call({op:'region.lock',handle:region,timeoutMs:0});
    main.call({op:'region.unlock',handle:region,poison:true});
    for(const lane of [main,audio]) {
        assert.equal(lane.call({op:'size',handle:blob}),3);
        assert.equal(lane.call({op:'size',handle:region}),8);
        assert.equal(lane.call({op:'region.poisoned',handle:region}),true);
    }
    main.call({op:'owner.close',owner});
    for(const lane of [main,audio]) {
        assert.equal(lane.call({op:'size',handle:blob}),0);
        assert.equal(lane.call({op:'size',handle:region}),0);
        assert.equal(lane.call({op:'region.poisoned',handle:region}),false);
        assert.deepEqual(lane.call({op:'blob.read',handle:blob,offset:0,length:3}),[]);
        assert.equal(lane.call({op:'capacity',handle:ring}),16);
        assert.equal(lane.call({op:'capacity',handle:snapshot}),16);
    }
});
test('closed owner metadata retires after its last retained object releases',()=>{
    const main=new SharedStore(),owners=main.owners.size,objects=main.objects.size;
    for(let i=0;i<100;i++) {
        const owner=main.call({op:'owner.create'});
        const handle=main.call({op:'region.create',owner,capacity:4});
        main.call({op:'shared.retain',handle});
        main.call({op:'owner.close',owner});
        assert.equal(main.owners.size,owners+1);
        assert.equal(main.objects.size,objects+1);
        assert.equal(main.call({op:'size',handle}),0);
        main.call({op:'shared.release',handle});
        assert.equal(main.owners.size,owners+1);
        main.call({op:'shared.release',handle});
        assert.equal(main.owners.size,owners);
        assert.equal(main.objects.size,objects);
        // Repeated close is harmless, including an owner with no allocations.
        main.call({op:'owner.close',owner});
        const empty=main.call({op:'owner.create'});
        main.call({op:'owner.close',owner:empty});
        assert.equal(main.owners.size,owners);
    }
});
test('response drain never executes a mutation twice',()=>{
    const memory=new WebAssembly.Memory({initial:1});let mutations=0;
    const imports=createImports(()=>memory,()=>{mutations++;return 'a long response';});
    const request=new TextEncoder().encode('{"op":"test"}');new Uint8Array(memory.buffer,0,request.length).set(request);
    const size=-imports.host_call(1,0,request.length,100,1);
    assert.ok(size>1);assert.equal(mutations,1);
    assert.equal(imports.host_call(2,0,0,100,size),size);assert.equal(mutations,1);
    assert.deepEqual(JSON.parse(new TextDecoder().decode(new Uint8Array(memory.buffer,100,size))),{ok:true,value:'a long response'});
});

test('binary PCM bridge preserves framing, atomics, pending holds and lane teardown',()=>{
    const {main,create}=setup(),ring=create('ring',64),seeks=create('region',8),audio=new SharedStore('processor');
    const memory=new WebAssembly.Memory({initial:1}),data=new Uint8Array(memory.buffer),view=new DataView(memory.buffer);
    const bridge=createImports(()=>memory,()=>{throw Error('PCM must not use JSON');},main);
    view.setUint32(0,7,true);view.setUint32(4,2,true);view.setFloat64(8,100,true);view.setFloat32(16,0.25,true);
    assert.equal(bridge.shared_raw(2,ring.owner,ring.id,0,20,0,0),1);
    main.retain(ring); // queued processor initialization, before worklet import
    assert.equal(bridge.shared_raw(5,ring.owner,ring.id,0,0,0,0),2);
    audio.import(main.export([ring,seeks]));
    const reader=createImports(()=>memory,()=>{throw Error('PCM must not use JSON');},audio);
    assert.equal(reader.shared_raw(1,ring.owner,ring.id,32,16,0,0),21);
    assert.equal(reader.shared_raw(1,ring.owner,ring.id,32,20,0,0),21);
    assert.deepEqual(data.slice(32,52),data.slice(0,20));
    assert.equal(reader.shared_raw(1,ring.owner,ring.id,32,20,0,0),0);
    assert.equal(bridge.shared_raw(4,seeks.owner,seeks.id,0,0,0,0xffffffff),0);
    assert.equal(reader.shared_raw(3,seeks.owner,seeks.id,0,0,0,0),0xffffffff);
    assert.equal(reader.shared_raw(4,seeks.owner,seeks.id,0,0,0,1),0xffffffff);
    assert.equal(bridge.shared_raw(3,seeks.owner,seeks.id,0,0,0,0),0);
    audio.dispose();main.release(ring);
    assert.equal(bridge.shared_raw(5,ring.owner,ring.id,0,0,0,0),1);
});
test('PCM callback returns immediately when the producer holds the ring lock',()=>{
    const {main,create}=setup(),ring=create('ring',32),memory=new WebAssembly.Memory({initial:1});
    const bridge=createImports(()=>memory,()=>{},main);
    const descriptor=main.export([ring])[0],words=new Int32Array(descriptor.buffer,0,32);
    Atomics.store(words,1,1);
    assert.equal(bridge.shared_raw(1,ring.owner,ring.id,0,32,0,0),0);
    Atomics.store(words,1,0);
});
test('decode reservations share the object budget and refund exactly once',()=>{
    const main=new SharedStore(),owner=main.call({op:'owner.create',maximumBytes:16});
    main.call({op:'owner.reserve',owner,bytes:12});
    assert.throws(()=>main.call({op:'blob.create',owner,bytes:[1,2,3,4,5]}),/budget/);
    main.call({op:'owner.refund',owner,bytes:12});
    assert.throws(()=>main.call({op:'owner.refund',owner,bytes:12}),/refund/);
    main.call({op:'blob.create',owner,bytes:[1,2,3,4,5]});
});
