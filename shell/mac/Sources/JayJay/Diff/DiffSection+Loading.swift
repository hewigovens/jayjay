import JayJayCore
import SwiftUI

extension DiffSection {
    var shouldShowBlockingProgress: Bool {
        Self.shouldShowBlockingProgress(
            isComputing: isComputing,
            hasCurrentDiff: hasCurrentRenderableDiff
        )
    }

    var hasCurrentRenderableDiff: Bool {
        Self.hasCurrentRenderableDiff(
            loadedPath: loadedPath,
            hunkPath: hunk.path,
            hasRenderedDiff: fileDiff != nil,
            loadedProjectionMode: loadedProjection?.mode,
            requestedProjectionMode: projectionRequestMode
        )
    }

    nonisolated static func shouldKeepLoadedContentWhileLoading(
        loadedPath: String?,
        hunkPath: String,
        hasRenderedDiff: Bool,
        loadedProjectionMode: DiffProjectionMode?,
        requestedProjectionMode: DiffProjectionMode?
    ) -> Bool {
        hasCurrentRenderableDiff(
            loadedPath: loadedPath,
            hunkPath: hunkPath,
            hasRenderedDiff: hasRenderedDiff,
            loadedProjectionMode: loadedProjectionMode,
            requestedProjectionMode: requestedProjectionMode
        )
    }

    nonisolated static func hasCurrentRenderableDiff(
        loadedPath: String?,
        hunkPath: String,
        hasRenderedDiff: Bool,
        loadedProjectionMode: DiffProjectionMode?,
        requestedProjectionMode: DiffProjectionMode?
    ) -> Bool {
        loadedPath == hunkPath
            && hasRenderedDiff
            && loadedProjectionMode == requestedProjectionMode
    }

    nonisolated static func shouldShowBlockingProgress(
        isComputing: Bool,
        hasCurrentDiff: Bool
    ) -> Bool {
        isComputing && !hasCurrentDiff
    }

    func computeDiffAsync() async {
        // Captured once at compute start so the identity describes exactly the basis this diff is computed under, not the controls at some later click.
        let identity = DiffContextExpansionIdentity(
            compareFromRev: compareFromRev,
            commitId: commitId,
            rev: rev,
            path: hunk.path,
            ignoreWhitespace: settings.ignoreWhitespace,
            projectionMode: projectionModeKey
        )
        if let content = placeholderContent {
            resetContextExpansion()
            loadedDiff = DiffSectionLoadedDiff(
                path: hunk.path,
                fileDiff: nil,
                displayLines: nil,
                displayGroups: nil,
                content: content,
                identity: nil
            )
            isComputing = false
            return
        }

        let path = hunk.path
        let requestedProjectionMode = projectionRequestMode
        if let cached = await diffStore.cachedDiff(
            hunk: hunk, rev: rev, commitId: commitId,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace,
            projectionMode: requestedProjectionMode
        ) {
            await applyLoaded(cached, path: path, identity: identity)
            isComputing = false
            return
        }

        isComputing = true
        if !Self.shouldKeepLoadedContentWhileLoading(
            loadedPath: loadedPath,
            hunkPath: path,
            hasRenderedDiff: fileDiff != nil,
            loadedProjectionMode: loadedProjection?.mode,
            requestedProjectionMode: requestedProjectionMode
        ) {
            clearLoadedContent()
        }

        if let cached = await diffStore.loadDiff(
            hunk: hunk, rev: rev, commitId: commitId, repo: repo,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace,
            projectionMode: requestedProjectionMode
        ) {
            await applyLoaded(cached, path: path, identity: identity)
        }
        isComputing = false
    }

    private var placeholderContent: DiffLoadedContent? {
        if hunk.isSubmodulePlaceholder {
            return DiffLoadedContent(oldContent: hunk.oldContent, newContent: hunk.newContent)
        }
        if hunk.isContentFreeRename {
            return DiffLoadedContent()
        }
        return nil
    }

    private func applyLoaded(
        _ cached: DiffStore.CachedDiff,
        path: String,
        identity: DiffContextExpansionIdentity
    ) async {
        let prepared = await Self.prepareLoadedDiff(
            cached,
            path: path,
            identity: identity,
            hunk: hunk,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        guard !Task.isCancelled, hunk.path == path else { return }
        apply(prepared)
    }

    private func clearLoadedContent() {
        resetContextExpansion()
        loadedDiff = nil
    }

    nonisolated private static func prepareLoadedDiff(
        _ cached: DiffStore.CachedDiff,
        path: String,
        identity: DiffContextExpansionIdentity,
        hunk: DiffHunk,
        ignoreWhitespace: Bool
    ) async -> DiffSectionLoadedDiff {
        await Task.detached {
            let lines = diffDisplayLines(lines: cached.diff.lines)
            return DiffSectionLoadedDiff(
                path: path,
                fileDiff: cached.diff,
                displayLines: lines,
                displayGroups: changeGroups(lines: lines),
                content: cached.content,
                identity: identity
            )
            .withReviewFingerprints(hunk: hunk, ignoreWhitespace: ignoreWhitespace)
        }.value
    }

    private func apply(_ prepared: DiffSectionLoadedDiff) {
        resetContextExpansion()
        loadedDiff = prepared
        onReviewSnapshotLoaded?(hunk, loadedDiff?.reviewQuery?.snapshot)
        refreshActiveNotes()
    }

    func resetContextExpansion() {
        contextExpansion.reset()
    }
}
