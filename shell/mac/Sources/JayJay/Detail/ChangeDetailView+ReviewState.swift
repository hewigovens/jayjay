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
        let files = visibleDiff.map { (path: $0.path, identity: $0.reviewIdentity) }
        fileRollups = reviewStore.fileRollups(changeId: reviewChangeId, files: files)
        reviewedPaths = Set(fileRollups.compactMap { path, rollup in
            rollup == .reviewed ? path : nil
        })
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
