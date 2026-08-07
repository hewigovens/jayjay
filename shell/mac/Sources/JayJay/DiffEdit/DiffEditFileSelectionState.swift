import Observation

@MainActor
@Observable
final class DiffEditFileSelectionState {
    private(set) var selectedChangedLines: Set<Int> = []
    @ObservationIgnored private(set) var hasLoadedSelection = false

    func replace(with lines: Set<Int>) {
        hasLoadedSelection = true
        guard selectedChangedLines != lines else { return }
        selectedChangedLines = lines
    }

    func reset() {
        hasLoadedSelection = false
        guard !selectedChangedLines.isEmpty else { return }
        selectedChangedLines = []
    }

    func toggle(_ lineNumber: Int) {
        var lines = selectedChangedLines
        if lines.contains(lineNumber) {
            lines.remove(lineNumber)
        } else {
            lines.insert(lineNumber)
        }
        replace(with: lines)
    }

    func formUnion(_ lines: Set<Int>) {
        replace(with: selectedChangedLines.union(lines))
    }
}
