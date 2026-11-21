import Foundation

public struct ReviewFileSummary: Decodable, Sendable, Identifiable, Equatable {
    public var path: String
    public var oldPath: String?
    public var kind: FileKind
    public var added: Int
    public var removed: Int
    public var review: ReviewState
    public var noteCount: Int
    public var notice: String?

    public var id: String {
        path
    }

    public var name: String {
        (path as NSString).lastPathComponent
    }

    public var folder: String {
        (path as NSString).deletingLastPathComponent
    }

    public var sourcePath: String? {
        kind == .renamed || kind == .copied ? oldPath : nil
    }

    public init(
        path: String, oldPath: String? = nil, kind: FileKind = .modified, added: Int, removed: Int,
        review: ReviewState = .unreviewed, noteCount: Int = 0, notice: String? = nil
    ) {
        self.path = path
        self.oldPath = oldPath
        self.kind = kind
        self.added = added
        self.removed = removed
        self.review = review
        self.noteCount = noteCount
        self.notice = notice
    }

    private enum CodingKeys: String, CodingKey { case path, oldPath, kind, added, removed, review, noteCount, notice }

    public init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        path = try c.decode(String.self, forKey: .path)
        oldPath = try c.decodeIfPresent(String.self, forKey: .oldPath)
        // A backend that predates `kind` reads as modified.
        kind = try c.decodeIfPresent(String.self, forKey: .kind).flatMap(FileKind.init(rawValue:)) ?? .modified
        added = try c.decode(Int.self, forKey: .added)
        removed = try c.decode(Int.self, forKey: .removed)
        review = try c.decode(ReviewState.self, forKey: .review)
        noteCount = try c.decodeIfPresent(Int.self, forKey: .noteCount) ?? 0
        notice = try c.decodeIfPresent(String.self, forKey: .notice)
    }
}
