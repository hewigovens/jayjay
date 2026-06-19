import JayJayCore

extension ChangeInfo {
    var selectionRevision: String {
        isDivergent ? commitId.id : changeId.id
    }

    func matchesRevision(_ rev: String) -> Bool {
        changeId.id == rev || commitId.id == rev
    }
}
