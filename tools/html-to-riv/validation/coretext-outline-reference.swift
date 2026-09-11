// Diagnostic only: CoreText outlines from the supplied font bytes. This does
// not establish that Chromium selected CoreText for the corresponding face.
import Foundation
import CoreText
import CoreGraphics
let url = URL(fileURLWithPath: CommandLine.arguments[1])
guard let provider = CGDataProvider(url: url as CFURL), let face = CGFont(provider) else { fatalError("invalid font") }
let font = CTFontCreateWithGraphicsFont(face, 64, nil, nil)
var output: [[String: Any]] = []
let text = CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : "qj"
for character: UniChar in text.utf16 {
    var scalar = character
    var glyph: CGGlyph = 0
    guard CTFontGetGlyphsForCharacters(font, &scalar, &glyph, 1) else { fatalError("missing glyph") }
    let path = CTFontCreatePathForGlyph(font, glyph, nil)
    var advance = CGSize.zero
    CTFontGetAdvancesForGlyphs(font, .horizontal, &glyph, &advance, 1)
    var segments: [[String: Any]] = []
    path?.applyWithBlock { pointer in
        let element = pointer.pointee
        let count: Int
        let verb: String
        switch element.type {
        case .moveToPoint: count = 1; verb = "Move"
        case .addLineToPoint: count = 1; verb = "Line"
        case .addQuadCurveToPoint: count = 2; verb = "Quad"
        case .addCurveToPoint: count = 3; verb = "Cubic"
        case .closeSubpath: count = 0; verb = "Close"
        @unknown default: fatalError("unknown path verb")
        }
        let points = (0..<count).map { index in [Double(element.points[index].x)/64, -Double(element.points[index].y)/64] }
        segments.append(["verb": verb, "points": points])
    }
    output.append(["scalar": Int(character), "glyph": Int(glyph), "segments": segments, "advanceAt24": Double(advance.width) / 64 * 24])
}
let data = try JSONSerialization.data(withJSONObject: output, options: [.prettyPrinted, .sortedKeys])
print(String(decoding: data, as: UTF8.self))
