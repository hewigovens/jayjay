import JayJayCore

extension DiffStore {
    /// Throwing twin of the render path's load: the same fetch sequence, with primary-fetch errors propagated so destructive callers (Done's strict reload) can abort instead of seeing empty content.
    func loadContentStrict(
        hunk: DiffHunk,
        rev: String,
        repo: JayJayRepo
    ) async throws -> DiffLoadedContent {
        try await Self.fetchContent(
            repo: repo, hunk: hunk, rev: rev, fromRev: nil, projectionMode: nil
        )
    }

    func loadFileContent(
        repo: JayJayRepo,
        hunk: DiffHunk,
        rev: String?,
        fromRev: String?,
        projectionMode: DiffProjectionMode?
    ) async -> DiffLoadedContent {
        await (try? Self.fetchContent(
            repo: repo, hunk: hunk, rev: rev, fromRev: fromRev, projectionMode: projectionMode
        )) ?? DiffLoadedContent(oldContent: "", newContent: "")
    }

    private static func fetchContent(
        repo: JayJayRepo,
        hunk: DiffHunk,
        rev: String?,
        fromRev: String?,
        projectionMode: DiffProjectionMode?
    ) async throws -> DiffLoadedContent {
        let path = hunk.path
        let hunkType = hunk.hunkType
        let oldPath = hunk.oldPath
        let raw = effectiveProjectionMode(hunk: hunk, mode: projectionMode) == .raw
        if let fromRev, let rev {
            return try await loadedContent {
                raw
                    ? try repo.interdiffFileRaw(fromRev: fromRev, toRev: rev, path: path)
                    : try repo.interdiffFile(fromRev: fromRev, toRev: rev, path: path)
            }
        }
        if let rev, hunkType == .renamed, let oldPath {
            return try await loadedContent {
                raw
                    ? try repo.showFileRenameRaw(rev: rev, oldPath: oldPath, newPath: path)
                    : try repo.showFileRename(rev: rev, oldPath: oldPath, newPath: path)
            }
        }
        if let rev {
            let loaded = try await loadedContent {
                raw
                    ? try repo.showFileRaw(rev: rev, path: path)
                    : try repo.showFile(rev: rev, path: path)
            }
            if loaded.oldText.isEmpty, loaded.newText.isEmpty {
                // Best-effort enhancement, kept soft even for strict callers: image hunks land here with empty text sides, and a hard failure would drop their previews.
                let content = await Task.detached {
                    try? repo.fileContent(rev: rev, path: path)
                }.value
                if let content, !content.isEmpty {
                    return DiffLoadedContent(oldContent: nil, newContent: content)
                }
            }
            return loaded
        }
        return DiffLoadedContent(oldContent: "", newContent: "")
    }

    private static func loadedContent(
        loading: @escaping () throws -> DiffHunk
    ) async throws -> DiffLoadedContent {
        let hunk = try await Task.detached { try loading() }.value
        // Absent sides stay nil: core's staleness guard materializes them as None, and Some("") would abort the apply as stale.
        return DiffLoadedContent(
            oldContent: hunk.oldContent,
            newContent: hunk.newContent,
            oldPreview: hunk.oldPreview,
            newPreview: hunk.newPreview,
            projection: hunk.projection
        )
    }
}
