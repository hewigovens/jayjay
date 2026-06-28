@testable import JayJay
import JayJayCore
import XCTest

final class DiffCacheTests: XCTestCase {
    // MARK: - Content-addressed key

    func testKeyUsesCommitIdOverRev() {
        // Same working-copy rev, different content hash (e.g. after an edit/amend) must produce different keys so a stale diff is never served.
        let a = DiffStore.key(commitId: "c1", rev: "@", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        let b = DiffStore.key(commitId: "c2", rev: "@", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        XCTAssertNotEqual(a, b)
    }

    func testKeyIgnoresRevWhenCommitIdPresent() {
        // A change keeps its content hash across selection-revision spellings.
        let a = DiffStore.key(commitId: "c1", rev: "abc", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        let b = DiffStore.key(commitId: "c1", rev: "xyz", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        XCTAssertEqual(a, b)
    }

    func testKeyFallsBackToRevWhenCommitIdMissing() {
        let a = DiffStore.key(commitId: nil, rev: "r1", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        let b = DiffStore.key(commitId: "", rev: "r1", compareFromRev: nil, ignoreWhitespace: false, path: "f")
        XCTAssertEqual(a, b, "empty commit id should fall back to rev")
    }

    func testKeyDistinguishesWhitespaceMode() {
        let on = DiffStore.key(commitId: "c1", rev: nil, compareFromRev: nil, ignoreWhitespace: true, path: "f")
        let off = DiffStore.key(commitId: "c1", rev: nil, compareFromRev: nil, ignoreWhitespace: false, path: "f")
        XCTAssertNotEqual(on, off)
    }

    func testKeyDistinguishesCompareSide() {
        let plain = DiffStore.key(commitId: "c1", rev: nil, compareFromRev: nil, ignoreWhitespace: false, path: "f")
        let compare = DiffStore.key(commitId: "c1", rev: nil, compareFromRev: "c0", ignoreWhitespace: false, path: "f")
        XCTAssertNotEqual(plain, compare)
    }

    // MARK: - LRU eviction

    func testEvictsLeastRecentlyUsedPastBudget() async {
        let cache = DiffCache(budgetBytes: 100)
        await cache.set("A", value: entry(bytes: 60))
        await cache.set("B", value: entry(bytes: 30)) // total 90, both fit

        let a = await cache.get("A") // touch A so B is now least recently used
        XCTAssertNotNil(a)

        await cache.set("C", value: entry(bytes: 30)) // total 120 > 100, evict LRU (B)

        let evicted = await cache.get("B")
        let keptA = await cache.get("A")
        let keptC = await cache.get("C")
        XCTAssertNil(evicted, "B was least recently used and should be evicted")
        XCTAssertNotNil(keptA)
        XCTAssertNotNil(keptC)
    }

    func testKeepsSingleEntryLargerThanBudget() async {
        let cache = DiffCache(budgetBytes: 100)
        await cache.set("big", value: entry(bytes: 500))
        let stored = await cache.get("big")
        XCTAssertNotNil(stored, "the only entry is kept even when it exceeds the budget")
    }

    // MARK: - Fixtures

    private func entry(bytes: Int) -> DiffStore.CachedDiff {
        DiffStore.CachedDiff(
            diff: FileDiff(path: "f", language: "", lines: [], whitespaceOnlyHidden: false),
            oldContent: "",
            newContent: String(repeating: "x", count: bytes),
            oldPreview: nil,
            newPreview: nil
        )
    }
}
