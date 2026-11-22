/// The review store's own shape (`jayjay_primitives::ReviewNoteStatus`), snake_case inside.
public struct ReviewNote: Decodable, Sendable, Identifiable, Equatable {
    public enum Status: String, Decodable, Sendable { case current, stale, orphaned, resolved }

    public struct Entry: Decodable, Sendable, Equatable {
        public var id: String
        public var path: String
        public var side: NoteSide
        public var line: Int
        public var body: String
        public var resolved: Bool
        public var excerpt: String

        public init(id: String, path: String, side: NoteSide, line: Int, body: String, resolved: Bool = false, excerpt: String = "") {
            self.id = id
            self.path = path
            self.side = side
            self.line = line
            self.body = body
            self.resolved = resolved
            self.excerpt = excerpt
        }

        private enum CodingKeys: String, CodingKey { case id, path, side, line, body, resolved, excerpt = "anchor_excerpt" }
    }

    public var note: Entry
    public var status: Status
    public var id: String {
        note.id
    }

    public var stale: Bool {
        status == .stale || status == .orphaned
    }

    public init(note: Entry, status: Status) {
        self.note = note
        self.status = status
    }
}
