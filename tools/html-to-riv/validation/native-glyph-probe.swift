// Diagnostic macOS font rasterizer. Input positions are exported by the actual
// runtime; this does not perform shaping, layout or browser measurement.
import Foundation
import CoreGraphics
import CoreText
import ImageIO

struct Glyph: Decodable { let id: UInt16; let size: CGFloat; let x: CGFloat; let y: CGFloat; let color: UInt32 }
struct Run: Decodable { let matrix: [CGFloat]; let glyphs: [Glyph] }
let args = CommandLine.arguments
if args.count != 7 {
    fatalError("native-glyph-probe <font.ttf> <glyphs.json> <width> <height> <smooth|plain|mask-smooth|mask-plain> <out.png>")
}
let runs = try JSONDecoder().decode([Run].self, from: Data(contentsOf: URL(fileURLWithPath: args[2])))
let width = Int(args[3])!, height = Int(args[4])!
precondition(width > 0 && height > 0 && width <= 8192 && height <= 8192)
precondition(["smooth", "plain", "mask-smooth", "mask-plain"].contains(args[5]))
let provider = CGDataProvider(data: try Data(contentsOf: URL(fileURLWithPath: args[1])) as CFData)!
let graphicsFont = CGFont(provider)!
let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8,
    bytesPerRow: width * 4, space: colorSpace,
    bitmapInfo: CGBitmapInfo.byteOrder32Big.rawValue | CGImageAlphaInfo.premultipliedLast.rawValue)!
if args[5].hasPrefix("mask-") {
    context.clear(CGRect(x: 0, y: 0, width: width, height: height))
} else {
    context.setFillColor(red: 1, green: 1, blue: 1, alpha: 1)
    context.fill(CGRect(x: 0, y: 0, width: width, height: height))
}
context.translateBy(x: 0, y: CGFloat(height))
context.scaleBy(x: 1, y: -1)
context.setAllowsAntialiasing(true)
context.setShouldAntialias(true)
context.setAllowsFontSmoothing(true)
context.setShouldSmoothFonts(args[5].hasSuffix("smooth"))
context.setAllowsFontSubpixelPositioning(true)
context.setShouldSubpixelPositionFonts(true)
context.setAllowsFontSubpixelQuantization(false)
context.setShouldSubpixelQuantizeFonts(false)
var fonts: [CGFloat: CTFont] = [:]
var count = 0
for run in runs {
    precondition(run.matrix.count == 6 && run.matrix.allSatisfy { $0.isFinite })
    let m = run.matrix
    context.saveGState()
    context.concatenate(CGAffineTransform(a:m[0],b:m[1],c:m[2],d:m[3],tx:m[4],ty:m[5]))
    for item in run.glyphs {
        precondition(item.size > 0 && item.size.isFinite && item.x.isFinite && item.y.isFinite)
        let font = fonts[item.size] ?? CTFontCreateWithGraphicsFont(graphicsFont, item.size, nil, nil)
        fonts[item.size] = font
        let color = item.color
        context.setFillColor(red:CGFloat((color >> 16) & 255)/255,green:CGFloat((color >> 8) & 255)/255,
            blue:CGFloat(color & 255)/255,alpha:CGFloat(color >> 24)/255)
        context.saveGState()
        context.translateBy(x:item.x,y:item.y)
        context.scaleBy(x:1,y:-1)
        context.textMatrix = .identity
        var glyph = item.id
        var position = CGPoint.zero
        CTFontDrawGlyphs(font, &glyph, &position, 1, context)
        context.restoreGState()
        count += 1
    }
    context.restoreGState()
}
let destination = CGImageDestinationCreateWithURL(URL(fileURLWithPath:args[6]) as CFURL,"public.png" as CFString,1,nil)!
CGImageDestinationAddImage(destination,context.makeImage()!,nil)
precondition(CGImageDestinationFinalize(destination))
print("Rendered \(count) runtime glyphs with \(args[5]) font smoothing")
