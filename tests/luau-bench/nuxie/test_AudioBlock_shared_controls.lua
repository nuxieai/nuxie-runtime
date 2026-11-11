-- Nuxie bench (not upstream): the AudioBlock sine processor, reading its
-- controls from shared memory at the start of every block, as a processor
-- reads a `SharedRegion` with `region:atomicLoad(offset)`. Both bench VMs
-- install `__bench_atomic_load(index)`, a host function over a host-owned
-- array of atomic words (tools/luau-bench/src/vm.rs), so the read is a real
-- host call on each VM.
local function prequire(name) local success, result = pcall(require, name); return success and result end
local bench = script and require(script.Parent.bench_support) or prequire("bench_support") or require("../bench_support")

local FRAMES = 1024
local BLOCKS = 32
local CONTROLS = 16
local atomicLoad = __bench_atomic_load

local processor = { phase = 0, controls = table.create(CONTROLS, 0) }

function processor:process(out, frames, start)
    local controls = self.controls
    for c = 1, CONTROLS do
        controls[c] = atomicLoad(c - 1)
    end
    local step = controls[1] / 48000 * 2 * math.pi
    local gain = controls[2] / 1000
    local phase = self.phase
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

bench.runCode(test, "AudioBlock: shared controls, 1024 frames")
