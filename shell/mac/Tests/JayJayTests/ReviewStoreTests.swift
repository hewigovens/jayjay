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

    func testMarkQueriesReflectMutationsThroughCache() {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)

        // The first query deliberately primes the cache with a miss; the mutation must invalidate it or the follow-up query returns the stale cached miss.
        XCTAssertFalse(store.isHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0))
        store.toggleHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0)
        XCTAssertTrue(store.isHunkReviewed(changeId: "c1", path: "a.txt", identity: "id", hunkIndex: 0))
        XCTAssertFalse(store.isReviewed(changeId: "c1", path: "a.txt", identity: "id"))

        store.markReviewed(changeId: "c1", path: "a.txt", identity: "id")
        XCTAssertTrue(store.isReviewed(changeId: "c1", path: "a.txt", identity: "id"))
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
}
