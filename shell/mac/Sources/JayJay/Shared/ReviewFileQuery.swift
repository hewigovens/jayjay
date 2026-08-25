import JayJayCore

struct ReviewFileQuery {
    let path: String
    let identity: String
    let snapshot: ReviewFileSnapshot?

    init(path: String, identity: String, snapshot: ReviewFileSnapshot? = nil) {
        self.path = path
        self.identity = identity
        self.snapshot = snapshot
    }
}
