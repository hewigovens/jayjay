import AVFoundation
import ScreenCaptureKit

/// The screen sends frames only on change, so the newest one is appended on a fixed clock.
final class FrameWriter: NSObject, SCStreamOutput, @unchecked Sendable {
    let queue = DispatchQueue(label: "director.frames")
    private let writer: AVAssetWriter
    private let input: AVAssetWriterInput
    private let adaptor: AVAssetWriterInputPixelBufferAdaptor
    private let fps: Int
    private var timer: DispatchSourceTimer?
    private var latest: CVPixelBuffer?
    private var start: CFTimeInterval?
    private var startDate: Date?
    private var nextIndex: Int64 = 0

    init(output: URL, width: Int, height: Int, fps: Int) throws {
        self.fps = fps
        try? FileManager.default.removeItem(at: output)
        writer = try AVAssetWriter(outputURL: output, fileType: .mov)
        input = AVAssetWriterInput(mediaType: .video, outputSettings: [
            AVVideoCodecKey: AVVideoCodecType.hevc,
            AVVideoWidthKey: width,
            AVVideoHeightKey: height,
            AVVideoCompressionPropertiesKey: [
                AVVideoAverageBitRateKey: 60_000_000,
                AVVideoExpectedSourceFrameRateKey: fps,
                AVVideoMaxKeyFrameIntervalKey: fps
            ]
        ])
        input.expectsMediaDataInRealTime = true
        adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: nil)
        writer.add(input)
        guard writer.startWriting() else {
            throw Failure("cannot write \(output.path): \(writer.error.map(String.init(describing:)) ?? "unknown error")")
        }
        super.init()
        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(deadline: .now(), repeating: .nanoseconds(1_000_000_000 / fps), leeway: .milliseconds(1))
        timer.setEventHandler { [weak self] in self?.tick() }
        timer.resume()
        self.timer = timer
    }

    var startedAt: Date? {
        queue.sync { startDate }
    }

    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen, sampleBuffer.isValid,
              let attachments = CMSampleBufferGetSampleAttachmentsArray(sampleBuffer, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
              let rawStatus = attachments.first?[.status] as? Int,
              SCFrameStatus(rawValue: rawStatus) == .complete,
              let frame = sampleBuffer.imageBuffer
        else { return }
        latest = frame
    }

    private func tick() {
        guard let latest else { return }
        let now = CACurrentMediaTime()
        if start == nil {
            start = now
            startDate = Date()
            writer.startSession(atSourceTime: .zero)
        }
        let due = Int64(((now - (start ?? now)) * Double(fps)).rounded(.down))
        // A late tick fills every slot it missed, so the clip keeps real time.
        while nextIndex <= due, input.isReadyForMoreMediaData {
            adaptor.append(latest, withPresentationTime: CMTime(value: nextIndex, timescale: CMTimeScale(fps)))
            nextIndex += 1
        }
    }

    func finish() async throws {
        await withCheckedContinuation { continuation in
            queue.async {
                self.timer?.cancel()
                continuation.resume()
            }
        }
        guard start != nil else { throw Failure("no frames were captured") }
        input.markAsFinished()
        await writer.finishWriting()
        if let error = writer.error {
            throw error
        }
    }
}
