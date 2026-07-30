import Observation

/// Transient bridge from window-level commands (palette) to the visible diff section; each request bumps a counter the section observes.
@Observable
final class DiffCommands {
    private(set) var expandAllContextRequest: UInt64 = 0

    func expandAllContext() {
        expandAllContextRequest &+= 1
    }
}
