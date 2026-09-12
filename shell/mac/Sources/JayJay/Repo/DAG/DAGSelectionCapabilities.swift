import JayJayCore

/// A class so a body pass compares one reference instead of a per-row array.
final class DAGSelectionCapabilities {
    static let empty = DAGSelectionCapabilities()

    private let state: SelectionState
    private let rowByCommitId: [String: Int]

    private init() {
        state = SelectionState(
            canAbandon: false,
            canSquash: false,
            canDiff: false,
            canMerge: false,
            canRebaseOnto: [],
            canMergeWith: []
        )
        rowByCommitId = [:]
    }

    init(graph: DagSelectionGraph, entries: [GraphEntry], selectedCommitIds: [String]) {
        state = graph.selectionState(selectedCommitIds: selectedCommitIds)
        rowByCommitId = Dictionary(
            entries.enumerated().map { ($0.element.change.commitId.id, $0.offset) },
            uniquingKeysWith: { first, _ in first }
        )
    }

    var canAbandon: Bool {
        state.canAbandon
    }

    var canSquash: Bool {
        state.canSquash
    }

    var canMerge: Bool {
        state.canMerge
    }

    func canRebase(onto change: ChangeInfo) -> Bool {
        row(of: change).map { state.canRebaseOnto[$0] } ?? false
    }

    func canMerge(with change: ChangeInfo) -> Bool {
        row(of: change).map { state.canMergeWith[$0] } ?? false
    }

    private func row(of change: ChangeInfo) -> Int? {
        rowByCommitId[change.commitId.id]
    }
}
