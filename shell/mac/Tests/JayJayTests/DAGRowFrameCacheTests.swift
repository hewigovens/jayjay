import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class DAGRowFrameCacheTests: XCTestCase {
    func testFramePreferencesReplaceDragTargetsWithoutUpdatingTheOwner() {
        let measurements = Measurements()
        let probe = Probe()
        let window = NSWindow(
            contentRect: CGRect(x: 0, y: 0, width: 300, height: 400),
            styleMask: [.borderless], backing: .buffered, defer: false
        )
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = NSHostingView(rootView: FrameTrackingView(measurements: measurements, probe: probe))
        window.layoutIfNeeded()
        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
        let initialBodyCount = probe.bodyCount

        for count in [7, 200, 22, 0] {
            let frames = Dictionary(uniqueKeysWithValues: (0 ..< count).map { index in
                ("\(count)-\(index)", CGRect(x: 0, y: index * 80, width: 300, height: 80))
            })
            measurements.frames = frames
            window.layoutIfNeeded()
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))

            XCTAssertEqual(probe.cache?.frames, frames, "Drag targets must be ready before a gesture, with old rows removed")
            XCTAssertEqual(probe.bodyCount, initialBodyCount, "Collecting row frames must not invalidate their owning view")
        }
    }

    @Observable
    fileprivate final class Measurements {
        var frames: [String: CGRect] = [:]
    }

    private final class Probe {
        var bodyCount = 0
        var cache: DAGRowFrameCache?
    }

    private struct FrameTrackingView: View {
        let measurements: Measurements
        let probe: Probe
        @State private var cache = DAGRowFrameCache()

        var body: some View {
            probe.bodyCount += 1
            probe.cache = cache
            return MeasuredRows(measurements: measurements)
                .onPreferenceChange(DAGRebaseRowFramePreferenceKey.self) { cache.frames = $0 }
        }
    }

    private struct MeasuredRows: View {
        let measurements: Measurements

        var body: some View {
            Color.clear.preference(key: DAGRebaseRowFramePreferenceKey.self, value: measurements.frames)
        }
    }
}
