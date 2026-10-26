import test from 'node:test';
import assert from 'node:assert/strict';
import {LuauAudioMixer} from './worklet.mjs';

function kernel(throwOnStop=false) {
    const memory=new WebAssembly.Memory({initial:1});let stops=0,length=0;
    return {
        get stops(){return stops;},
        exports:{memory,nuxie_audio_alloc:()=>128,nuxie_audio_free:()=>{},
            nuxie_audio_request(ptr,size){
                const request=JSON.parse(new TextDecoder().decode(new Uint8Array(memory.buffer,ptr,size)));
                assert.equal(request.op,'stop');stops++;
                if(throwOnStop)throw Error('destructor trapped');
                const bytes=new TextEncoder().encode('{"ok":true,"value":null}');length=bytes.length;
                new Uint8Array(memory.buffer,1024,length).set(bytes);return 1024;
            },nuxie_audio_response_len:()=>length},
    };
}
test('processor failure destroys kernel and preserves original error',()=>{
    for(const throwOnStop of [false,true]){
        const messages=[],mixer=new LuauAudioMixer(m=>messages.push(m),new Int32Array(new SharedArrayBuffer(16))),vm=kernel(throwOnStop);
        mixer.lanes.set(1,{exports:vm.exports});mixer.relist();
        mixer.fail(1,'original budget failure');
        assert.equal(vm.stops,1);assert.equal(mixer.lanes.size,0);
        assert.deepEqual(messages,[{id:1,kind:'failed',text:'original budget failure'}]);
    }
});
test('a trapping stop is not retried and emits only the failure',()=>{
    const messages=[],mixer=new LuauAudioMixer(m=>messages.push(m),new Int32Array(new SharedArrayBuffer(16))),vm=kernel(true);
    mixer.lanes.set(1,{exports:vm.exports});mixer.relist();mixer.stop(1);
    assert.equal(vm.stops,1);assert.equal(mixer.lanes.size,0);
    assert.deepEqual(messages,[{id:1,kind:'failed',text:'destructor trapped'}]);
});
