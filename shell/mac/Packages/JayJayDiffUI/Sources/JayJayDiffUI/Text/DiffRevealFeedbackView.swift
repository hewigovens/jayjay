import AppKit
import JayJayCore

struct PendingDiffRevealFeedback {
    let feedback: DiffContextRevealFeedback
    let reduceMotion: Bool
}

enum DiffContextRevealFeedbackPolicy {
    static let maximumAnimatedLineCount: UInt32 = 100

    static func shouldAnimate(
        feedback: DiffContextRevealFeedback,
        reduceMotion: Bool
    ) -> Bool {
        !reduceMotion
            && feedback.newLines.count > 0
            && feedback.newLines.count <= maximumAnimatedLineCount
            && feedback.newLineRange != nil
    }
}

final class DiffRevealFeedbackView: NSView {
    override func hitTest(_ point: NSPoint) -> NSView? {
        nil
    }
}
