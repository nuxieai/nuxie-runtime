// Qualification only: observe real AudioWorklet output and mute the destination.
class QualificationCapture extends AudioWorkletProcessor {
    constructor() {
        super();this.expectation=null;this.matches=0;this.frames=0;
        this.port.onmessage=({data})=>{this.expectation=data;this.matches=0;};
    }
    process(inputs,outputs) {
        for(const channel of outputs[0]??[])channel.fill(0);
        const input=inputs[0],expect=this.expectation;
        this.frames+=input?.[0]?.length??0;
        if(!expect||input.length!==2||input[0].length!==128||input[1].length!==128)return true;
        let equal=true,peak=0;
        const actual=new Array(256);
        for(let i=0;i<128;i++)for(let c=0;c<2;c++){
            const value=input[c][i];actual[i*2+c]=value;peak=Math.max(peak,Math.abs(value));
            if(!Number.isFinite(value)||value!==expect.samples[i*2+c])equal=false;
        }
        this.matches=equal?this.matches+1:0;
        if(this.matches===3){
            this.port.postMessage({id:expect.id,pass:true,matchedBlocks:this.matches,observedFrames:this.frames,peak,samples:actual});
            this.expectation=null;
        }
        return true;
    }
}
registerProcessor('nuxie-qualification-capture',QualificationCapture);
