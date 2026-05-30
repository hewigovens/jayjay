import JayJayCore

struct RevsetEndpoint: Equatable {
    let rev: String
    let label: String
}

struct CompareDisplay: Equatable {
    let title: String
    let from: String
    let to: String
}

struct BookmarkDiffRequest: Equatable {
    let base: RevsetEndpoint
    let head: RevsetEndpoint

    var compareFromRev: String {
        RevsetExpressions.bookmarkDiffBase(base: base.rev, head: head.rev)
    }

    var display: CompareDisplay {
        CompareDisplay(title: "PR Diff", from: base.label, to: head.label)
    }
}

enum RevsetExpressions {
    static let trunk = RevsetEndpoint(rev: "trunk()", label: "trunk")

    static func bookmarkEndpoint(for bookmark: BookmarkInfo) -> RevsetEndpoint {
        if !bookmark.hasLocalTarget, let remote = bookmark.availableRemotes.first {
            return bookmarkEndpoint(name: "\(bookmark.name)@\(remote)")
        }
        return bookmarkEndpoint(name: bookmark.name)
    }

    static func bookmarkEndpoint(name: String) -> RevsetEndpoint {
        RevsetEndpoint(rev: quotedSymbol(name), label: name)
    }

    static func bookmarkDiffRequest(head: RevsetEndpoint, base: RevsetEndpoint = trunk) -> BookmarkDiffRequest {
        BookmarkDiffRequest(base: base, head: head)
    }

    static func bookmarkDiffBase(base: String, head: String) -> String {
        "fork_point(\(base) | \(head))"
    }

    static func compareDisplay(from: String, to: String, changes: [ChangeInfo]) -> CompareDisplay {
        CompareDisplay(
            title: "Comparing",
            from: displayLabel(for: from, changes: changes),
            to: displayLabel(for: to, changes: changes)
        )
    }

    static func quotedSymbol(_ symbol: String) -> String {
        let escaped = symbol
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        return "\"\(escaped)\""
    }

    static func primaryBaseBookmarkEndpoint(for change: ChangeInfo) -> RevsetEndpoint? {
        let name = change.bookmarks.first(where: isTrunkBookmark) ?? change.bookmarks.first
        return name.map(bookmarkEndpoint(name:))
    }

    static func primaryHeadBookmarkEndpoint(for change: ChangeInfo) -> RevsetEndpoint? {
        guard let name = change.bookmarks.first(where: { !isTrunkBookmark($0) }) else {
            return nil
        }
        return bookmarkEndpoint(name: name)
    }

    private static func displayLabel(for rev: String, changes: [ChangeInfo]) -> String {
        if let change = changes.first(where: { $0.changeId == rev || $0.commitId == rev }) {
            return displayLabel(for: change)
        }
        if rev.hasPrefix("\""), rev.hasSuffix("\"") {
            return String(rev.dropFirst().dropLast())
        }
        if rev.contains("(") || rev.contains(" ") {
            return rev
        }
        return String(rev.prefix(8))
    }

    private static func displayLabel(for change: ChangeInfo) -> String {
        if let bookmark = change.bookmarks.first, !bookmark.isEmpty {
            return bookmark
        }
        if change.isWorkingCopy {
            return "@"
        }
        return String(change.changeId.prefix(8))
    }
}
