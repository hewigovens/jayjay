#!/usr/bin/env swift

import AppKit

guard CommandLine.arguments.count == 4 else {
    fputs("usage: render-dock-icons.swift BIRD_PNG LIGHT_PNG DARK_PNG\n", stderr)
    exit(2)
}

let birdURL = URL(fileURLWithPath: CommandLine.arguments[1])
guard let birdImage = NSImage(contentsOf: birdURL) else {
    fputs("Could not load jaybird artwork at \(birdURL.path)\n", stderr)
    exit(1)
}

let canvasSize = 1024
let canvasRect = NSRect(x: 0, y: 0, width: canvasSize, height: canvasSize)
let iconRect = canvasRect.insetBy(dx: 100, dy: 100)
let iconPath = NSBezierPath(roundedRect: iconRect, xRadius: 205, yRadius: 205)

func render(background: NSColor, border: NSColor?, shadowAlpha: CGFloat, to outputPath: String) {
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
        fputs("Could not create Dock icon bitmap\n", stderr)
        exit(1)
    }

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = context
    context.imageInterpolation = .high
    NSColor.clear.setFill()
    canvasRect.fill()

    let shadow = NSShadow()
    shadow.shadowColor = .black.withAlphaComponent(shadowAlpha)
    shadow.shadowBlurRadius = 46
    shadow.shadowOffset = NSSize(width: 0, height: -18)
    shadow.set()
    background.setFill()
    iconPath.fill()

    NSGraphicsContext.saveGraphicsState()
    iconPath.addClip()
    birdImage.draw(in: iconRect, from: .zero, operation: .sourceOver, fraction: 1)
    NSGraphicsContext.restoreGraphicsState()

    if let border {
        border.setStroke()
        iconPath.lineWidth = 8
        iconPath.stroke()
    }
    NSGraphicsContext.restoreGraphicsState()

    guard let data = bitmap.representation(using: .png, properties: [:]) else {
        fputs("Could not encode Dock icon PNG\n", stderr)
        exit(1)
    }
    do {
        try data.write(to: URL(fileURLWithPath: outputPath), options: .atomic)
    } catch {
        fputs("Could not write Dock icon to \(outputPath): \(error)\n", stderr)
        exit(1)
    }
}

render(
    background: .white,
    border: nil,
    shadowAlpha: 0.30,
    to: CommandLine.arguments[2]
)
render(
    background: NSColor(srgbRed: 0.055, green: 0.063, blue: 0.082, alpha: 1),
    border: .white.withAlphaComponent(0.10),
    shadowAlpha: 0.45,
    to: CommandLine.arguments[3]
)
