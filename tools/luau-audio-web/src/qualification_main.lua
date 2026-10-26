return {
    init = function(self)
        self.changes = Audio.Ring.create(4096)
        self.meter = Audio.Snapshot.create(4096)
        local source = buffer.create(1)
        buffer.writeu8(source, 0, 128)
        self.blob = Audio.Blob.from(source)
        buffer.writeu8(source, 0, 1) -- the processor must see the immutable copy
        self.region = Audio.Region.create(8)
        self.region:atomicStore(4, 1)
        self.tone = Audio.start("qualification", {
            changes=self.changes, meter=self.meter, blob=self.blob, region=self.region,
        })
        return true
    end,
    advance = function(self)
        if self.spawnFailures then self.spawnFailures=false;self:failOthers() end
        return true
    end,
    gain = function(self, value)
        assert(self.changes:send({gain=value}))
    end,
    regionGain = function(self, value)
        self.region:atomicStore(4, value)
    end,
    failOthers = function(self)
        self.thrower = Audio.start("thrower")
        self.runaway = Audio.start("runaway")
    end,
    retainedState = function(self) return self.tone.state end,
    stopTone = function(self) self.tone:stop() end,
    report = function(self)
        local latest = self.meter:latest()
        return {
            state=self.tone.state,
            failure=self.tone.failed,
            thrower=self.thrower and self.thrower.state or "absent",
            throwerFailure=self.thrower and self.thrower.failed or nil,
            runaway=self.runaway and self.runaway.state or "absent",
            runawayFailure=self.runaway and self.runaway.failed or nil,
            meter=latest and latest.value or nil,
            sequence=latest and latest.sequence or 0,
            blocks=self.region:atomicLoad(0),
        }
    end,
}
