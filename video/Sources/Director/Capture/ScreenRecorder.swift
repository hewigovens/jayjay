import AVFoundation
import ScreenCaptureKit

/// Crops the display, because capturing app windows swaps their traffic lights for a sharing badge.
final class ScreenRecorder {
    let startedAt: Date
    private let stream: SCStream
    private let writer: FrameWriter

    private init(stream: SCStream, writer: FrameWriter, startedAt: Date) {
        self.stream = stream
        self.writer = writer
        self.startedAt = startedAt
    }

    static func start(rect: CGRect, output: URL, fps: Int = 60) async throws -> ScreenRecorder {
        let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
        guard let display = content.displays.first(where: { $0.frame.contains(CGPoint(x: rect.midX, y: rect.midY)) }) else {
            throw Failure("the window is not on a capturable display")
        }
        let filter = SCContentFilter(display: display, excludingWindows: [])
        let scale = CGFloat(filter.pointPixelScale)
        let config = SCStreamConfiguration()
        config.sourceRect = rect.offsetBy(dx: -display.frame.minX, dy: -display.frame.minY)
        config.width = Int(rect.width * scale)
        config.height = Int(rect.height * scale)
        config.minimumFrameInterval = CMTime(value: 1, timescale: CMTimeScale(fps))
        config.showsCursor = false
        config.pixelFormat = kCVPixelFormatType_32BGRA
        config.colorSpaceName = CGColorSpace.sRGB
        config.queueDepth = 6

        let writer = try FrameWriter(output: output, width: config.width, height: config.height, fps: fps)
        let stream = SCStream(filter: filter, configuration: config, delegate: nil)
        try stream.addStreamOutput(writer, type: .screen, sampleHandlerQueue: writer.queue)
        try await stream.startCapture()
        let deadline = Date().addingTimeInterval(5)
        while Date() < deadline {
            if let startedAt = writer.startedAt {
                return ScreenRecorder(stream: stream, writer: writer, startedAt: startedAt)
            }
            try await Task.sleep(for: .milliseconds(20))
        }
        try? await stream.stopCapture()
        throw Failure("the screen sent no frames; allow Screen Recording for the terminal that runs record")
    }

    func stop() async throws {
        try? await stream.stopCapture()
        try await writer.finish()
    }
}
