-- Nuxie bench (not upstream): one audio processor block as the Luau audio
-- engine runs it (crates/nuxie-scripting/src/vm/luau_audio_processor_engine.rs):
-- `process(self, out, frames, start)` writes an interleaved stereo f32 block
-- with `buffer.writef32`, one call per sample and channel. 1024 frames, 48 kHz.
local function prequire(name) local success, result = pcall(require, name); return success and result end
local bench = script and require(script.Parent.bench_support) or prequire("bench_support") or require("../bench_support")

local FRAMES = 1024
local BLOCKS = 32

local processor = { phase = 0, step = 440 / 48000 * 2 * math.pi, gain = 0.25 }

function processor:process(out, frames, start)
    local phase, step, gain = self.phase, self.step, self.gain
    for i = 0, frames - 1 do
        local sample = math.sin(phase) * gain
        buffer.writef32(out, i * 8, sample)
        buffer.writef32(out, i * 8 + 4, -sample)
        phase += step
    end
    self.phase = phase % (2 * math.pi)
end

function test()
    local out = buffer.create(FRAMES * 2 * 4)
    local ts0 = os.clock()
    for block = 0, BLOCKS - 1 do
        processor:process(out, FRAMES, block * FRAMES)
    end
    local ts1 = os.clock()
    assert(buffer.readf32(out, 0) == buffer.readf32(out, 0))
    return ts1 - ts0
end

bench.runCode(test, "AudioBlock: stereo sine, 1024 frames")
