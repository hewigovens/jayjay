import Foundation
import JayJayCore

public struct DiffContextRevealFeedback: Hashable, Sendable {
    public let generation: UInt64
    public let newLines: LineSpan

    public init(generation: UInt64, newLines: LineSpan) {
        self.generation = generation
        self.newLines = newLines
    }

    var newLineRange: ClosedRange<UInt32>? {
        guard newLines.count > 0 else { return nil }
        let (end, overflow) = newLines.start.addingReportingOverflow(newLines.count - 1)
        guard !overflow else { return nil }
        return newLines.start ... end
    }
}
