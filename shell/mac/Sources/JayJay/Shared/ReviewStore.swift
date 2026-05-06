import Foundation

@Observable
final class ReviewStore {
    private static let storageKey = "jayjay.reviewedFiles"

    private struct ReviewEntry {
        let identity: String
        let fileMarked: Bool
        let hunks: [UInt32]
    }

    private var reviewed: [String: ReviewEntry]

    init() {
        reviewed = UserDefaults.standard.data(forKey: Self.storageKey).map(Self.decode) ?? [:]
    }

    private func save() {
        if let data = Self.encode(reviewed) {
            UserDefaults.standard.set(data, forKey: Self.storageKey)
        }
    }

    // MARK: File-level review

    func isReviewed(changeId: String, path: String, identity: String) -> Bool {
        guard let entry = reviewed[key(changeId: changeId, path: path)] else { return false }
        return entry.fileMarked && entry.identity == identity
    }

    func markReviewed(changeId: String, path: String, identity: String) {
        guard !identity.isEmpty else { return }
        reviewed[key(changeId: changeId, path: path)] =
            ReviewEntry(identity: identity, fileMarked: true, hunks: [])
        save()
    }

    func markUnreviewed(changeId: String, path: String) {
        reviewed.removeValue(forKey: key(changeId: changeId, path: path))
        save()
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
        if let existing = reviewed[k], existing.identity == identity {
            var hunks = existing.hunks
            if !hunks.contains(hunkIndex) {
                hunks.append(hunkIndex)
                hunks.sort()
            }
            reviewed[k] = ReviewEntry(identity: identity, fileMarked: existing.fileMarked, hunks: hunks)
        } else {
            reviewed[k] = ReviewEntry(identity: identity, fileMarked: false, hunks: [hunkIndex])
        }
        save()
    }

    func markHunkUnreviewed(changeId: String, path: String, hunkIndex: UInt32) {
        let k = key(changeId: changeId, path: path)
        guard let existing = reviewed[k] else { return }
        var hunks = existing.hunks
        hunks.removeAll(where: { $0 == hunkIndex })
        // Caller calls setReviewedHunks if they want the surviving hunks kept.
        if hunks.isEmpty {
            reviewed.removeValue(forKey: k)
        } else {
            reviewed[k] = ReviewEntry(identity: existing.identity, fileMarked: false, hunks: hunks)
        }
        save()
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
            reviewed.removeValue(forKey: k)
        } else {
            let unique = Array(Set(hunkIndices)).sorted()
            reviewed[k] = ReviewEntry(identity: identity, fileMarked: false, hunks: unique)
        }
        save()
    }

    func clearAll() {
        reviewed.removeAll()
        save()
    }

    // MARK: Internals

    private func key(changeId: String, path: String) -> String {
        "\(changeId)|\(path)"
    }

    // MARK: Persistence

    // JSON shape (mirrors the Rust ReviewStore). Unrecognized entries are dropped on load.
    //   `{"changeId|path": {"identity": "<hex>", "file_marked": true, "hunks": [0,2]}}`

    private static func decode(_ data: Data) -> [String: ReviewEntry] {
        guard let raw = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return [:]
        }
        return raw.compactMapValues { value in
            guard let dict = value as? [String: Any], let identity = dict["identity"] as? String else {
                return nil
            }
            let fileMarked = dict["file_marked"] as? Bool ?? false
            let hunks = (dict["hunks"] as? [Int])?.compactMap { $0 >= 0 ? UInt32($0) : nil } ?? []
            return ReviewEntry(identity: identity, fileMarked: fileMarked, hunks: hunks)
        }
    }

    private static func encode(_ entries: [String: ReviewEntry]) -> Data? {
        let raw = entries.mapValues { entry -> [String: Any] in
            var dict: [String: Any] = ["identity": entry.identity, "file_marked": entry.fileMarked]
            if !entry.hunks.isEmpty {
                dict["hunks"] = entry.hunks.map { Int($0) }
            }
            return dict
        }
        return try? JSONSerialization.data(withJSONObject: raw)
    }
}
