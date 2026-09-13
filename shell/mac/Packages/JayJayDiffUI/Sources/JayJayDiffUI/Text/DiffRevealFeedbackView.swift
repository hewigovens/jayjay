import AppKit
import JayJayCore

struct PendingDiffRevealFeedback {
    let feedback: ContextExpansionReveal
    let reduceMotion: Bool
}

extension ContextExpansionReveal {
    var newLineRange: ClosedRange<UInt32>? {
        guard newLines.count > 0 else { return nil }
        let (end, overflow) = newLines.start.addingReportingOverflow(newLines.count - 1)
        guard !overflow else { return nil }
        return newLines.start ... end
    }
}

enum DiffContextRevealFeedbackPolicy {
    static let maximumAnimatedLineCount: UInt32 = 100

    static func shouldAnimate(
        feedback: ContextExpansionReveal,
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
