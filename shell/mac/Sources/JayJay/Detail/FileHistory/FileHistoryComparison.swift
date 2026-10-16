import JayJayCore

struct FileHistoryEndpoint {
    let commitId: ShortId
    let changeId: ShortId
}

struct FileHistoryComparison {
    private(set) var base: FileHistoryEndpoint?
    private(set) var target: FileHistoryEndpoint?

    var isPinned: Bool {
        base != nil
    }

    var isSameRevision: Bool {
        guard let base, let target else { return false }
        return base.commitId.id == target.commitId.id
    }

    var compareFrom: String? {
        base?.commitId.id
    }

    var compareTo: String? {
        target?.commitId.id
    }

    mutating func select(_ endpoint: FileHistoryEndpoint) {
        target = endpoint
    }

    mutating func pin(_ endpoint: FileHistoryEndpoint) {
        target = endpoint
        base = endpoint
    }

    mutating func swap() {
        guard let base, let target else { return }
        (self.base, self.target) = (target, base)
    }

    mutating func clear() {
        base = nil
    }
}
