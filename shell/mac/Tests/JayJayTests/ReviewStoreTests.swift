@testable import JayJay
import XCTest

final class ReviewStoreTests: XCTestCase {
    private func tempStoreURL() -> URL {
        FileManager.default.temporaryDirectory
            .appendingPathComponent("review-\(UUID().uuidString)")
            .appendingPathComponent("review_store.json")
    }

    /// The bug: two windows each held an init-time snapshot and overwrote the
    /// whole file on save, so the second window's save erased the first's marks.
    /// Merge-on-save must keep both windows' marks because the keyspace
    /// (changeId|path) cannot collide.
    func testConcurrentStoresDoNotClobberEachOther() {
        let url = tempStoreURL()

        // Window A opens (empty file) and marks a file.
        let windowA = ReviewStore(storeURL: url)
        windowA.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")

        // Window B opened before A's mark — its init snapshot is empty.
        // (Construct after A's mark so B's cache is the loaded-from-disk state.)
        let windowB = ReviewStore(storeURL: url)
        windowB.markReviewed(changeId: "c1", path: "b.txt", identity: "idB")

        // A reload sees both marks — neither window erased the other's.
        let reloaded = ReviewStore(storeURL: url)
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "b.txt", identity: "idB"))
    }

    func testRemovalMergesAgainstOnDiskState() {
        let url = tempStoreURL()
        let windowA = ReviewStore(storeURL: url)
        windowA.markReviewed(changeId: "c1", path: "a.txt", identity: "idA")
        windowA.markReviewed(changeId: "c1", path: "b.txt", identity: "idA")

        // A second window unmarks one path; the other must survive.
        let windowB = ReviewStore(storeURL: url)
        windowB.markUnreviewed(changeId: "c1", path: "a.txt")

        let reloaded = ReviewStore(storeURL: url)
        XCTAssertFalse(reloaded.isReviewed(changeId: "c1", path: "a.txt", identity: "idA"))
        XCTAssertTrue(reloaded.isReviewed(changeId: "c1", path: "b.txt", identity: "idA"))
    }

    /// Persisted JSON must use the same {"reviewed": {...}} envelope as core so
    /// marks transfer between the SwiftUI and GPUI shells.
    func testPersistsCoreCompatibleEnvelopeAndHunks() throws {
        let url = tempStoreURL()
        let store = ReviewStore(storeURL: url)
        store.setReviewedHunks(changeId: "c1", path: "a.txt", identity: "id", hunkIndices: [2, 0])

        let data = try Data(contentsOf: url)
        let root = try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: Any])
        let reviewed = try XCTUnwrap(root["reviewed"] as? [String: Any])
        let entry = try XCTUnwrap(reviewed["c1|a.txt"] as? [String: Any])
        XCTAssertEqual(entry["identity"] as? String, "id")
        XCTAssertEqual(entry["file_marked"] as? Bool, false)
        XCTAssertEqual(entry["hunks"] as? [Int], [0, 2])
    }
}
