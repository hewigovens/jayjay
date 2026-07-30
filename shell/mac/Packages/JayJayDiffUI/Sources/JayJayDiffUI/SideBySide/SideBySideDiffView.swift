import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
public struct SideBySideDiffView: View {
    public let diff: FileDiff
    public var onExpandContext: ((DiffContextExpansionRequest) -> Void)?
    public var resetSelectionGeneration: UInt64
    public var revealFeedback: DiffContextRevealFeedback?

    public init(
        diff: FileDiff,
        onExpandContext: ((DiffContextExpansionRequest) -> Void)? = nil,
        resetSelectionGeneration: UInt64 = 0,
        revealFeedback: DiffContextRevealFeedback? = nil
    ) {
        self.diff = diff
        self.onExpandContext = onExpandContext
        self.resetSelectionGeneration = resetSelectionGeneration
        self.revealFeedback = revealFeedback
    }

    public var body: some View {
        SideBySideRepresentable(
            diff: diff,
            onExpandContext: onExpandContext,
            resetSelectionGeneration: resetSelectionGeneration,
            revealFeedback: revealFeedback
        )
    }
}
