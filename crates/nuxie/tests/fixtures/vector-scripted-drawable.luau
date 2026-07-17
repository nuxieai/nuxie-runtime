local Palette = require("Palette")
Palette.protocolBoots += 1
local protocolBoot = Palette.protocolBoots

return function(_context)
    return {
        frames = 0,
        offset = protocolBoot * 100,
        init = function(self)
            self.path = Path.new()
            self.path:moveTo(Vector(0, 0))
            self.path:lineTo(Vector(12, 0))
            self.path:lineTo(Vector(12, 8))
            self.path:close()
            self.paint = Palette.paint
            return true
        end,
        advance = function(self, _seconds)
            self.frames += 1
            return true
        end,
        update = function(self)
            self.offset = protocolBoot * 100 + self.frames * 3
        end,
        draw = function(self, renderer)
            renderer:save()
            renderer:transform(Mat2D.withTranslation(self.offset, 2))
            renderer:drawPath(self.path, self.paint)
            renderer:restore()
        end,
    }
end
