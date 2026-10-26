-- Same module is executed offline and inside the real browser AudioWorklet.
return {
    init = function(self, rate, maxFrames, channels, input)
        assert(channels == 2 and maxFrames > 0 and rate > 0)
        self.channels = channels
        self.input = input
        self.gain = 1
        self.next = {}
        self.byte = buffer.create(1)
        self.report = {}
        assert(input.blob.size == 1 and input.region.size == 8)
        assert(buffer.readu8(input.blob:toBuffer(), 0) == 128)
    end,
    process = function(self, out, frames, startFrame)
        while self.input.changes:receiveInto(self.next) do
            self.gain = self.next.gain
        end
        assert(self.input.blob:read(0, self.byte) == 1)
        local byte = buffer.readu8(self.byte, 0)
        local factor = self.input.region:atomicLoad(4)
        local gain = self.gain * byte / 1024 * factor
        local count = self.input.region:atomicAdd(0, 1) + 1
        for i = 0, frames - 1 do
            local sample = gain * ((startFrame + i) % 64) / 64
            buffer.writef32(out, i * 8, sample)
            buffer.writef32(out, i * 8 + 4, -sample)
        end
        self.report.start = startFrame
        self.report.frames = frames
        self.report.gain = gain
        self.report.blocks = count
        self.report.blobByte = byte
        assert(self.input.meter:publish(self.report))
    end,
}
