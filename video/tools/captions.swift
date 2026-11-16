// usage: swift video/tools/captions.swift [--from <seconds>] [--rings-only] <take.mov> <out.mp4>
// Overlays a take's cues with ffmpeg: a caption card for each label that is a sentence, and a ring
// at each click. Cues before --from (default 1.0, the prelude) are skipped.
import AppKit
import Foundation

var args = Array(CommandLine.arguments.dropFirst())
var from = 1.0, ringsOnly = false
while let flag = args.first, flag.hasPrefix("--") {
    args.removeFirst()
    switch flag {
        case "--from": from = Double(args.removeFirst())!
        case "--rings-only": ringsOnly = true
        default: fatalError("unknown option \(flag)")
    }
}
guard args.count == 2 else { fatalError("usage: captions.swift [--from <seconds>] [--rings-only] <take.mov> <out.mp4>") }
let (movie, output) = (args[0], args[1])
let cuesPath = URL(fileURLWithPath: movie).deletingPathExtension().appendingPathExtension("cues.jsonl").path
struct Cue: Decodable { let label: String; let t: Double; let x: Double?; let y: Double? }
let cues = try String(contentsOfFile: cuesPath, encoding: .utf8).split(separator: "\n")
    .compactMap { try? JSONDecoder().decode(Cue.self, from: Data($0.utf8)) }
    .filter { $0.t >= from }

func probe(_ key: String) -> Double {
    let p = Process(); p.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    p.arguments = ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=\(key)", "-of", "csv=p=0", movie]
    let pipe = Pipe(); p.standardOutput = pipe; try! p.run(); p.waitUntilExit()
    return Double(String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)) ?? 0
}
let width = probe("width"), height = probe("height")
let dir = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("captions-\(getpid())")
try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

func png(_ size: NSSize, _ draw: () -> Void, to url: URL) {
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(size.width), pixelsHigh: Int(size.height), bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState(); NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep); draw(); NSGraphicsContext.restoreGraphicsState()
    try! rep.representation(using: .png, properties: [:])!.write(to: url)
}
let scale = width / 2880 * 2
let font = NSFont.systemFont(ofSize: 30 * scale, weight: .semibold)
let captions = cues.filter { $0.label.contains(" ") }  // one-word labels are cue names, not captions
for (i, cue) in captions.enumerated() {
    let attr = NSAttributedString(string: cue.label, attributes: [.font: font, .foregroundColor: NSColor.white])
    let text = attr.boundingRect(with: NSSize(width: width * 0.8, height: 400 * scale), options: [.usesLineFragmentOrigin])
    let pad = 22 * scale
    let size = NSSize(width: ceil(text.width + pad * 2), height: ceil(text.height + pad * 1.4))
    png(size, {
        let ctx = NSGraphicsContext.current!.cgContext
        ctx.setFillColor(CGColor(red: 0.08, green: 0.08, blue: 0.1, alpha: 0.88))
        ctx.addPath(CGPath(roundedRect: CGRect(origin: .zero, size: size), cornerWidth: 14 * scale, cornerHeight: 14 * scale, transform: nil))
        ctx.fillPath()
        attr.draw(with: NSRect(x: pad, y: pad * 0.7, width: text.width, height: text.height), options: [.usesLineFragmentOrigin])
    }, to: dir.appendingPathComponent("c\(i).png"))
}
let ring = 70 * scale
png(NSSize(width: ring, height: ring), {
    NSColor.systemYellow.withAlphaComponent(0.9).setStroke()
    let path = NSBezierPath(ovalIn: NSRect(x: 5 * scale, y: 5 * scale, width: ring - 10 * scale, height: ring - 10 * scale)); path.lineWidth = 6 * scale; path.stroke()
}, to: dir.appendingPathComponent("ring.png"))

var inputs = ["-y", "-i", movie]
var filters: [String] = []
var last = "0:v"
for (i, cue) in captions.enumerated() where !ringsOnly {
    // Captions lead their action so the viewer reads what is about to happen before it does.
    let lead = 1.6
    let start = max(0, cue.t - lead)
    let end = i + 1 < captions.count ? captions[i + 1].t - lead : cue.t + 4
    inputs += ["-i", dir.appendingPathComponent("c\(i).png").path]
    filters.append("[\(last)][\(i + 1):v]overlay=x=(W-w)/2:y=H-h-\(Int(40 * scale)):enable='between(t,\(start),\(end))'[v\(i)]")
    last = "v\(i)"
}
inputs += ["-i", dir.appendingPathComponent("ring.png").path]
let ringInput = inputs.filter { $0 == "-i" }.count - 1
for (i, cue) in cues.enumerated() {
    guard let x = cue.x, let y = cue.y else { continue }
    filters.append("[\(last)][\(ringInput):v]overlay=x=\(Int(x * width - ring / 2)):y=\(Int(y * height - ring / 2)):enable='between(t,\(max(0, cue.t - 1.2)),\(cue.t + 0.4))'[r\(i)]")
    last = "r\(i)"
}
let p = Process(); p.executableURL = URL(fileURLWithPath: "/usr/bin/env")
p.arguments = ["ffmpeg", "-v", "error"] + inputs + ["-filter_complex", filters.joined(separator: ";") + ";[\(last)]scale=iw/2:-2[out]", "-map", "[out]", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20", output]
try p.run(); p.waitUntilExit()
try? FileManager.default.removeItem(at: dir)
exit(p.terminationStatus)
