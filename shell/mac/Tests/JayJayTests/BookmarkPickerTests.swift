@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class BookmarkPickerTests: XCTestCase {
    func testRemoteRowsBrowseEachRemote() throws {
        let remote = remoteBookmark("odd&name", remotes: ["upstream", "origin"])
        var selected: [String] = []
        let picker = BookmarkPicker(bookmarks: [remote], actions: nil, onSelect: { selected.append($0) })
        let section = try XCTUnwrap(picker.sections.first)
        XCTAssertEqual(picker.sections.count, 1)
        XCTAssertEqual(section.title, "Remote Only")
        XCTAssertEqual(section.rows.map(\.searchText), ["odd&name@origin", "odd&name@upstream"])
        for row in section.rows {
            try XCTUnwrap(row.action)()
        }
        XCTAssertEqual(selected, [
            "ancestors(remote_bookmarks(exact:\"odd&name\", exact:\"origin\"), 20)",
            "ancestors(remote_bookmarks(exact:\"odd&name\", exact:\"upstream\"), 20)"
        ])
    }

    func testDeletedBookmarkKeepsItsUntrackedRemote() {
        let bookmark = remoteBookmark("feature", remotes: ["origin", "upstream"], tracked: ["origin"], deleted: true)
        let picker = BookmarkPicker(bookmarks: [bookmark], actions: nil, onSelect: { _ in })
        XCTAssertEqual(picker.sections.flatMap(\.rows).map(\.searchText), ["feature@upstream"])

        let fullyDeleted = remoteBookmark("feature", remotes: ["origin", "upstream"], tracked: ["origin", "upstream"], deleted: true)
        let deleted = BookmarkPicker(bookmarks: [fullyDeleted], actions: nil, onSelect: { _ in })
        XCTAssertTrue(deleted.sections.isEmpty)
    }

    func testRemoteRowIdentityDoesNotDependOnItsDisplayLabel() {
        let bookmarks = [remoteBookmark("a@b", remotes: ["c"]), remoteBookmark("a", remotes: ["b@c"])]
        var selected: [String] = []
        let picker = BookmarkPicker(bookmarks: bookmarks, actions: nil, onSelect: { selected.append($0) })
        let rows = picker.sections.flatMap(\.rows)
        XCTAssertEqual(rows.map(\.searchText), ["a@b@c", "a@b@c"])
        XCTAssertEqual(Set(rows.map(\.id)).count, 2)
        rows.forEach { $0.action?() }
        XCTAssertEqual(Set(selected).count, 2)
    }

    private func remoteBookmark(_ name: String, remotes: [String], tracked: [String] = [], deleted: Bool = false) -> BookmarkInfo {
        BookmarkInfo(
            name: name, changeId: ShortId(id: "abc", shortLen: 3), description: "",
            isTrackingRemote: false, isDeleted: deleted, isConflicted: false,
            trackedRemotes: tracked, availableRemotes: remotes, hasLocalTarget: false, remoteTargets: []
        )
    }
}
