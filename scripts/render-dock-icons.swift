#!/usr/bin/env swift

import AppKit

func fail(_ message: String, code: Int32 = 1) -> Never {
    fputs("\(message)\n", stderr)
    exit(code)
}

guard CommandLine.arguments.count == 4 else {
    fail("usage: render-dock-icons.swift BIRD_PNG LIGHT_PNG DARK_PNG", code: 2)
}

let birdURL = URL(fileURLWithPath: CommandLine.arguments[1])
guard let birdImage = NSImage(contentsOf: birdURL) else {
    fail("Could not load jaybird artwork at \(birdURL.path)")
}

let canvasSize = 1024
let canvasRect = NSRect(x: 0, y: 0, width: canvasSize, height: canvasSize)
let iconRect = canvasRect.insetBy(dx: 100, dy: 100)
let iconPath = NSBezierPath(roundedRect: iconRect, xRadius: 205, yRadius: 205)
let darkTop = NSColor(srgbRed: 0.192, green: 0.192, blue: 0.192, alpha: 1)
let darkBottom = NSColor(srgbRed: 0.078, green: 0.078, blue: 0.078, alpha: 1)
guard let darkGradient = NSGradient(starting: darkBottom, ending: darkTop) else {
    fail("Could not create dark Dock icon gradient")
}

func setShadow(alpha: CGFloat) {
    let shadow = NSShadow()
    shadow.shadowColor = .black.withAlphaComponent(alpha)
    shadow.shadowBlurRadius = 34
    shadow.shadowOffset = NSSize(width: 0, height: -8)
    shadow.set()
}

func render(dark: Bool, to outputPath: String) {
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
        fail("Could not create Dock icon bitmap")
    }

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = context
    context.imageInterpolation = .high
    NSColor.clear.setFill()
    canvasRect.fill()

    // Give the card its own shadow; NSGradient drawing does not reliably produce it.
    NSGraphicsContext.saveGraphicsState()
    setShadow(alpha: 0.28)
    (dark ? darkBottom : .white).setFill()
    iconPath.fill()
    NSGraphicsContext.restoreGraphicsState()

    if dark {
        darkGradient.draw(in: iconPath, angle: 90)
    } else {
        NSColor.white.setFill()
        iconPath.fill()
    }

    // Icon Composer applies the configured neutral 0.36 shadow to the artwork group.
    NSGraphicsContext.saveGraphicsState()
    iconPath.addClip()
    setShadow(alpha: 0.36)
    birdImage.draw(in: iconRect, from: .zero, operation: .sourceOver, fraction: 1)
    NSGraphicsContext.restoreGraphicsState()

    if dark {
        NSColor.white.withAlphaComponent(0.18).setStroke()
        iconPath.lineWidth = 8
        iconPath.stroke()
    }
    NSGraphicsContext.restoreGraphicsState()

    guard let data = bitmap.representation(using: .png, properties: [:]) else {
        fail("Could not encode Dock icon PNG")
    }
    do {
        try data.write(to: URL(fileURLWithPath: outputPath), options: .atomic)
    } catch {
        fail("Could not write Dock icon to \(outputPath): \(error)")
    }
}

render(dark: false, to: CommandLine.arguments[2])
render(dark: true, to: CommandLine.arguments[3])
