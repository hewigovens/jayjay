import Foundation

/// Tracks which files have been reviewed.
/// Uses changeId + path as key, with file modification time to detect changes.
/// If a file's mtime hasn't changed since review, the review persists across refreshes.
@Observable
final class ReviewStore {
    private static let storageKey = "jayjay.reviewedFiles"

    /// Key: "changeId|path" → value: mtime (TimeInterval) when reviewed
    private var reviewed: [String: TimeInterval]
    private var repoPath: String = ""

    init() {
        if let data = UserDefaults.standard.data(forKey: Self.storageKey),
           let dict = try? JSONDecoder().decode([String: TimeInterval].self, from: data) {
            reviewed = dict
        } else {
            reviewed = [:]
        }
    }

    func setRepoPath(_ path: String) {
        repoPath = path
    }

    private func save() {
        if let data = try? JSONEncoder().encode(reviewed) {
            UserDefaults.standard.set(data, forKey: Self.storageKey)
        }
    }

    func isReviewed(changeId: String, path: String) -> Bool {
        let k = key(changeId: changeId, path: path)
        guard let reviewedMtime = reviewed[k] else { return false }
        let currentMtime = fileMtime(path)
        // If file was modified after review, invalidate
        if currentMtime > reviewedMtime + 1 {
            reviewed.removeValue(forKey: k)
            save()
            return false
        }
        return true
    }

    func markReviewed(changeId: String, path: String) {
        reviewed[key(changeId: changeId, path: path)] = fileMtime(path)
        save()
    }

    func markUnreviewed(changeId: String, path: String) {
        reviewed.removeValue(forKey: key(changeId: changeId, path: path))
        save()
    }

    func toggleReviewed(changeId: String, path: String) {
        if isReviewed(changeId: changeId, path: path) {
            markUnreviewed(changeId: changeId, path: path)
        } else {
            markReviewed(changeId: changeId, path: path)
        }
    }

    func reviewedPaths(changeId: String, allPaths: [String]) -> Set<String> {
        Set(allPaths.filter { isReviewed(changeId: changeId, path: $0) })
    }

    func clearAll() {
        reviewed.removeAll()
        save()
    }

    private func key(changeId: String, path: String) -> String {
        "\(changeId)|\(path)"
    }

    private func fileMtime(_ relativePath: String) -> TimeInterval {
        guard !repoPath.isEmpty else { return 0 }
        let fullPath = (repoPath as NSString).appendingPathComponent(relativePath)
        let attrs = try? FileManager.default.attributesOfItem(atPath: fullPath)
        return (attrs?[.modificationDate] as? Date)?.timeIntervalSince1970 ?? 0
    }
}
