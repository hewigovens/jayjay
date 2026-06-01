import JayJayCore

extension ChangeInfo {
    var selectionRevision: String {
        isDivergent ? commitId : changeId
    }

    func matchesRevision(_ rev: String) -> Bool {
        changeId == rev || commitId == rev
    }
}
