# Luau-controlled Rust audio processors

Historical plan for PR #1193. Superseded by
`luau-authored-audio-processors-plan.md`: authors now define the DSP in Luau.

Implement the approved fork-specific API. Luau is the control language; no Wasm
engine, Luau sample callbacks, compatibility shims, or new dependency campaign.
Do not load implement/TDD skills. Existing asset playback stays intact.

## Contract

`Audio.createProcessor("oscillator", {waveform="sine", frequency=440, gain=0})`
returns a prepared, silent handle. V1 supports sine only; reject unsupported names
and fields rather than implying other waveforms exist. Frequency must be finite,
positive, and below Nyquist; gain finite in [0,1]. Defaults as above.

- `start({atFrame?})` schedules a one-shot start, default next available block.
- `set({frequency?,gain?}, {atFrame?,ramp?})` submits one atomic parameter update.
- `stop({atFrame?,fade?})` is terminal and idempotent; release attachment after fade.
  At the stop frame, it supersedes previously accepted updates scheduled within or
  after that fade, so they cannot restart the processor or override the fade.
- `status()` returns state (prepared/scheduled/running/stopping/stopped/failed) and
  error where applicable; `meter()` returns latest peak and RMS.
- Frame times are nonnegative exact integers; ramps/fades are finite nonnegative
  seconds. Validate and report errors on the script thread. Late commands execute
  at the next available frame. Equal-time events preserve submission order.
- Future commands use bounded capacity and reject overflow without partial change.
  Never silently drop accepted commands. Discrete start/stop and numeric ramps
  execute at sample offsets inside a mixer block; phase persists across changes.
- No Luau access, dynamic allocation, blocking control locks, or object destruction
  introduced on the audio callback by this feature. Prepare storage off-thread.
- Script-instance teardown cancels/detaches owned processors independently of Lua
  garbage collection. Device format/block size is host-owned. No routing/effects
  graph, custom DSP scripting, or sample-player feature in this change.

## Execution phases

1. Implement Rust oscillator/control scheduling and Luau bindings/lifetime ownership.
   Add focused semantic coverage for timing, ramps, ordering, queue pressure, stop,
   validation, and teardown. These are behavior checks, not a TDD workflow.
2. Freeze implementation. Independent review one: API/behavior against this contract
   and upstream lifecycle lessons (source is not authority for our new API).
3. Separate independent review two: Rust safety, callback behavior, Lua ownership,
   platform integration. Correct and rereview proven findings.
4. Run relevant pool tests/builds. Extend the existing device qualification hosts
   to exercise the shipped processor; exercise real Luau controls in validation,
   not just a handwritten substitute. Validate Mac, wireless Bat Phone, Android
   emulator. Preserve measured limits; do not claim acoustic or physical Android
   coverage. Fix blockers, publish results and merge after validation.

Examples and an implementation/results note will document the final public API.
Do not add broad ledgers or unrelated performance/consumer/release work.
