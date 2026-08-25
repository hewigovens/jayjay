import JayJayCore

/// One loaded diff's per-group review lookup; the cache key is built once here so the gutter's per-line queries stay O(1).
struct ReviewDisplayQuery {
    let path: String
    let identity: String
    let snapshot: ReviewFileSnapshot
    let mapping: [[UInt32]]
    let cacheKey: String

    init(path: String, identity: String, snapshot: ReviewFileSnapshot, mapping: [[UInt32]]) {
        self.path = path
        self.identity = identity
        self.snapshot = snapshot
        self.mapping = mapping
        cacheKey = ([identity] + snapshot.fingerprints.map(\.digest)
            + mapping.map { $0.map(String.init).joined(separator: ",") })
            .joined(separator: "\n")
    }
}
