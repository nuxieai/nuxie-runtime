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
byte strings, dense arrays and string-keyed records. Shared Ring/Snapshot handles
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
The oscillator is now Luau code, not a fixed selection in the runtime. Browser
AudioWorklet integration and full editor export validation are not established by
native device validation; consult the validation results for actual coverage.
