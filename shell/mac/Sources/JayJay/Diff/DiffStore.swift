import JayJayCore
import Observation

@Observable
final class DiffStore {
    struct CachedDiff {
        let diff: FileDiff
        let content: DiffLoadedContent

        var oldContent: String { content.oldText }
        var newContent: String { content.newText }
        var oldPreview: DiffPreview? { content.oldPreview }
        var newPreview: DiffPreview? { content.newPreview }
        var projection: DiffProjection? { content.projection }
    }

    struct CacheKeyParts {
        let commitId: String?
        let rev: String?
        let compareFromRev: String?
        let ignoreWhitespace: Bool
        let path: String
        let projectionKey: String
    }

    private let cache = DiffCache()
    @ObservationIgnored private var preloadTask: Task<Void, Never>?

    func clear() {
        Task { await cache.clear() }
    }

    func cachedDiff(
        hunk: DiffHunk,
        rev: String?,
        commitId: String? = nil,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false,
        projectionMode: DiffProjectionMode? = nil
    ) async -> CachedDiff? {
        guard !hunk.isSubmodulePlaceholder else { return nil }
        guard !hunk.isContentFreeRename else { return nil }
        let key = Self.key(CacheKeyParts(
            commitId: commitId, rev: rev, compareFromRev: compareFromRev,
            ignoreWhitespace: ignoreWhitespace, path: hunk.path,
            projectionKey: Self.projectionKey(hunk: hunk, mode: projectionMode)
        ))
        return await cache.get(key)
    }

    /// `commitId` is the immutable content hash used as the cache identity; `rev` (the mutable selection revision) is what jj resolves to fetch content, but keying on it would serve stale diffs after an amend/rebase reuses the id.
    func loadDiff(
        hunk: DiffHunk,
        rev: String?,
        commitId: String? = nil,
        repo: JayJayRepo?,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false,
        projectionMode: DiffProjectionMode? = nil
    ) async -> CachedDiff? {
        guard let repo else { return nil }
        guard !hunk.isSubmodulePlaceholder else { return nil }
        // A byte-identical rename has no content to load; loading by the new path alone would render every line as added.
        guard !hunk.isContentFreeRename else { return nil }

        let key = Self.key(CacheKeyParts(
            commitId: commitId, rev: rev, compareFromRev: compareFromRev,
            ignoreWhitespace: ignoreWhitespace, path: hunk.path,
            projectionKey: Self.projectionKey(hunk: hunk, mode: projectionMode)
        ))

        if let cached = await cache.get(key) {
            return cached
        }

        var content = DiffLoadedContent(
            oldContent: hunk.oldContent ?? "",
            newContent: hunk.newContent ?? "",
            oldPreview: hunk.oldPreview,
            newPreview: hunk.newPreview,
            projection: hunk.projection
        )

        let needsProjectionModeReload = hunk.projection != nil
            && projectionMode != nil
            && hunk.projection?.mode != projectionMode
        if Self.shouldLoadFileContent(
            oldContent: content.oldText,
            newContent: content.newText,
            projectionModeChanged: needsProjectionModeReload
        ) {
            let loaded = await loadFileContent(
                repo: repo, hunk: hunk, rev: rev, fromRev: compareFromRev,
                projectionMode: projectionMode
            )
            content = DiffLoadedContent(
                oldContent: loaded.oldContent,
                newContent: loaded.newContent,
                oldPreview: content.oldPreview ?? loaded.oldPreview,
                newPreview: content.newPreview ?? loaded.newPreview,
                projection: loaded.projection
            )
        }

        let path = content.projection?.virtualPath ?? hunk.path
        let diff = await Task.detached {
            repo.computeNativeDiff(
                path: path, oldContent: content.oldText, newContent: content.newText,
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        let cached = CachedDiff(
            diff: diff,
            content: content
        )
        await cache.set(key, value: cached)
        return cached
    }

    func preload(
        hunks: [DiffHunk],
        rev: String?,
        commitId: String? = nil,
        repo: JayJayRepo?,
        compareFromRev: String? = nil,
        ignoreWhitespace: Bool = false,
        projectionMode: DiffProjectionMode? = nil
    ) {
        guard let repo, let rev else { return }
        // Cancel any in-flight preload so rapid commit navigation doesn't pile up detached tasks all racing the FFI.
        preloadTask?.cancel()
        preloadTask = Task.detached(priority: .utility) { [weak self] in
            for hunk in hunks {
                if Task.isCancelled { return }
                _ = await self?.loadDiff(
                    hunk: hunk, rev: rev, commitId: commitId, repo: repo,
                    compareFromRev: compareFromRev, ignoreWhitespace: ignoreWhitespace,
                    projectionMode: projectionMode
                )
            }
        }
    }

    // MARK: - Private

    private func loadFileContent(
        repo: JayJayRepo,
        hunk: DiffHunk,
        rev: String?,
        fromRev: String?,
        projectionMode: DiffProjectionMode?
    ) async -> DiffLoadedContent {
        let path = hunk.path
        let hunkType = hunk.hunkType
        let oldPath = hunk.oldPath
        let raw = Self.effectiveProjectionMode(hunk: hunk, mode: projectionMode) == .raw
        if let fromRev, let rev {
            return await loadRepositoryHunk(raw: raw) {
                try repo.interdiffFileRaw(fromRev: fromRev, toRev: rev, path: path)
            } processed: {
                try repo.interdiffFile(fromRev: fromRev, toRev: rev, path: path)
            }
        }
        if let rev, hunkType == .renamed, let oldPath {
            return await loadRepositoryHunk(raw: raw) {
                try repo.showFileRenameRaw(rev: rev, oldPath: oldPath, newPath: path)
            } processed: {
                try repo.showFileRename(rev: rev, oldPath: oldPath, newPath: path)
            }
        }
        if let rev {
            let loaded = await loadRepositoryHunk(raw: raw) {
                try repo.showFileRaw(rev: rev, path: path)
            } processed: {
                try repo.showFile(rev: rev, path: path)
            }
            if loaded.oldText.isEmpty, loaded.newText.isEmpty {
                let content = await Task.detached {
                    try? repo.fileContent(rev: rev, path: path)
                }.value
                if let content, !content.isEmpty {
                    return DiffLoadedContent(
                        oldContent: "",
                        newContent: content
                    )
                }
            }
            return loaded
        }
        return DiffLoadedContent(oldContent: "", newContent: "")
    }

    private func loadRepositoryHunk(
        raw: Bool,
        raw rawLoad: @escaping () throws -> DiffHunk,
        processed processedLoad: @escaping () throws -> DiffHunk
    ) async -> DiffLoadedContent {
        let hunk = await Task.detached {
            raw ? (try? rawLoad()) : (try? processedLoad())
        }.value
        return Self.loadedContent(from: hunk)
    }

    private static func loadedContent(from hunk: DiffHunk?) -> DiffLoadedContent {
        DiffLoadedContent(
            oldContent: hunk?.oldContent ?? "",
            newContent: hunk?.newContent ?? "",
            oldPreview: hunk?.oldPreview,
            newPreview: hunk?.newPreview,
            projection: hunk?.projection
        )
    }

    nonisolated static func key(_ parts: CacheKeyParts) -> String {
        let base = (parts.commitId?.isEmpty == false ? parts.commitId : parts.rev) ?? ""
        let identity = parts.compareFromRev.map { "\($0)→\(base)" } ?? base
        return "\(identity)|\(parts.ignoreWhitespace ? "iw" : "")|\(parts.projectionKey)|\(parts.path)"
    }

    nonisolated static func shouldLoadFileContent(
        oldContent: String,
        newContent: String,
        projectionModeChanged: Bool
    ) -> Bool {
        (oldContent.isEmpty && newContent.isEmpty) || projectionModeChanged
    }

    nonisolated static func effectiveProjectionMode(
        hunk: DiffHunk,
        mode: DiffProjectionMode?
    ) -> DiffProjectionMode? {
        mode ?? hunk.projection?.mode
    }

    nonisolated private static func projectionKey(
        hunk: DiffHunk,
        mode: DiffProjectionMode?
    ) -> String {
        guard let projection = hunk.projection else { return "raw" }
        let activeMode = effectiveProjectionMode(hunk: hunk, mode: mode) ?? projection.mode
        return projection.identityPart(mode: activeMode)
    }
}

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

    /// Drop least-recently-used entries until under budget, always keeping the most recent one (so a single oversized file is still cached for its view).
    private func evict() {
        while totalBytes > budgetBytes, order.count > 1, let oldest = order.first {
            order.removeFirst()
            if let removed = entries.removeValue(forKey: oldest) {
                totalBytes -= bytes(removed)
            }
        }
    }

    private func bytes(_ diff: DiffStore.CachedDiff) -> Int {
        diff.content.oldText.utf8.count + diff.content.newText.utf8.count
    }
}
