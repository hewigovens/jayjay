import JayJayCore

struct BisectDAGState {
    let isEvaluating: Bool
    let candidateCommitId: String?
    let focusedCommitIds: Set<String>
    let marks: [String: BisectVerdict]

    init(_ state: BisectState) {
        isEvaluating = state.phase == .evaluating
        candidateCommitId = isEvaluating ? state.candidate?.commitId : nil
        focusedCommitIds = Set(state.focusedCommitIds)
        marks = state.phase == .finished ? [:] : Dictionary(state.marks.map { ($0.commitId, $0.verdict) }) { _, last in last }
    }
}
