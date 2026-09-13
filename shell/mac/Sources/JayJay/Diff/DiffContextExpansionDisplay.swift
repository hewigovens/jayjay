import JayJayCore

struct DiffContextExpansionDisplay {
    var errorMessage: String?
    var revealFeedback: ContextExpansionReveal?
    /// Kept across resets so representables never receive a reused selection token.
    var selectionGeneration: UInt64 = 0

    mutating func clearRevealFeedback(generation: UInt64) {
        guard revealFeedback?.generation == generation else { return }
        revealFeedback = nil
    }

    mutating func reset() {
        errorMessage = nil
        revealFeedback = nil
    }
}
