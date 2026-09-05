import CoreGraphics

/// Drag hit-testing needs current frames, but publishing layout measurements back to SwiftUI can keep the lazy stack laying itself out.
final class DAGRowFrameCache {
    var frames: [String: CGRect] = [:]
}
