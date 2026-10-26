-- Exercise platform C-char compiler paths before audio initialization.
assert(string.char(0, 127, 128, 255) == "\000\127\128\255")
local percent = "50%"
assert(`value {percent} complete%` == "value 50% complete%")

return {
    init = function(self)
        self.tone = Audio.createProcessor("oscillator", {
            waveform = "sine", frequency = 440, gain = 0.02,
        })
        assert(self.tone:status().state == "prepared")
        self.tone:start({atFrame = Audio.timeFrame() + 256})
        return true
    end,
    advance = function(self, phase)
        if phase == 1 then
            self.tone:set({frequency = 880, gain = 0.01}, {
                atFrame = Audio.timeFrame() + 256, ramp = 0.020,
            })
        elseif phase == 2 then
            self.tone:stop({fade = 0.020})
            -- Exercise a second, live processor owned by this script instance.
            self.owned = Audio.createProcessor("oscillator", {
                frequency = 330, gain = 0.005,
            })
            self.owned:start()
        elseif phase == 3 then
            assert(self.tone:status().state == "stopped", "fade did not stop processor")
            self.stopped = true
            -- Keep this handle reachable through the table after its script
            -- instance is dropped; ownership teardown must still silence it.
            assert(self.owned:status().state == "running", "owned processor never ran")
        end
        local meter = self.tone:meter()
        assert(meter.peak >= 0 and meter.peak <= 1)
        assert(meter.rms >= 0 and meter.rms <= 1)
        return true
    end,
}
