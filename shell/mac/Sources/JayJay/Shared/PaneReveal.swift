import AppKit
import Observation
import QuartzCore

/// How far a pane is shown, driven frame by frame so toolbar items can follow the same value as the pane.
@MainActor @Observable
final class PaneReveal {
    static let duration: CFTimeInterval = 0.25

    private(set) var progress: CGFloat {
        didSet {
            if isShown != (progress > 0) {
                isShown = progress > 0
            }
        }
    }

    /// Stored, so a view that only reads this is not invalidated by every frame of `progress`.
    private(set) var isShown: Bool
    @ObservationIgnored private var link: CADisplayLink?

    init(shown: Bool) {
        progress = shown ? 1 : 0
        isShown = shown
    }

    /// Follows SwiftUI's easeInOut curve so it matches panes animated by SwiftUI.
    func animate(toShown shown: Bool, in window: NSWindow?, animated: Bool) {
        link?.invalidate()
        link = nil
        let from = progress
        let to: CGFloat = shown ? 1 : 0
        guard animated, from != to, let window else {
            progress = to
            return
        }
        let start = CACurrentMediaTime()
        let target = DisplayLinkTarget { [weak self] link in
            let time = min(1, (link.targetTimestamp - start) / Self.duration)
            self?.progress = from + (to - from) * Self.easeInOut(time)
            guard time >= 1 else { return }
            link.invalidate()
            self?.link = nil
        }
        let link = window.displayLink(target: target, selector: #selector(DisplayLinkTarget.step(_:)))
        link.add(to: .main, forMode: .common)
        self.link = link
    }

    private static func easeInOut(_ time: CGFloat) -> CGFloat {
        func curve(_ u: CGFloat, _ p1: CGFloat, _ p2: CGFloat) -> CGFloat {
            3 * (1 - u) * (1 - u) * u * p1 + 3 * (1 - u) * u * u * p2 + u * u * u
        }
        var u = time
        for _ in 0 ..< 8 {
            let slope = 3 * (1 - u) * (1 - u) * 0.42 + 6 * (1 - u) * u * (0.58 - 0.42) + 3 * u * u * (1 - 0.58)
            guard slope > 1e-6 else { break }
            u = min(1, max(0, u - (curve(u, 0.42, 0.58) - time) / slope))
        }
        return curve(u, 0, 1)
    }
}

private final class DisplayLinkTarget: NSObject {
    let onFrame: (CADisplayLink) -> Void

    init(onFrame: @escaping (CADisplayLink) -> Void) {
        self.onFrame = onFrame
    }

    @objc func step(_ link: CADisplayLink) {
        onFrame(link)
    }
}
