import Foundation
import JayJayCore

/// Per-window review marks, persisted to jayjay-core's shared `review_store.json`
/// (`reviewStorePath()`) so marks transfer between the SwiftUI and GPUI shells.
/// Mutations read-modify-write the file at key level, so concurrent windows can't clobber each other.
@Observable
final class ReviewStore {
    private static let legacyStorageKey = "jayjay.reviewedFiles"

    private struct ReviewEntry {
        let identity: String
        let fileMarked: Bool
        let hunks: [UInt32]
    }

    private let storeURL: URL?
    private var reviewed: [String: ReviewEntry]

    init() {
        storeURL = reviewStorePath().map { URL(fileURLWithPath: $0) }
        reviewed = Self.loadInitial(from: storeURL)
    }

    /// Test seam: persist to an explicit file instead of the shared store path.
    init(storeURL: URL) {
        self.storeURL = storeURL
        reviewed = Self.read(from: storeURL)
    }

    // MARK: File-level review

    func isReviewed(changeId: String, path: String, identity: String) -> Bool {
        guard let entry = reviewed[key(changeId: changeId, path: path)] else { return false }
        return entry.fileMarked && entry.identity == identity
    }

    func markReviewed(changeId: String, path: String, identity: String) {
        guard !identity.isEmpty else { return }
        upsert(
            key(changeId: changeId, path: path),
            ReviewEntry(identity: identity, fileMarked: true, hunks: [])
        )
    }

    func markUnreviewed(changeId: String, path: String) {
        remove(key(changeId: changeId, path: path))
    }

    func toggleReviewed(changeId: String, path: String, identity: String) {
        if isReviewed(changeId: changeId, path: path, identity: identity) {
            markUnreviewed(changeId: changeId, path: path)
        } else {
            markReviewed(changeId: changeId, path: path, identity: identity)
        }
    }

    func reviewedPaths(changeId: String, files: [(path: String, identity: String)]) -> Set<String> {
        var out: Set<String> = []
        for f in files where isReviewed(changeId: changeId, path: f.path, identity: f.identity) {
            out.insert(f.path)
        }
        return out
    }

    // MARK: Hunk-level review

    /// file_marked implies all hunks reviewed; otherwise check the explicit set.
    func isHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) -> Bool {
        guard let entry = reviewed[key(changeId: changeId, path: path)] else { return false }
        guard entry.identity == identity else { return false }
        return entry.fileMarked || entry.hunks.contains(hunkIndex)
    }

    func markHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) {
        guard !identity.isEmpty else { return }
        let k = key(changeId: changeId, path: path)
        let existing = reviewed[k]
        if let existing, existing.identity == identity {
            var hunks = existing.hunks
            if !hunks.contains(hunkIndex) {
                hunks.append(hunkIndex)
                hunks.sort()
            }
            upsert(k, ReviewEntry(identity: identity, fileMarked: existing.fileMarked, hunks: hunks))
        } else {
            upsert(k, ReviewEntry(identity: identity, fileMarked: false, hunks: [hunkIndex]))
        }
    }

    func markHunkUnreviewed(changeId: String, path: String, hunkIndex: UInt32) {
        let k = key(changeId: changeId, path: path)
        guard let existing = reviewed[k] else { return }
        var hunks = existing.hunks
        hunks.removeAll(where: { $0 == hunkIndex })
        // Caller calls setReviewedHunks if they want the surviving hunks kept.
        if hunks.isEmpty {
            remove(k)
        } else {
            upsert(k, ReviewEntry(identity: existing.identity, fileMarked: false, hunks: hunks))
        }
    }

    func toggleHunkReviewed(changeId: String, path: String, identity: String, hunkIndex: UInt32) {
        if isHunkReviewed(changeId: changeId, path: path, identity: identity, hunkIndex: hunkIndex) {
            markHunkUnreviewed(changeId: changeId, path: path, hunkIndex: hunkIndex)
        } else {
            markHunkReviewed(changeId: changeId, path: path, identity: identity, hunkIndex: hunkIndex)
        }
    }

    func setReviewedHunks(changeId: String, path: String, identity: String, hunkIndices: [UInt32]) {
        guard !identity.isEmpty else { return }
        let k = key(changeId: changeId, path: path)
        if hunkIndices.isEmpty {
            remove(k)
        } else {
            let unique = Array(Set(hunkIndices)).sorted()
            upsert(k, ReviewEntry(identity: identity, fileMarked: false, hunks: unique))
        }
    }

    func clearAll() {
        reviewed.removeAll()
        persist { $0.removeAll() }
    }

    // MARK: Internals

    private func key(changeId: String, path: String) -> String {
        "\(changeId)|\(path)"
    }

    private func upsert(_ k: String, _ entry: ReviewEntry) {
        reviewed[k] = entry
        persist { $0[k] = entry }
    }

    private func remove(_ k: String) {
        reviewed.removeValue(forKey: k)
        persist { $0.removeValue(forKey: k) }
    }

    /// Read-modify-write the on-disk map so a concurrent window's marks survive.
    private func persist(_ mutate: (inout [String: ReviewEntry]) -> Void) {
        guard let storeURL else { return }
        var disk = Self.read(from: storeURL)
        mutate(&disk)
        Self.write(disk, to: storeURL)
    }

    // MARK: Persistence

    // JSON shape (mirrors jayjay-core's ReviewStore):
    //   `{"reviewed": {"changeId|path": {"identity": "<hex>", "file_marked": true, "hunks": [0,2]}}}`

    private static func loadInitial(from url: URL?) -> [String: ReviewEntry] {
        guard let url else {
            return importLegacyDefaults()
        }
        let disk = read(from: url)
        if !disk.isEmpty || FileManager.default.fileExists(atPath: url.path) {
            return disk
        }
        // First run on a shared store with no file yet: import any marks the old
        // UserDefaults-backed shell left behind, then drop the legacy blob.
        let legacy = importLegacyDefaults()
        if !legacy.isEmpty {
            write(legacy, to: url)
            UserDefaults.standard.removeObject(forKey: legacyStorageKey)
        }
        return legacy
    }

    private static func importLegacyDefaults() -> [String: ReviewEntry] {
        guard let data = UserDefaults.standard.data(forKey: legacyStorageKey),
              let raw = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else { return [:] }
        return decodeEntries(raw)
    }

    private static func read(from url: URL) -> [String: ReviewEntry] {
        guard let data = try? Data(contentsOf: url),
              let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let raw = root["reviewed"] as? [String: Any]
        else { return [:] }
        return decodeEntries(raw)
    }

    private static func decodeEntries(_ raw: [String: Any]) -> [String: ReviewEntry] {
        raw.compactMapValues { value in
            guard let dict = value as? [String: Any], let identity = dict["identity"] as? String else {
                return nil
            }
            let fileMarked = dict["file_marked"] as? Bool ?? false
            let hunks = (dict["hunks"] as? [Int])?.compactMap { $0 >= 0 ? UInt32($0) : nil } ?? []
            return ReviewEntry(identity: identity, fileMarked: fileMarked, hunks: hunks)
        }
    }

    private static func write(_ entries: [String: ReviewEntry], to url: URL) {
        let body = entries.mapValues { entry -> [String: Any] in
            var dict: [String: Any] = ["identity": entry.identity, "file_marked": entry.fileMarked]
            if !entry.hunks.isEmpty {
                dict["hunks"] = entry.hunks.map { Int($0) }
            }
            return dict
        }
        guard let data = try? JSONSerialization.data(withJSONObject: ["reviewed": body]) else { return }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        // Atomic write so a concurrent reader never sees a half-written file.
        try? data.write(to: url, options: .atomic)
    }
}
