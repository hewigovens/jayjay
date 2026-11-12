import JayJayCore

struct Comparison: Equatable {
    /// The revision asked for; `fromId` is later pinned to the commit it resolved to.
    let fromRevision: String
    var fromId: String
    let toId: String
    let display: CompareDisplay?

    init(fromId: String, toId: String, display: CompareDisplay?) {
        fromRevision = fromId
        self.fromId = fromId
        self.toId = toId
        self.display = display
    }
}
