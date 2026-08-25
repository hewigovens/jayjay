@testable import JayJay
import JayJayCore
import XCTest

final class ReviewStoreTests: XCTestCase {
    private func tempStoreURL() -> URL {
        FileManager.default.temporaryDirectory
            .appendingPathComponent("review-\(UUID().uuidString)")
            .appendingPathComponent("review_store.json")
    }

    private func anchor(excerpt: String = "new line") -> NoteAnchor {
        NoteAnchor(
            changeId: "c1",
            path: "a.txt",
            identity: "idA",
            side: .new,
            line: 2,
            anchorExcerpt: excerpt,
            anchorContext: [excerpt],
            ignoreWhitespace: false
        )
    }

    /// Regression: two windows each held an init-time snapshot and overwrote the whole file on save, so the second window's save erased the first's marks; merge-on-save must keep both because the (changeId|path) keyspace cannot collide.
    func testConcurrentStoresDoNotClobberEachOther() {
        let url = tempStoreURL()

        let windowA = ReviewStore(storeURL: url)
        windowA.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")

        let windowB = ReviewStore(storeURL: url)
        windowB.markReviewed(changeId: "c1", path: "b.txt", identity: "idB")

        let reloaded = ReviewStore(storeURL: url)
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "b.txt", identity: "idB"))
    }

    func testRemovalMergesAgainstOnDiskState() {
        let url = tempStoreURL()
        let windowA = ReviewStore(storeURL: url)
        windowA.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")
        windowA.markReviewed(changeId: "c1", path: "b.txt", identity: "idA")

        let windowB = ReviewStore(storeURL: url)
        windowB.markUnreviewed(changeId: "c1", path: "a.txt")

        let reloaded = ReviewStore(storeURL: url)
        XCTAssertFalse(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "b.txt", identity: "idA"))
    }

    /// Persisted JSON must use the same tagged review entry as core so marks transfer between the SwiftUI and GPUI shells.
    func testPersistsCoreCompatibleHunkState() throws {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        store.setReviewedHunks(changeId: "c1", path: "a.txt", identity: "id", hunkIndices: [2, 0])

        let data = try Data(contentsOf: url)
        let root = try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: Any])
        let reviewed = try XCTUnwrap(root["reviewed"] as? [String: Any])
        let entry = try XCTUnwrap(reviewed["c1|a.txt"] as? [String: Any])
        XCTAssertEqual(entry["identity"] as? String, "id")
        let state = try XCTUnwrap(entry["state"] as? [String: Any])
        XCTAssertEqual(state["kind"] as? String, "hunks")
        XCTAssertEqual(state["indices"] as? [Int], [0, 2])
    }

    func testMarkingHunkPreservesNotesAndUnknownRootKeys() throws {
        let url = tempStoreURL()
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        let seeded: [String: Any] = [
            "reviewed": [:],
            "future": true,
            "notes": [[
                "id": "n1",
                "change_id": "c1",
                "path": "a.txt",
                "identity": "id",
                "side": "new",
                "line": 2,
                "anchor_excerpt": "new line",
                "anchor_context": ["new line"],
                "body": "Please check this",
                "created_at_ms": 1000,
                "updated_at_ms": 1000
            ]]
        ]
        let data = try JSONSerialization.data(withJSONObject: seeded)
        try data.write(to: url)

        let store = ReviewStore(storeURL: url)
        store.setReviewedHunks(changeId: "c1", path: "b.txt", identity: "idB", hunkIndices: [0])

        let root = try XCTUnwrap(try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any])
        XCTAssertEqual(root["future"] as? Bool, true)
        let notes = try XCTUnwrap(root["notes"] as? [[String: Any]])
        XCTAssertEqual(notes.first?["id"] as? String, "n1")
        let reviewed = try XCTUnwrap(root["reviewed"] as? [String: Any])
        XCTAssertNotNil(reviewed["c1|b.txt"])
    }

    func testClearChangeKeepsOtherChangesMarks() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        store.markReviewed(changeId: "committed", path: "a.txt", identity: "idA")
        store.markReviewed(changeId: "other", path: "a.txt", identity: "idA")

        store.clearChange(changeId: "committed")

        XCTAssertFalse(store.isReviewed(changeId: "committed", path: "a.txt", identity: "idA"))
        XCTAssertTrue(store.isReviewed(changeId: "other", path: "a.txt", identity: "idA"))
    }

    func testAddingAndResolvingNotesPreservesReviewedMarks() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        store.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")

        let note = store.addNote(anchor: anchor(), body: "Please check this")
        XCTAssertEqual(store.listNotes(changeId: "c1").count, 1)

        store.resolveNote(id: note.id)
        XCTAssertTrue(store.listNotes(changeId: "c1").isEmpty)
        XCTAssertEqual(store.listNotes(changeId: "c1", includeResolved: true).count, 1)

        let reloaded = ReviewStore(storeURL: url)
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertEqual(reloaded.listNotes(changeId: "c1", includeResolved: true).first?.id, note.id)
    }

    func testAddingMultipleNotesAtSameLineUpdatesExistingActiveNote() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)

        let first = store.addNote(anchor: anchor(), body: "First note")
        let second = store.addNote(anchor: anchor(excerpt: "newer line text"), body: "Second note")

        XCTAssertEqual(first.id, second.id)
        XCTAssertEqual(second.anchorExcerpt, "newer line text")
        XCTAssertEqual(store.listNotes(changeId: "c1").map(\.body), ["Second note"])

        store.resolveNote(id: first.id)
        let next = store.addNote(anchor: anchor(), body: "Next active note")

        XCTAssertNotEqual(first.id, next.id)
        XCTAssertEqual(store.listNotes(changeId: "c1").map(\.body), ["Next active note"])
        XCTAssertEqual(store.listNotes(changeId: "c1", includeResolved: true).count, 2)
    }

    func testMarkQueriesReflectMutationsThroughCache() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)

        // The first query deliberately primes the cache with a miss; the mutation must invalidate it or the follow-up query returns the stale cached miss.
        XCTAssertFalse(store.isHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0))
        store.toggleHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0)
        XCTAssertTrue(store.isHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0))
        XCTAssertEqual(
            store.reviewedPaths(changeId: "c1", files: [(path: "a.txt", identity: "id")]),
            []
        )

        store.markReviewed(changeId: "c1", path: "a.txt", identity: "id")
        XCTAssertEqual(
            store.reviewedPaths(changeId: "c1", files: [(path: "a.txt", identity: "id")]),
            ["a.txt"]
        )
    }

    func testSnapshotMarkingOneGroupDoesNotMarkSiblings() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        let old = "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n"
        let new = "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n"
        let snapshot = reviewCanonicalSnapshot(oldContent: old, newContent: new)
        XCTAssertEqual(snapshot.fingerprints.count, 2)

        store.markHunkReviewed(
            changeId: "c1",
            path: "a.txt",
            identity: "id",
            hunkIndex: 0,
            snapshot: snapshot
        )
        let states = store.displayHunkStates(
            changeId: "c1",
            query: ReviewDisplayQuery(path: "a.txt", identity: "id", snapshot: snapshot, mapping: [[0], [1]])
        )
        XCTAssertEqual(states, [.reviewed, .unreviewed])
        XCTAssertEqual(
            store.fileRollup(changeId: "c1", path: "a.txt", identity: "id"),
            .partial
        )
    }

    func testClearAllRemovesMarksAndNotes() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        store.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")
        store.addNote(anchor: anchor(), body: "check this")
        XCTAssertEqual(store.summary(), ReviewStoreSummary(marks: 1, notes: 1))

        store.clearAll()

        XCTAssertEqual(store.summary(), ReviewStoreSummary(marks: 0, notes: 0))
        XCTAssertTrue(store.notes.isEmpty)
        let reloaded = ReviewStore(storeURL: url)
        XCTAssertFalse(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertTrue(reloaded.listNotes(changeId: "c1", includeResolved: true).isEmpty)
    }

    func testMalformedStoreIsPreservedBeforeWrite() throws {
        let url = tempStoreURL()
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        let corruptText = "{\"reviewed\":"
        try Data(corruptText.utf8).write(to: url)

        let store = ReviewStore(storeURL: url)
        store.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")

        let root = try XCTUnwrap(try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any])
        XCTAssertNotNil((root["reviewed"] as? [String: Any])?["c1|a.txt"])

        let backups = try FileManager.default.contentsOfDirectory(
            at: url.deletingLastPathComponent(),
            includingPropertiesForKeys: nil
        )
        let backup = try XCTUnwrap(backups.first { $0.lastPathComponent.hasPrefix("review_store.json.corrupt") })
        XCTAssertEqual(try String(data: Data(contentsOf: backup), encoding: .utf8), corruptText)
    }
}
