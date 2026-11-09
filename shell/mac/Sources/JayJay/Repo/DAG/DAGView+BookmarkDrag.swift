import JayJayCore
import SwiftUI

extension DAGView {
    func handleBookmarkDragChanged(name: String, sourceCommitId: String, value: DragGesture.Value) {
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: name,
            drag: bookmarkDrag.state,
            location: value.location
        )
        switch action {
            case .ignore:
                break
            case .beginPress:
                beginBookmarkPress(name: name, sourceCommitId: sourceCommitId, location: value.location)
            case .beginDragging:
                beginBookmarkDraggingIfNeeded()
                updateBookmarkDrag(location: value.location)
            case .updateDragging:
                updateBookmarkDrag(location: value.location)
        }
    }

    func handleBookmarkDragEnded(name: String, value: DragGesture.Value) {
        switch BookmarkDragGesturePolicy.endAction(bookmarkName: name, drag: bookmarkDrag.state) {
            case .ignore:
                break
            case .cancel:
                bookmarkDrag.end()
            case .confirmDrop:
                updateBookmarkDrag(location: value.location)
                confirmBookmarkDrop()
        }
    }

    @ViewBuilder
    var bookmarkDragOverlay: some View {
        if let drag = bookmarkDrag.state, drag.phase == .dragging {
            // Anchor the ghost to the top-left of the pointer, independent of its size.
            Color.clear
                .frame(width: 0, height: 0)
                .overlay(alignment: .bottomTrailing) {
                    BookmarkDragGhost(label: drag.bookmarkName)
                }
                .offset(x: drag.location.x - 8, y: drag.location.y - 8)
                .allowsHitTesting(false)
        }
    }

    func bookmarkPreviewText(for change: ChangeInfo) -> String? {
        guard bookmarkDrag.previewTargetId == change.commitId.id, let drag = bookmarkDrag.state else { return nil }
        if drag.bookmarkName == workingCopyDragLabel {
            // jj refuses to edit an immutable change; say so instead of offering a drop that would do nothing.
            return change.isImmutable ? Localized.Repo.cantMoveHereImmutable : Localized.Repo.moveWorkingCopyHere
        }
        return Localized.Repo.moveBookmarkHere(bookmark: drag.bookmarkName)
    }

    private func beginBookmarkPress(name: String, sourceCommitId: String, location: CGPoint) {
        guard bookmarkDrag.state?.bookmarkName != name else { return }
        // The chip lives inside a row that also carries the rebase drag; claim it.
        rebaseDrag.end()
        activePane = .dag
        bookmarkDrag.armTask?.cancel()
        bookmarkDrag.state = BookmarkDragState(
            bookmarkName: name,
            sourceCommitId: sourceCommitId,
            isConflicted: conflictedBookmarkNames.contains(name),
            startLocation: location,
            armedAt: nil,
            phase: .pressing,
            location: location,
            hoveredCommitId: nil
        )
        scheduleBookmarkArm(name: name)
    }

    private func scheduleBookmarkArm(name: String) {
        bookmarkDrag.armTask = Task {
            try? await Task.sleep(for: .seconds(BookmarkDragGesturePolicy.armDuration))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard var drag = bookmarkDrag.state, drag.bookmarkName == name, drag.phase == .pressing
                else { return }
                drag.phase = .armed
                drag.armedAt = .now
                bookmarkDrag.state = drag
            }
        }
    }

    private func beginBookmarkDraggingIfNeeded() {
        guard var drag = bookmarkDrag.state, drag.phase != .dragging else { return }
        drag.phase = .dragging
        bookmarkDrag.state = drag
    }

    private func updateBookmarkDrag(location: CGPoint) {
        guard var drag = bookmarkDrag.state else { return }
        let hovered = rowFrameCache.frames.first(where: { $0.value.contains(location) })?.key
        let normalized = BookmarkDragGesturePolicy.normalizedHoveredCommitId(hovered, drag: drag)
        drag.location = location
        drag.hoveredCommitId = normalized
        bookmarkDrag.state = drag
        updateBookmarkPreviewTarget(normalized)
    }

    private func updateBookmarkPreviewTarget(_ commitId: String?) {
        if commitId == bookmarkDrag.previewTargetId {
            return
        }
        bookmarkDrag.clearPreview()
        guard let commitId else { return }
        bookmarkDrag.previewTask = Task {
            try? await Task.sleep(for: .milliseconds(BookmarkDragGesturePolicy.previewDelayMs))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard bookmarkDrag.state?.hoveredCommitId == commitId else { return }
                bookmarkDrag.previewTargetId = commitId
            }
        }
    }

    func confirmBookmarkDrop() {
        guard let request = BookmarkDragGesturePolicy.dropRequest(
            drag: bookmarkDrag.state,
            previewTargetCommitId: bookmarkDrag.previewTargetId,
            hoveredCommitId: bookmarkDrag.state?.hoveredCommitId,
            entries: entries
        ) else {
            bookmarkDrag.end()
            return
        }

        bookmarkDrag.end()
        if request.bookmarkName == workingCopyDragLabel {
            let target = entries.first(where: { $0.change.matchesRevision(request.destRev) })
            guard target?.change.isImmutable != true else { return }
            actions?.edit(rev: request.destRev)
        } else {
            actions?.moveBookmark(name: request.bookmarkName, toRev: request.destRev)
        }
    }
}

private struct BookmarkDragGhost: View {
    let label: String

    var body: some View {
        Group {
            if label == workingCopyDragLabel {
                Text("@").jayjayFont(12, weight: .bold).foregroundStyle(.tint)
            } else {
                Image(systemName: "bookmark").jayjayFont(12, weight: .semibold).foregroundStyle(.green)
            }
        }
        .frame(width: 18, height: 18)
        .padding(6)
        .glassEffect(in: Circle())
        .overlay(Circle().stroke(Color.primary.opacity(0.12), lineWidth: 1))
        .shadow(color: .black.opacity(0.12), radius: 6, y: 3)
    }
}
