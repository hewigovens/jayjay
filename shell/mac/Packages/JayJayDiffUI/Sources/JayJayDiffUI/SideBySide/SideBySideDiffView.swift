import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
public struct SideBySideDiffView: View {
    public let diff: FileDiff
    public var onExpandContext: ((ContextExpansionRequest) -> Void)?
    public var resetSelectionGeneration: UInt64
    public var revealFeedback: ContextExpansionReveal?

    public init(
        diff: FileDiff,
        onExpandContext: ((ContextExpansionRequest) -> Void)? = nil,
        resetSelectionGeneration: UInt64 = 0,
        revealFeedback: ContextExpansionReveal? = nil
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
