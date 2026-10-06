@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class BookmarkPickerTests: XCTestCase {
    func testRemoteRowsBrowseEachRemote() throws {
        let remote = remoteBookmark("odd&name", remotes: ["upstream", "origin"])
        let actions = RevealRecorder()
        let picker = BookmarkPicker(bookmarks: [remote], isLoaded: true, actions: actions)
        let section = try XCTUnwrap(picker.sections.first)
        XCTAssertEqual(picker.sections.count, 1)
        XCTAssertEqual(section.title, "Remote Only")
        XCTAssertEqual(section.rows.map(\.searchText), ["odd&name@origin", "odd&name@upstream"])
        for row in section.rows {
            try XCTUnwrap(row.action)()
        }
        XCTAssertEqual(actions.revealed.map { revsetFilter(revset: $0.revset).label }, ["odd&name@origin", "odd&name@upstream"])
    }

    func testDeletedBookmarkKeepsItsUntrackedRemote() {
        let bookmark = remoteBookmark("feature", remotes: ["origin", "upstream"], tracked: ["origin"], deleted: true)
        let picker = BookmarkPicker(bookmarks: [bookmark], isLoaded: true, actions: nil)
        XCTAssertEqual(picker.sections.flatMap(\.rows).map(\.searchText), ["feature@upstream"])

        let fullyDeleted = remoteBookmark("feature", remotes: ["origin", "upstream"], tracked: ["origin", "upstream"], deleted: true)
        let deleted = BookmarkPicker(bookmarks: [fullyDeleted], isLoaded: true, actions: nil)
        XCTAssertTrue(deleted.sections.isEmpty)
    }

    func testRemoteRowIdentityDoesNotDependOnItsDisplayLabel() {
        let bookmarks = [remoteBookmark("a@b", remotes: ["c"]), remoteBookmark("a", remotes: ["b@c"])]
        let actions = RevealRecorder()
        let picker = BookmarkPicker(bookmarks: bookmarks, isLoaded: true, actions: actions)
        let rows = picker.sections.flatMap(\.rows)
        XCTAssertEqual(rows.map(\.searchText), ["a@b@c", "a@b@c"])
        XCTAssertEqual(Set(rows.map(\.id)).count, 2)
        rows.forEach { $0.action?() }
        XCTAssertEqual(Set(actions.revealed.map(\.revset)).count, 2)
    }

    private func remoteBookmark(_ name: String, remotes: [String], tracked: [String] = [], deleted: Bool = false) -> BookmarkInfo {
        BookmarkInfo(
            name: name, changeId: ShortId(id: "abc", shortLen: 3), description: "",
            isTrackingRemote: false, isDeleted: deleted, isConflicted: false,
            trackedRemotes: tracked, availableRemotes: remotes, hasLocalTarget: false, remoteTargets: []
        )
    }
}

private final class RevealRecorder: BookmarkActions {
    var revealed: [BookmarkFilterTarget] = []

    func revealBookmark(_ target: BookmarkFilterTarget) {
        revealed.append(target)
    }

    func filterByBookmark(_: BookmarkFilterTarget) {}
    func createBookmark(name _: String, rev _: String) {}
    func deleteBookmark(name _: String) {}
    func removeBookmark(name _: String, fromRev _: String) {}
    func forgetBookmark(name _: String) {}
    func moveBookmarkForward(name _: String) {}
    func moveBookmark(name _: String, toRev _: String) {}
    func renameBookmark(oldName _: String, newName _: String) {}
    func trackBookmark(name _: String, remote _: String) {}
    func gitPush(bookmark _: String) {}
    func gitFetch() {}
    func gitPullBookmark(name _: String) {}
    func openPR(bookmark _: String) {}
}
