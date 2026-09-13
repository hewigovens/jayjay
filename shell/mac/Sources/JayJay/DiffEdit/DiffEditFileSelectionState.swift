import Observation

/// Per-card mirror of core's selection: one card's change invalidates only its own section.
@MainActor
@Observable
final class DiffEditFileSelectionState {
    private(set) var selectedChangedLines: Set<Int> = []

    func replace(with lines: Set<Int>) {
        guard selectedChangedLines != lines else { return }
        selectedChangedLines = lines
    }
}
