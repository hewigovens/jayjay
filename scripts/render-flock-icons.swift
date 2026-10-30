#!/usr/bin/env swift

import AppKit

func fail(_ message: String, code: Int32 = 1) -> Never {
    fputs("\(message)\n", stderr)
    exit(code)
}

guard CommandLine.arguments.count == 3 else {
    fail("usage: render-flock-icons.swift ARTWORK_PNG OUTPUT_PNG", code: 2)
}

guard let artwork = NSImage(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1])) else {
    fail("Could not load artwork at \(CommandLine.arguments[1])")
}

let canvasSize = 1024
let canvasRect = NSRect(x: 0, y: 0, width: canvasSize, height: canvasSize)
guard let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: canvasSize,
    pixelsHigh: canvasSize,
    bitsPerSample: 8,
    samplesPerPixel: 4,
    hasAlpha: true,
    isPlanar: false,
    colorSpaceName: .deviceRGB,
    bytesPerRow: 0,
    bitsPerPixel: 0
), let context = NSGraphicsContext(bitmapImageRep: bitmap)
else {
    fail("Could not create Flock icon bitmap")
}

NSGraphicsContext.current = context
context.imageInterpolation = .high
/// Matches the card shadow in render-dock-icons.swift so every Dock choice sits at the same depth.
let shadow = NSShadow()
shadow.shadowColor = .black.withAlphaComponent(0.28)
shadow.shadowBlurRadius = 34
shadow.shadowOffset = NSSize(width: 0, height: -8)
shadow.set()
artwork.draw(in: canvasRect, from: .zero, operation: .sourceOver, fraction: 1)

guard let data = bitmap.representation(using: .png, properties: [:]) else {
    fail("Could not encode Flock icon PNG")
}

do {
    try data.write(to: URL(fileURLWithPath: CommandLine.arguments[2]), options: .atomic)
} catch {
    fail("Could not write Flock icon to \(CommandLine.arguments[2]): \(error)")
}
