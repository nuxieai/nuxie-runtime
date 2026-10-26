-- The DSP algorithm belongs to the author module, not to Rust.
return {
    init = function(self, rate, maxFrames, channels, input)
        self.rate = rate
        self.channels = channels
        self.phase = 0
        self.frequency = input.frequency
        self.gain = input.gain
        self.changes = input.changes
        self.meter = input.meter
        self.blob = input.blob
        self.region = input.region
        self.byte = buffer.create(1)
        assert(self.blob.size == 1 and self.region.size == 4)
        assert(buffer.readu8(self.blob:toBuffer(), 0) == 128)
        self.next = {}
        self.report = {frames = 0, frequency = self.frequency, blocks = 0, blobByte = 128}
    end,
    process = function(self, out, frames, startFrame)
        while self.changes:receiveInto(self.next) do
            self.frequency = self.next.frequency
            self.gain = self.next.gain
        end
        assert(self.blob:read(0, self.byte) == 1)
        local byte = buffer.readu8(self.byte, 0)
        assert(byte == 128)
        local blocks = self.region:atomicAdd(0, 1) + 1
        local phase = self.phase
        local step = 2 * math.pi * self.frequency / self.rate
        for i = 0, frames - 1 do
            local sample = math.sin(phase) * self.gain
            for channel = 0, self.channels - 1 do
                buffer.writef32(out, (i * self.channels + channel) * 4, sample)
            end
            phase = (phase + step) % (2 * math.pi)
        end
        self.phase = phase
        self.report.frames = startFrame + frames
        self.report.frequency = self.frequency
        self.report.blocks = blocks
        self.report.blobByte = byte
        assert(self.meter:publish(self.report))
    end,
}
