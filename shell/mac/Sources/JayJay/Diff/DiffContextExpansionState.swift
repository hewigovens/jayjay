import JayJayCore
import JayJayDiffUI

struct DiffContextExpansionState {
    private(set) var session: ExpandableDiff?
    private(set) var generation: UInt64 = 0
    private(set) var isInFlight = false
    private(set) var pendingRequest: DiffContextExpansionRequest?
    private(set) var errorMessage: String?
    private(set) var revealFeedback: DiffContextRevealFeedback?
    /// Keep this monotonic across resets so representables never receive a reused selection token.
    private(set) var selectionResetGeneration: UInt64 = 0

    mutating func start(_ request: DiffContextExpansionRequest) -> (generation: UInt64, session: ExpandableDiff?)? {
        guard !isInFlight else {
            pendingRequest = request
            return nil
        }
        generation &+= 1
        isInFlight = true
        return (generation, session)
    }

    mutating func complete(
        session: ExpandableDiff,
        revealFeedback: DiffContextRevealFeedback?
    ) -> DiffContextExpansionRequest? {
        isInFlight = false
        self.session = session
        self.revealFeedback = revealFeedback
        errorMessage = nil
        selectionResetGeneration &+= 1
        defer { pendingRequest = nil }
        return pendingRequest
    }

    mutating func fail(message: String) {
        isInFlight = false
        pendingRequest = nil
        errorMessage = message
    }

    mutating func clearError() {
        errorMessage = nil
    }

    mutating func clearRevealFeedback(generation: UInt64) {
        guard revealFeedback?.generation == generation else { return }
        revealFeedback = nil
    }

    mutating func reset() {
        generation &+= 1
        session = nil
        isInFlight = false
        pendingRequest = nil
        errorMessage = nil
        revealFeedback = nil
    }
}
