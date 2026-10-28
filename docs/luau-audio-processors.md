# Luau-authored audio processors

Authors define audio processing in Luau. Each processor gets an isolated Luau VM:
its module and `init` run on an initialization thread, then `process` runs directly
on the audio callback. The runtime mixes its interleaved output with ordinary
sounds. Wasm is not involved. Existing `Audio.play` asset playback is unchanged.

A processor is a named, precompiled Luau module returning an object with `init`
and `process` methods. Successfully registered ScriptAsset modules are available
by name; hosts can also call `ScriptVm::register_audio_processor_module(name,
bytecode)` explicitly. Production playback does not need a source compiler.
`require` resolves the registered module bytes inside the isolated processor VM,
not the main VM's already-executed tables or closures.

For example, compile/register this module as `tone`:

```lua
return {
    init = function(self, sampleRate, maxFrames, channels, input)
        self.rate = sampleRate
        self.channels = channels
        self.phase = 0
        self.frequency = input.frequency
        self.gain = input.gain
        self.controls = input.controls
        self.meter = input.meter
        self.nextControl = {}
        self.measurement = {peak = 0, frame = 0}
    end,

    process = function(self, output, frames, startFrame)
        while self.controls:receiveInto(self.nextControl) do
            self.frequency = self.nextControl.frequency
            self.gain = self.nextControl.gain
        end
        local peak = 0
        for frame = 0, frames - 1 do
            local sample = math.sin(self.phase) * self.gain
            self.phase = (self.phase + 2 * math.pi * self.frequency / self.rate)
                % (2 * math.pi)
            peak = math.max(peak, math.abs(sample))
            for channel = 0, self.channels - 1 do
                buffer.writef32(output, (frame * self.channels + channel) * 4, sample)
            end
        end
        self.measurement.peak = peak
        self.measurement.frame = startFrame
        self.meter:publish(self.measurement)
    end,
}
```

The ordinary instance script starts and controls it:

```lua
return function(context)
    local tone, controls, meter
    local elapsed = 0
    local changed = false
    local stopped = false
    return {
        init = function(self)
            controls = Audio.Ring.create(1024)
            meter = Audio.Snapshot.create(1024)
            tone = Audio.start("tone", {
                frequency = 440, gain = 0.02,
                controls = controls, meter = meter,
            })
            return true
        end,
        advance = function(self, seconds)
            elapsed += seconds
            if tone.failed ~= nil then
                print(tone.failed)
                return true
            end
            if elapsed >= 0.5 and not changed then
                -- A full ring returns false; retry on the next advance.
                changed = controls:send({frequency = 880, gain = 0.01})
            end
            local measurement = meter:latest()
            if measurement ~= nil then
                -- measurement.value contains the author's own record.
                local peak = measurement.value.peak
            end
            if elapsed >= 1 and not stopped then
                tone:stop()
                stopped = true
            end
            return true
        end,
    }
end
```

`Audio.start` starts initialization immediately and returns a handle whose `state`
is `starting`, `running`, `stopped`, or `failed`. `failed` is nil or failure text.
Processor `print` output is buffered and delivered through the main VM host logger
through ordinary runtime async polling, including idle frames, and script callback
boundaries—never directly from the audio thread. Initialization and processing
failures are reported once through that logger without requiring authors to read
`failed`; the handle still retains the failure text for inspection.
`stop()` is terminal and idempotent; it leaves the mix after any active block has
finished. Instance teardown, failed instance initialization and VM teardown stop
owned processors even if another Lua object retains their handles. Start inside
an instance's `init` or another owned callback; tools playback gating still applies.

The processor receives the engine sample rate, maximum frames per block and
channel count. `output` is a Luau buffer of interleaved float32 samples; byte offsets
are zero-based. Write only the current `frames * channels` samples. `startFrame`
is the absolute engine frame clock. Parameter interpolation, scheduling within a
block, envelopes and waveform choice are ordinary authored Luau code, rather than
hardcoded oscillator parameters in the host API.

Shared rings and snapshots can be included anywhere in the initial plain-data
record, so a processor may have multiple independent control and feedback channels.
Capacities are bytes, not message counts. The Luau constructors require an explicit
capacity because AssemblyScript's typed constructor default is not present in the
public runtime source. Shared storage follows upstream's 512 MiB aggregate and
65,536-object defaults per VM; a ring is limited to 256 MiB and a snapshot reserves two copies of at most 128 MiB
each. Zero-capacity snapshots are valid; rings require nonzero capacity. A ring preserves message order and reports
full capacity from `send` as false. `receiveInto(table)` fills a caller-provided
record/array and returns false when empty; `receive()` returns the next value or nil.
A snapshot retains the latest published value; `latest()` returns nil before any
publication, then `{value = ..., sequence = ...}`. Repeated reads return the latest
publication, not only changes since the previous read.

Luau data is copied between VMs. Supported data consists of nil, booleans, numbers,
byte strings, dense arrays and string-keyed records. Shared Ring/Snapshot/Blob/Region handles
are allowed in initialization input and main-script shared records. As upstream,
a processor cannot send/publish shared handles or consume records containing them;
a rejected ring record stays queued. Pass processor handles in initialization input. Cycles,
functions and arbitrary host userdata cannot be copied. The dynamic Luau encoding
is a language adaptation of upstream's generated typed schemas, not byte-compatible
with AssemblyScript shared records. Its nesting check protects the host traversal
stack; channel capacities bound encoded messages.

Execution policy belongs to the host, not the author. The default `hardBlocks`
policy is two blocks; hosts configure it with `ScriptVm::set_audio_processor_hard_blocks`.
Zero disables the block-time budget for intentional host-controlled use such as
offline rendering. Initialization uses upstream's separate time budget. Processor
memory inherits the host ScriptVm memory limit. A failed processor emits silence
and reports its failure without stopping other processors. Time budgets use VM
interrupt points and elapsed-time checks; they are not an operating-system hard
realtime guarantee, and native library calls or GC may exceed a deadline before
control returns. These limitations must be measured rather than hidden.

This replaces the previous `Audio.createProcessor("oscillator", ...)` interface.
The oscillator is now Luau code, not a fixed selection in the runtime. Processor
execution supports native callbacks and browser AudioWorklets, using Luau in both.
Full editor export integration and SDK releases remain outside this qualification.

## Blob and Region

`Audio.Blob.from(bufferOrString)` copies immutable bytes. Read them using
`blob.size`, `blob:read(offset, outputBuffer)` (returns bytes copied), or
`blob:toBuffer()`. Empty blobs are valid; partial reads preserve the output suffix.

`Audio.Region.create(byteLength)` creates zeroed shared storage. `size` and
`poisoned` are readable fields. Aligned in-bounds U32 operations are
`atomicLoad(offset)`, `atomicStore(offset, value)`, `atomicAdd(offset, value)`,
and `atomicCas(offset, expected, desired)`. Store returns zero; add and CAS return
the previous value. Arithmetic wraps; atomics are sequentially consistent.

Main scripts may call `region:tryExclusive(callback, timeoutSeconds)` with an
explicit timeout. The callback receives a scoped view with `read`, `write`,
`getU32`, and `setU32`; it expires on callback return. Errors poison and unlock
the region, but later locking and atomic access remain allowed. Processor scripts
may read blobs and use Region atomics, but cannot create shared objects or lock
regions. Pass shared objects through `Audio.start` initialization input.

## Offline rendering

The Rust host configures process-wide offline mode using
`ScriptVm::set_audio_render_offline(enabled, AudioProcessorFormat { sample_rate,
channels, max_frames })`. Zero fields clamp to one; either toggle resets the clock.
New processors capture this format and use no realtime block deadline or device
attachment. Existing processors retain their start format.

Obtain owned lanes with `vm.audio_processors()`. Once a lane is running, call
`handle.render_offline(&mut interleaved_samples, frames, start_frame)` for each
lane, then publish the completed frame using `ScriptVm::set_audio_offline_frame`.
The output length must equal frames times channels. Starting, stopped, and failed
lanes return silence. `Audio.sampleRate`, `Audio.timeFrame`, and `Audio.time`
reflect the configured offline clock. The host owns mixing and clock advancement.

## Browser hosting

The browser executes the same Rust Luau interpreter in private
`wasm32-unknown-unknown` instances on an AudioWorklet. This does not add the
upstream Wasm guest scripting engine. Only explicitly shared audio objects use
SharedArrayBuffer; VM heaps stay private. Initialization runs in the worklet
message handler, and sample generation runs directly in its audio callback.

The embedding host supplies audio imports and forwards asynchronous notifications.
It requires a secure context, cross-origin isolation, and user-activated audio
resume. See [browser hosting and qualification](../tools/luau-audio-web/README.md)
for the integration contract and tested coverage.

## Decoded audio samples and streaming

Audio assets expose `sampleRate`, `channels`, and `duration`. Decode to the
processor's selected format, which may differ from the source:

```lua
local rate = Audio.processorSampleRate()
local channels = Audio.processorChannels()
source:toBlob(rate, channels):andThen(function(pcm)
    -- Immutable interleaved little-endian f32 samples; transferable to processors.
    Audio.start("sample-player", { samples = pcm, channels = channels, sampleRate = rate })
end)
```

`toBlob` runs decoding through the runtime work pool and settles on the owner VM.
Cancelling its promise cancels delivery and requests decoder cancellation. The
shared-object budget reserves the estimated decode size, refunds that reservation
on settlement/cancellation, and charges the actual immutable blob on success.
Empty/failed decodes reject. Zero rate or channel arguments retain source format.

For incremental decode, `stream(rate, channels, chunkFrames, bufferFrames)` takes
explicit frame counts. Its promise settles after initial PCM or an end marker is
buffered. This is a Luau adaptation of upstream's public host interface; it does
not claim the private SDK's convenience defaults or duration-based options.

```lua
source:stream(rate, channels, 1024, 4096):andThen(function(stream)
    stream.loop = true
    Audio.start("stream-player", stream.descriptor)
    -- Keep stream to control subsequent seeking or looping.
    currentStream = stream
end)
```

An isolated processor creates `local reader = Audio.streamReader(input)` from
that descriptor. `reader:readInto(buffer)` writes interleaved f32 PCM without
blocking or decoding and returns the number of complete frames written. Read
`reader.frame`, `reader.ended`, `reader.sampleRate`, and `reader.channels` for
position/format. An empty read before `ended` means no samples are currently
available; the processor decides how to fill the rest of its output block.
Main scripts can also use `stream:reader()`.

`stream:seekFrame(frame)` returns whether the decoder accepted the seek. Seek
publishes a new generation before discarding old queued records, so readers
invalidate prefetched samples. `stream.loop` controls rewind at end of source;
an empty source never spins. `stream.ended` reflects the producer's end marker,
whereas `reader.ended` reflects consuming that marker. `stream:release()` retires
control immediately; the owner continues pumping while a transferred descriptor
or reader retains the stream, until the producer ends. VM teardown cancels decode
jobs and closes its pumps. No Wasm guest interpreter is involved.
