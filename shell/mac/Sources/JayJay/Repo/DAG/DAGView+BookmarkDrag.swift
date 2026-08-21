import JayJayCore
import SwiftUI

extension DAGView {
    func handleBookmarkDragChanged(name: String, sourceCommitId: String, value: DragGesture.Value) {
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: name,
            drag: bookmarkDrag,
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
        switch BookmarkDragGesturePolicy.endAction(bookmarkName: name, drag: bookmarkDrag) {
            case .ignore:
                break
            case .cancel:
                cancelBookmarkDrag()
            case .confirmDrop:
                updateBookmarkDrag(location: value.location)
                confirmBookmarkDrop()
        }
    }

    @ViewBuilder
    var bookmarkDragOverlay: some View {
        if let bookmarkDrag, bookmarkDrag.phase == .dragging {
            // Anchor the ghost to the top-left of the pointer, independent of its size.
            Color.clear
                .frame(width: 0, height: 0)
                .overlay(alignment: .bottomTrailing) {
                    BookmarkDragGhost(label: bookmarkDrag.bookmarkName)
                }
                .offset(x: bookmarkDrag.location.x - 8, y: bookmarkDrag.location.y - 8)
                .allowsHitTesting(false)
        }
    }

    func bookmarkPreviewText(for change: ChangeInfo) -> String? {
        guard bookmarkPreviewTargetId == change.commitId.id, let bookmarkDrag else { return nil }
        if bookmarkDrag.bookmarkName == workingCopyDragLabel {
            // jj refuses to edit an immutable change; say so instead of offering a drop that would do nothing.
            return change.isImmutable ? "Can't move @ here (immutable)" : "Move working copy here?"
        }
        return "Move \(bookmarkDrag.bookmarkName) here?"
    }

    private func beginBookmarkPress(name: String, sourceCommitId: String, location: CGPoint) {
        guard bookmarkDrag?.bookmarkName != name else { return }
        // The chip lives inside a row that also carries the rebase drag; claim it.
        cancelRebaseDrag()
        activePane = .dag
        bookmarkArmTask?.cancel()
        bookmarkDrag = BookmarkDragState(
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
        bookmarkArmTask = Task {
            try? await Task.sleep(for: .seconds(BookmarkDragGesturePolicy.armDuration))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard var drag = bookmarkDrag, drag.bookmarkName == name, drag.phase == .pressing
                else { return }
                drag.phase = .armed
                drag.armedAt = .now
                bookmarkDrag = drag
            }
        }
    }

    private func beginBookmarkDraggingIfNeeded() {
        guard var drag = bookmarkDrag, drag.phase != .dragging else { return }
        drag.phase = .dragging
        bookmarkDrag = drag
    }

    private func updateBookmarkDrag(location: CGPoint) {
        guard var drag = bookmarkDrag else { return }
        let hovered = rebaseRowFrames.first(where: { $0.value.contains(location) })?.key
        let normalized = BookmarkDragGesturePolicy.normalizedHoveredCommitId(hovered, drag: drag)
        drag.location = location
        drag.hoveredCommitId = normalized
        bookmarkDrag = drag
        updateBookmarkPreviewTarget(normalized)
    }

    private func updateBookmarkPreviewTarget(_ commitId: String?) {
        if commitId == bookmarkPreviewTargetId {
            return
        }
        bookmarkPreviewTask?.cancel()
        bookmarkPreviewTask = nil
        bookmarkPreviewTargetId = nil
        guard let commitId else { return }
        bookmarkPreviewTask = Task {
            try? await Task.sleep(for: .milliseconds(BookmarkDragGesturePolicy.previewDelayMs))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard bookmarkDrag?.hoveredCommitId == commitId else { return }
                bookmarkPreviewTargetId = commitId
            }
        }
    }

    func confirmBookmarkDrop() {
        guard let request = BookmarkDragGesturePolicy.dropRequest(
            drag: bookmarkDrag,
            previewTargetCommitId: bookmarkPreviewTargetId,
            hoveredCommitId: bookmarkDrag?.hoveredCommitId,
            entries: entries
        ) else {
            cancelBookmarkDrag()
            return
        }

        cancelBookmarkDrag()
        if request.bookmarkName == workingCopyDragLabel {
            let target = entries.first(where: { $0.change.matchesRevision(request.destRev) })
            guard target?.change.isImmutable != true else { return }
            onMoveWorkingCopyToRev?(request.destRev)
        } else {
            onMoveBookmarkToRev?(request.bookmarkName, request.destRev)
        }
    }

    func cancelBookmarkDrag() {
        bookmarkArmTask?.cancel()
        bookmarkArmTask = nil
        bookmarkPreviewTask?.cancel()
        bookmarkPreviewTask = nil
        bookmarkPreviewTargetId = nil
        bookmarkDrag = nil
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
