import AppKit
import AVFoundation

/// Converts a take into an H.264 MP4 that every browser plays, plus a PNG of its first frame for the poster.
struct WebClip {
    let take: URL
    let from: Double
    let out: URL

    private static let fps = 30
    private static let bitRate = 1_500_000

    func write() async throws {
        let asset = AVURLAsset(url: take)
        guard let track = try await asset.loadTracks(withMediaType: .video).first else { throw Failure("\(take.path) has no video") }
        let (size, duration) = try await track.load(.naturalSize, .timeRange)
        let start = CMTime(seconds: from, preferredTimescale: 600)
        guard start < duration.end else { throw Failure("--from \(from) is past the end of \(take.lastPathComponent)") }
        try FileManager.default.createDirectory(at: out, withIntermediateDirectories: true)
        let name = take.deletingPathExtension().lastPathComponent
        try await writePoster(asset, at: start, to: out.appendingPathComponent("\(name).png"))
        try writeVideo(track, asset: asset, range: CMTimeRange(start: start, end: duration.end), size: size, to: out.appendingPathComponent("\(name).mp4"))
        try printCues(in: take.deletingPathExtension().appendingPathExtension("cues.jsonl"))
    }

    private func writePoster(_ asset: AVAsset, at time: CMTime, to url: URL) async throws {
        let generator = AVAssetImageGenerator(asset: asset)
        generator.requestedTimeToleranceBefore = .zero
        generator.requestedTimeToleranceAfter = .zero
        let image = try await generator.image(at: time).image
        guard let png = NSBitmapImageRep(cgImage: image).representation(using: .png, properties: [:]) else { throw Failure("cannot encode \(url.lastPathComponent)") }
        try png.write(to: url)
    }

    private func writeVideo(_ track: AVAssetTrack, asset: AVAsset, range: CMTimeRange, size: CGSize, to url: URL) throws {
        let reader = try AVAssetReader(asset: asset)
        reader.timeRange = range
        let output = AVAssetReaderTrackOutput(track: track, outputSettings: [kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA])
        reader.add(output)
        try? FileManager.default.removeItem(at: url)
        let writer = try AVAssetWriter(outputURL: url, fileType: .mp4)
        writer.shouldOptimizeForNetworkUse = true
        let input = AVAssetWriterInput(mediaType: .video, outputSettings: [
            AVVideoCodecKey: AVVideoCodecType.h264,
            AVVideoWidthKey: Int(size.width),
            AVVideoHeightKey: Int(size.height),
            AVVideoCompressionPropertiesKey: [
                AVVideoAverageBitRateKey: Self.bitRate,
                AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
                AVVideoExpectedSourceFrameRateKey: Self.fps,
                AVVideoMaxKeyFrameIntervalKey: Self.fps * 2
            ]
        ])
        writer.add(input)
        guard reader.startReading(), writer.startWriting() else {
            throw Failure("cannot convert \(take.lastPathComponent): \(String(describing: reader.error ?? writer.error))")
        }
        writer.startSession(atSourceTime: range.start)
        do {
            try copy(from: output, reading: reader, to: input, writing: writer)
        } catch {
            writer.cancelWriting()
            try? FileManager.default.removeItem(at: url)
            throw error
        }
        let done = DispatchSemaphore(value: 0)
        writer.finishWriting { done.signal() }
        done.wait()
        guard writer.status == .completed else { throw Failure("cannot write \(url.path): \(String(describing: writer.error))") }
    }

    private func copy(from output: AVAssetReaderOutput, reading reader: AVAssetReader, to input: AVAssetWriterInput, writing writer: AVAssetWriter) throws {
        var nextFrame = CMTime.negativeInfinity
        let frameLength = CMTime(value: 1, timescale: CMTimeScale(Self.fps))
        while let sample = output.copyNextSampleBuffer() {
            let time = CMSampleBufferGetPresentationTimeStamp(sample)
            guard time >= nextFrame else { continue }
            nextFrame = time + frameLength
            while !input.isReadyForMoreMediaData {
                guard writer.status == .writing else { throw Failure("cannot encode \(take.lastPathComponent): \(String(describing: writer.error))") }
                usleep(1000)
            }
            guard input.append(sample) else { throw Failure("cannot encode \(take.lastPathComponent): \(String(describing: writer.error))") }
        }
        guard reader.status == .completed else { throw Failure("cannot read \(take.lastPathComponent): \(String(describing: reader.error))") }
        input.markAsFinished()
    }

    private func printCues(in url: URL) throws {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return }
        for line in text.split(separator: "\n") {
            guard let cue = try? JSONDecoder().decode(Cue.self, from: Data(line.utf8)), cue.t >= from, let x = cue.x, let y = cue.y else { continue }
            print(String(format: "%.2f %.3f %.3f %@", cue.t - from, x, y, cue.label))
        }
    }

    private struct Cue: Decodable {
        let label: String
        let t: Double
        let x, y: Double?
    }
}
