-- Exercise platform C-char compiler paths before audio initialization.
assert(string.char(0, 127, 128, 255) == "\000\127\128\255")
local percent = "50%"
assert(`value {percent} complete%` == "value 50% complete%")

return {
    init = function(self)
        self.changes = Audio.Ring.create(256)
        self.meter = Audio.Snapshot.create(256)
        self.tone = Audio.start("oscillator", {
            frequency = 440, gain = 0.02,
            changes = self.changes, meter = self.meter,
        })
        return true
    end,
    ready = function(self)
        local tone = self.owned or self.tone
        assert(tone.state ~= "failed", tone.failed)
        return tone.state == "running"
    end,
    advance = function(self, phase)
        assert(self.tone.state ~= "failed", self.tone.failed)
        if phase == 1 then
            local snapshot = self.meter:latest()
            assert(snapshot and snapshot.value.frames > 0)
            assert(snapshot.value.frequency == 440)
            assert(self.changes:send({frequency = 880, gain = 0.01}))
        elseif phase == 2 then
            local snapshot = self.meter:latest()
            assert(snapshot and snapshot.value.frequency == 880)
            self.tone:stop()
            self.owned = Audio.start("oscillator", {
                frequency = 330, gain = 0.005,
                changes = Audio.Ring.create(256), meter = Audio.Snapshot.create(256),
            })
        elseif phase == 3 then
            assert(self.tone.state == "stopped", "processor did not stop")
            self.stopped = true
            assert(self.owned.state == "running", self.owned.failed)
            -- Main Lua table and handles remain reachable after instance teardown.
        end
        return true
    end,
}
