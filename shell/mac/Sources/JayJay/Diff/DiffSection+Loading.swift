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
        guard !hunk.isSubmodulePlaceholder else {
            loadedDiff = DiffSectionLoadedDiff(
                path: hunk.path,
                fileDiff: nil,
                displayLines: nil,
                displayGroups: nil,
                content: DiffLoadedContent(
                    oldContent: hunk.oldContent,
                    newContent: hunk.newContent
                )
            )
            isComputing = false
            return
        }

        guard !hunk.isContentFreeRename else {
            loadedDiff = DiffSectionLoadedDiff(
                path: hunk.path,
                fileDiff: nil,
                displayLines: nil,
                displayGroups: nil,
                content: DiffLoadedContent()
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
            guard !Task.isCancelled, hunk.path == path else {
                isComputing = false
                return
            }
            apply(cached, path: path)
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
            guard !Task.isCancelled, hunk.path == path else {
                isComputing = false
                return
            }
            apply(cached, path: path)
        }
        isComputing = false
    }

    private func clearLoadedContent() {
        loadedDiff = nil
    }

    private func apply(_ cached: DiffStore.CachedDiff, path: String) {
        let lines = diffDisplayLines(lines: cached.diff.lines)
        loadedDiff = DiffSectionLoadedDiff(
            path: path,
            fileDiff: cached.diff,
            displayLines: lines,
            displayGroups: changeGroups(lines: lines),
            content: cached.content
        )
        refreshActiveNotes()
    }
}
