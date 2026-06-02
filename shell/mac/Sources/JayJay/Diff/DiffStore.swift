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
        let oldPreview: DiffPreview?
        let newPreview: DiffPreview?
    }

    private let cache = DiffCache()
    @ObservationIgnored private var preloadTask: Task<Void, Never>?

    func clear() {
        Task { await cache.clear() }
    }

    /// Load a single file's diff. Returns cached if available, otherwise computes and caches.
    ///
    /// `commitId` is the immutable content hash used as the cache identity. `rev`
    /// (the mutable selection revision) is what jj resolves to fetch content, but
    /// keying on it would serve stale diffs after an amend/rebase reuses the id.
    func loadDiff(
        hunk: DiffHunk,
        rev: String?,
        commitId: String? = nil,
        repo: JayJayRepo?,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false
    ) async -> CachedDiff? {
        guard let repo else { return nil }
        guard !hunk.isSubmodulePlaceholder else { return nil }

        let key = Self.key(
            commitId: commitId, rev: rev, compareFromRev: compareFromRev,
            ignoreWhitespace: ignoreWhitespace, path: hunk.path
        )

        if let cached = await cache.get(key) {
            return cached
        }

        var old = hunk.oldContent ?? ""
        var new = hunk.newContent ?? ""
        var oldPreview = hunk.oldPreview
        var newPreview = hunk.newPreview

        if old.isEmpty, new.isEmpty {
            let loaded = await loadFileContent(
                repo: repo, path: hunk.path, rev: rev,
                fromRev: compareFromRev, hunkType: hunk.hunkType, oldPath: hunk.oldPath
            )
            old = loaded.oldContent
            new = loaded.newContent
            oldPreview = oldPreview ?? loaded.oldPreview
            newPreview = newPreview ?? loaded.newPreview
        }

        let path = hunk.path
        let diff = await Task.detached {
            repo.computeNativeDiff(
                path: path, oldContent: old, newContent: new,
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        let cached = CachedDiff(
            diff: diff, oldContent: old, newContent: new,
            oldPreview: oldPreview, newPreview: newPreview
        )
        await cache.set(key, value: cached)
        return cached
    }

    /// Preload all files in the background.
    func preload(
        hunks: [DiffHunk],
        rev: String?,
        commitId: String? = nil,
        repo: JayJayRepo?,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false
    ) {
        guard let repo, let rev else { return }
        // Cancel any in-flight preload so rapid commit navigation doesn't pile
        // up detached tasks all racing the FFI.
        preloadTask?.cancel()
        preloadTask = preloadHunks(
            hunks, rev: rev, commitId: commitId, repo: repo,
            compareFromRev: compareFromRev, ignoreWhitespace: ignoreWhitespace
        )
    }

    // MARK: - Private

    @discardableResult
    private func preloadHunks(
        _ hunks: [DiffHunk],
        rev: String,
        commitId: String?,
        repo: JayJayRepo,
        compareFromRev: String?,
        ignoreWhitespace: Bool
    ) -> Task<Void, Never> {
        Task.detached(priority: .utility) { [cache] in
            for hunk in hunks {
                if Task.isCancelled { return }
                guard !hunk.isSubmodulePlaceholder else { continue }
                let key = DiffStore.key(
                    commitId: commitId, rev: rev, compareFromRev: compareFromRev,
                    ignoreWhitespace: ignoreWhitespace, path: hunk.path
                )
                if await cache.get(key) != nil { continue }

                var old = hunk.oldContent ?? ""
                var new = hunk.newContent ?? ""
                var oldPreview = hunk.oldPreview
                var newPreview = hunk.newPreview
                if old.isEmpty, new.isEmpty {
                    if let compareFromRev,
                       let h = try? repo.interdiffFile(fromRev: compareFromRev, toRev: rev, path: hunk.path)
                    {
                        old = h.oldContent ?? ""
                        new = h.newContent ?? ""
                        oldPreview = oldPreview ?? h.oldPreview
                        newPreview = newPreview ?? h.newPreview
                    } else if let h = try? repo.showFile(rev: rev, path: hunk.path) {
                        old = h.oldContent ?? ""
                        new = h.newContent ?? ""
                        oldPreview = oldPreview ?? h.oldPreview
                        newPreview = newPreview ?? h.newPreview
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
                    diff: diff, oldContent: old, newContent: new,
                    oldPreview: oldPreview, newPreview: newPreview
                ))
            }
        }
    }

    private struct LoadedFileContent {
        var oldContent: String
        var newContent: String
        var oldPreview: DiffPreview?
        var newPreview: DiffPreview?
    }

    private func loadFileContent(
        repo: JayJayRepo, path: String, rev: String?,
        fromRev: String?, hunkType: HunkType, oldPath: String?
    ) async -> LoadedFileContent {
        if let fromRev, let rev {
            let h = await Task.detached {
                try? repo.interdiffFile(fromRev: fromRev, toRev: rev, path: path)
            }.value
            return LoadedFileContent(
                oldContent: h?.oldContent ?? "",
                newContent: h?.newContent ?? "",
                oldPreview: h?.oldPreview,
                newPreview: h?.newPreview
            )
        }
        if let rev, hunkType == .renamed, let oldPath {
            let h = await Task.detached {
                try? repo.showFileRename(rev: rev, oldPath: oldPath, newPath: path)
            }.value
            return LoadedFileContent(
                oldContent: h?.oldContent ?? "",
                newContent: h?.newContent ?? "",
                oldPreview: h?.oldPreview,
                newPreview: h?.newPreview
            )
        }
        if let rev {
            let h = await Task.detached { try? repo.showFile(rev: rev, path: path) }.value
            let old = h?.oldContent ?? ""
            let new = h?.newContent ?? ""
            if old.isEmpty, new.isEmpty {
                let content = await Task.detached {
                    try? repo.fileContent(rev: rev, path: path)
                }.value
                if let content, !content.isEmpty {
                    return LoadedFileContent(oldContent: "", newContent: content, oldPreview: nil, newPreview: nil)
                }
            }
            return LoadedFileContent(
                oldContent: old, newContent: new,
                oldPreview: h?.oldPreview, newPreview: h?.newPreview
            )
        }
        return LoadedFileContent(oldContent: "", newContent: "", oldPreview: nil, newPreview: nil)
    }

    /// Content-addressed cache key: immutable `commitId` (falling back to `rev`),
    /// the compare-from side, the whitespace mode, and the path. Whitespace is
    /// part of the key because it changes the computed diff for the same content.
    nonisolated static func key(
        commitId: String?,
        rev: String?,
        compareFromRev: String?,
        ignoreWhitespace: Bool,
        path: String
    ) -> String {
        let base = (commitId?.isEmpty == false ? commitId : rev) ?? ""
        let identity = compareFromRev.map { "\($0)→\(base)" } ?? base
        return "\(identity)|\(ignoreWhitespace ? "iw" : "")|\(path)"
    }
}

/// Thread-safe LRU diff cache bounded by the total bytes of cached file content.
actor DiffCache {
    private var entries: [String: DiffStore.CachedDiff] = [:]
    private var order: [String] = [] // LRU recency; front = least recently used
    private var totalBytes = 0
    private let budgetBytes: Int

    init(budgetBytes: Int = 64 * 1024 * 1024) {
        self.budgetBytes = budgetBytes
    }

    func get(_ key: String) -> DiffStore.CachedDiff? {
        guard let value = entries[key] else { return nil }
        touch(key)
        return value
    }

    func set(_ key: String, value: DiffStore.CachedDiff) {
        if let existing = entries[key] {
            totalBytes -= bytes(existing)
            order.removeAll { $0 == key }
        }
        entries[key] = value
        order.append(key)
        totalBytes += bytes(value)
        evict()
    }

    func clear() {
        entries.removeAll()
        order.removeAll()
        totalBytes = 0
    }

    private func touch(_ key: String) {
        order.removeAll { $0 == key }
        order.append(key)
    }

    /// Drop least-recently-used entries until under budget, always keeping the
    /// most recent one (so a single oversized file is still cached for its view).
    private func evict() {
        while totalBytes > budgetBytes, order.count > 1, let oldest = order.first {
            order.removeFirst()
            if let removed = entries.removeValue(forKey: oldest) {
                totalBytes -= bytes(removed)
            }
        }
    }

    private func bytes(_ diff: DiffStore.CachedDiff) -> Int {
        diff.oldContent.utf8.count + diff.newContent.utf8.count
    }
}
