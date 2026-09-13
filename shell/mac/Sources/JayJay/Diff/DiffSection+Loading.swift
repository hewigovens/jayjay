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
        // Captured at compute start so the key names the basis this diff was computed under, not the controls at a later click.
        let basis = [
            compareFromRev ?? "",
            commitId ?? "",
            rev ?? "",
            hunk.path,
            String(settings.ignoreWhitespace),
            projectionModeKey
        ].joined(separator: "|")
        if let content = placeholderContent {
            resetContextExpansion()
            loadedDiff = DiffSectionLoadedDiff(
                path: hunk.path,
                fileDiff: nil,
                displayLines: nil,
                displayGroups: nil,
                content: content,
                basis: nil
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
            await applyLoaded(cached, path: path, basis: basis)
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
            await applyLoaded(cached, path: path, basis: basis)
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
        basis: String
    ) async {
        let prepared = await Self.prepareLoadedDiff(
            cached,
            path: path,
            basis: basis,
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
        basis: String,
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
                basis: basis
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
        contextExpansionDisplay.reset()
    }
}
