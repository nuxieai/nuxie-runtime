# Luau audio processors

Nuxie's audio processors are implemented in Rust and controlled by Luau. The
initial processor is a sine oscillator. No Lua callback executes on the audio
thread. Ordinary `Audio.play` and its asset-playback controls are unchanged.

Create processors inside an instance's `init` or another owned callback, not at
module/prototype scope. Each instance retains its processors and cancels them on
teardown even if another Lua value retains the handle. Failed initialization also
cancels playback. There are at most 64 live processors per instance; stopped
processors do not consume this limit. In editor/tools mode, `start` reports an
error while playback is disabled.

This is a complete script-generator example. The tone fades in, changes pitch
halfway through its lifetime, then fades out. `advance` controls the sound at
ordinary script frequency; Rust interpolates the actual audio samples.

```lua
return function(context)
    local tone = nil
    local elapsed = 0
    local changed = false
    local stopped = false

    return {
        init = function(self)
            tone = Audio.createProcessor("oscillator", {
                waveform = "sine",
                frequency = 440,
                gain = 0,
            })
            tone:start()
            tone:set({gain = 0.12}, {ramp = 0.020})
            return true
        end,

        advance = function(self, seconds)
            elapsed += seconds
            if elapsed >= 0.5 and not changed then
                changed = true
                tone:set({frequency = 660}, {ramp = 0.080})
            end
            if elapsed >= 1.0 and not stopped then
                stopped = true
                tone:stop({fade = 0.050})
            end
            return true
        end,
    }
end
```

For sample-clock scheduling, use the existing `Audio.timeFrame()` and
`Audio.sampleRate()` functions. These calls belong inside an owned callback:

```lua
local chime = Audio.createProcessor("oscillator", {frequency = 880, gain = 0})
local start = Audio.timeFrame() + math.floor(Audio.sampleRate() * 0.050)
chime:start({atFrame = start})
chime:set({gain = 0.12}, {atFrame = start, ramp = 0.005})
chime:stop({atFrame = start + math.floor(Audio.sampleRate() * 0.150), fade = 0.080})
```

`atFrame` is an absolute nonnegative integer below 2^53. If omitted, a command
applies at the next available audio block; late commands apply at the next
available frame. Equal-time commands preserve submission order. `ramp` and `fade`
are nonnegative finite seconds. A parameter table is one atomic command, so a
frequency/gain change can be scheduled together. Numeric ramps run per sample.

| API | Meaning |
| --- | --- |
| `Audio.createProcessor("oscillator", parameters?)` | Prepare a silent processor; defaults: sine, 440 Hz, gain 0. |
| `processor:start({atFrame?}?)` | Schedule one start. A stopped processor cannot restart. |
| `processor:set({frequency?,gain?}, {atFrame?,ramp?}?)` | Change at least one parameter, optionally interpolate. |
| `processor:stop({atFrame?,fade?}?)` | Terminal, idempotent stop; optional fade before release. |
| `processor:status()` | Snapshot with `state` and optional `error`. |
| `processor:meter()` | Snapshot with `peak` and `rms`. |

Frequency must be finite, positive, and below half the engine sample rate. Gain
must be finite in [0,1]. Unsupported processors, waveforms, parameter names,
option names, wrong types, invalid ranges, and full command queues raise ordinary
Luau errors. V1 does not accept square/saw/noise waveforms, effects routing,
custom Luau DSP, or changes to device sample rate/block size.

```lua
local status = tone:status()
if status.state == "failed" then
    print(status.error)
end
local meter = tone:meter()
print(meter.peak, meter.rms)
```

States are `prepared`, `scheduled`, `running`, `stopping`, `stopped`, and `failed`.
Commands are bounded and rejected visibly when full; accepted commands are not
silently dropped on overflow. A terminal stop supersedes previously accepted
updates at or after its stop frame, so they cannot override its fade. Metering/status reads and command submission happen on the
normal script thread. This API is intentionally Nuxie-specific, inspired by
upstream's processor handles and control messages, not its Wasm execution API.
