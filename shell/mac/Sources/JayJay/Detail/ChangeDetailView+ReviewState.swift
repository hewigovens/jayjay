import JayJayCore
import SwiftUI

extension ChangeDetailView {
    /// One entry point for every mutation site; forgetting a member of the trio is how stale badges and banners happen.
    func refreshReviewState() {
        // External writers (other windows, GPUI, the CLI) can't invalidate this process's caches; refresh is the boundary where we re-read the shared store.
        reviewStore.invalidateMarksCache()
        refreshReviewedPaths()
        refreshNoteCounts()
        refreshReviewNotes()
    }

    func refreshReviewedPaths() {
        applyKnownReviewRollups()
        fillMissingReviewSnapshots()
    }

    func rememberLoadedReviewSnapshot(path: String, snapshot: ReviewFileSnapshot?) {
        if let snapshot, !snapshot.fingerprints.isEmpty {
            reviewSnapshots[path] = snapshot
        } else {
            reviewSnapshots.removeValue(forKey: path)
        }
        applyKnownReviewRollups()
    }

    func applyKnownReviewRollups() {
        let files = visibleDiff.map { hunk in
            (path: hunk.path, identity: hunk.reviewIdentity, snapshot: reviewSnapshots[hunk.path])
        }
        fileRollups = reviewStore.fileRollups(changeId: reviewChangeId, files: files)
        reviewedPaths = Set(fileRollups.compactMap { path, rollup in
            rollup == .reviewed ? path : nil
        })
    }

    func fillMissingReviewSnapshots() {
        let missing = visibleDiff.filter { hunk in
            reviewSnapshots[hunk.path] == nil
                && fileRollups[hunk.path] == .changedSinceReview
                && !hunk.reviewIdentity.isEmpty
                && !hunk.isSubmodulePlaceholder
        }
        guard !missing.isEmpty else { return }
        reviewSnapshotRequestId &+= 1
        let requestId = reviewSnapshotRequestId
        let changeId = reviewChangeId
        let requests = missing.map(reviewSnapshotLoad(for:))
        Task.detached {
            var loaded: [(String, ReviewFileSnapshot)] = []
            for request in requests {
                guard let snapshot = await request.load(), !snapshot.fingerprints.isEmpty else {
                    continue
                }
                loaded.append((request.path, snapshot))
            }
            await MainActor.run {
                guard reviewSnapshotRequestId == requestId, reviewChangeId == changeId else { return }
                for (path, snapshot) in loaded {
                    reviewSnapshots[path] = snapshot
                }
                applyKnownReviewRollups()
            }
        }
    }

    func reviewSnapshotLoad(for hunk: DiffHunk) -> ReviewSnapshotLoad {
        ReviewSnapshotLoad(
            hunk: hunk,
            repo: repo,
            rev: detailRevision,
            diffStore: diffStore,
            commitId: detail.info.commitId.id,
            compareFromRev: compareFromId,
            ignoreWhitespace: appSettings.ignoreWhitespace
        )
    }

    var activeReviewNoteCount: Int {
        activeNoteCountsByPath.values.reduce(0, +)
    }

    func refreshNoteCounts() {
        guard showsReviewControls else {
            activeNoteCountsByPath = [:]
            showNotedFilesOnly = false
            return
        }
        // Reconciled `.current` notes only: stale/orphaned notes cannot show a marker (their identity no longer matches the diff), so counting them would advertise notes the file rows can't display — the stale-notes banner is their surface.
        let visiblePaths = Set(visibleDiff.map(\.path))
        activeNoteCountsByPath = Dictionary(
            grouping: reviewNoteStatuses.filter {
                $0.status == .current && visiblePaths.contains($0.note.path)
            },
            by: \.note.path
        )
        .mapValues(\.count)
        // Resolving the last note hides the badge, so drop the filter with it or the list would pin to empty with no control left to clear it.
        if activeNoteCountsByPath.isEmpty {
            showNotedFilesOnly = false
        }
    }

    var staleOrOrphanedReviewNotes: [ReviewNoteStatus] {
        reviewNoteStatuses.filter { item in
            item.status == .stale || item.status == .orphaned
        }
    }

    var staleReviewNoteIds: Set<String> {
        Set(staleOrOrphanedReviewNotes.map(\.note.id))
    }

    func refreshReviewNotes() {
        // @State token, not a captured copy: comparing captured detail fields against themselves is always true and lets a slower superseded refresh overwrite a newer one. Bump on every path so an in-flight refresh can't overwrite the cleared state either.
        reviewNotesRequestId &+= 1
        guard showsReviewControls, let repo else {
            reviewNoteStatuses = []
            return
        }
        let rev = detailRevision
        let requestId = reviewNotesRequestId
        Task.detached {
            // Keep the last known statuses on failure; clearing would silently hide the stale-notes banner.
            guard let statuses = try? repo.reviewNotes(rev: rev, includeResolved: false) else { return }
            await MainActor.run {
                guard reviewNotesRequestId == requestId else { return }
                reviewNoteStatuses = statuses
                // Counts derive from these statuses; recompute now that the reconciled report replaced the snapshot refreshNoteCounts saw.
                refreshNoteCounts()
            }
        }
    }
}

struct ReviewSnapshotLoad {
    let hunk: DiffHunk
    let repo: JayJayRepo?
    let rev: String
    let diffStore: DiffStore
    let commitId: String
    let compareFromRev: String?
    let ignoreWhitespace: Bool

    var path: String { hunk.path }

    func load() async -> ReviewFileSnapshot? {
        let inline = reviewSnapshotFromDiffHunk(hunk: hunk)
        if !inline.fingerprints.isEmpty {
            return inline
        }
        if let cached = await diffStore.cachedDiff(
            hunk: hunk,
            rev: rev,
            commitId: commitId,
            compareFromRev: compareFromRev,
            ignoreWhitespace: ignoreWhitespace
        ) {
            let snapshot = reviewCanonicalSnapshot(
                oldContent: cached.oldContent,
                newContent: cached.newContent
            )
            if !snapshot.fingerprints.isEmpty {
                return snapshot
            }
        }
        return try? repo?.reviewFileSnapshot(rev: rev, path: hunk.path, oldPath: hunk.oldPath)
    }
}
