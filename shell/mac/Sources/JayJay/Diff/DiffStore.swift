import JayJayCore
import Observation

/// Manages diff computation, caching, and preloading.
/// Views read computed diffs from the store instead of computing them inline.
@Observable
final class DiffStore {
    struct CachedDiff {
        let diff: FileDiff
        let oldContent: String
        let newContent: String
    }

    private let cache = DiffCache()

    func get(rev: String?, path: String) async -> CachedDiff? {
        await cache.get(Self.key(rev: rev, path: path))
    }

    func set(rev: String?, path: String, value: CachedDiff) async {
        await cache.set(Self.key(rev: rev, path: path), value: value)
    }

    func clear() {
        Task { await cache.clear() }
    }

    /// Load a single file's diff. Returns cached if available, otherwise computes and caches.
    func loadDiff(
        hunk: DiffHunk,
        rev: String?,
        repo: JayJayRepo?,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false
    ) async -> CachedDiff? {
        guard let repo else { return nil }
        guard !hunk.isSubmodulePlaceholder else { return nil }

        let cacheRev = compareFromRev != nil ? "\(compareFromRev!)→\(rev ?? "")" : rev
        let key = Self.key(rev: cacheRev, path: hunk.path)

        if let cached = await cache.get(key) {
            return cached
        }

        var old = hunk.oldContent ?? ""
        var new = hunk.newContent ?? ""

        if old.isEmpty, new.isEmpty {
            let loaded = await loadFileContent(
                repo: repo, path: hunk.path, rev: rev,
                fromRev: compareFromRev, hunkType: hunk.hunkType, oldPath: hunk.oldPath
            )
            old = loaded.0
            new = loaded.1
        }

        let path = hunk.path
        let diff = await Task.detached {
            repo.computeNativeDiff(
                path: path, oldContent: old, newContent: new,
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        let cached = CachedDiff(diff: diff, oldContent: old, newContent: new)
        await cache.set(key, value: cached)
        return cached
    }

    /// Preload all files in the background.
    func preload(
        hunks: [DiffHunk],
        rev: String?,
        repo: JayJayRepo?,
        ignoreWhitespace: Bool = false
    ) {
        guard let repo, let rev else { return }
        preloadHunks(hunks, rev: rev, repo: repo, ignoreWhitespace: ignoreWhitespace)
    }

    // MARK: - Private

    private func preloadHunks(
        _ hunks: [DiffHunk],
        rev: String,
        repo: JayJayRepo,
        ignoreWhitespace: Bool
    ) {
        Task.detached(priority: .utility) { [cache] in
            for hunk in hunks {
                guard !hunk.isSubmodulePlaceholder else { continue }
                let key = DiffStore.key(rev: rev, path: hunk.path)
                if await cache.get(key) != nil { continue }

                var old = hunk.oldContent ?? ""
                var new = hunk.newContent ?? ""
                if old.isEmpty, new.isEmpty {
                    if let h = try? repo.showFile(rev: rev, path: hunk.path) {
                        old = h.oldContent ?? ""
                        new = h.newContent ?? ""
                    }
                    if old.isEmpty, new.isEmpty {
                        if let content = try? repo.fileContent(rev: rev, path: hunk.path),
                           !content.isEmpty
                        {
                            new = content
                        }
                    }
                }

                let path = hunk.path
                let diff = repo.computeNativeDiff(
                    path: path, oldContent: old, newContent: new,
                    ignoreWhitespace: ignoreWhitespace
                )
                await cache.set(key, value: DiffStore.CachedDiff(
                    diff: diff, oldContent: old, newContent: new
                ))
            }
        }
    }

    private func loadFileContent(
        repo: JayJayRepo, path: String, rev: String?,
        fromRev: String?, hunkType: HunkType, oldPath: String?
    ) async -> (String, String) {
        if let fromRev, let rev {
            let h = await Task.detached {
                try? repo.interdiffFile(fromRev: fromRev, toRev: rev, path: path)
            }.value
            return (h?.oldContent ?? "", h?.newContent ?? "")
        }
        if let rev, hunkType == .renamed, let oldPath {
            let h = await Task.detached {
                try? repo.showFileRename(rev: rev, oldPath: oldPath, newPath: path)
            }.value
            return (h?.oldContent ?? "", h?.newContent ?? "")
        }
        if let rev {
            let h = await Task.detached { try? repo.showFile(rev: rev, path: path) }.value
            let old = h?.oldContent ?? ""
            let new = h?.newContent ?? ""
            if old.isEmpty, new.isEmpty {
                let content = await Task.detached {
                    try? repo.fileContent(rev: rev, path: path)
                }.value
                if let content, !content.isEmpty { return ("", content) }
            }
            return (old, new)
        }
        return ("", "")
    }

    nonisolated private static func key(rev: String?, path: String) -> String {
        "\(rev ?? "")|\(path)"
    }
}

/// Thread-safe diff cache actor.
actor DiffCache {
    private var entries: [String: DiffStore.CachedDiff] = [:]

    func get(_ key: String) -> DiffStore.CachedDiff? {
        entries[key]
    }

    func set(_ key: String, value: DiffStore.CachedDiff) {
        entries[key] = value
    }

    func clear() {
        entries.removeAll()
    }
}
