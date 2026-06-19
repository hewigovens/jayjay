import AppKit
import JayJayCore
import SwiftUI

enum DAGRebaseCoordinateSpace {
    static let name = "dag-rebase"
}

struct DAGRebaseRowFramePreferenceKey: PreferenceKey {
    static let defaultValue: [String: CGRect] = [:]

    static func reduce(value: inout [String: CGRect], nextValue: () -> [String: CGRect]) {
        value.merge(nextValue(), uniquingKeysWith: { _, next in next })
    }
}

extension DAGView {
    func rebaseGesture(for entry: GraphEntry, layout: DAGLayout) -> some Gesture {
        DragGesture(minimumDistance: 0, coordinateSpace: .named(DAGRebaseCoordinateSpace.name))
            .onChanged { value in
                handleRebaseGestureChanged(entry: entry, layout: layout, value: value)
            }
            .onEnded { value in
                handleRebaseGestureEnded(entry: entry, layout: layout, value: value)
            }
    }

    @ViewBuilder
    var rebaseDragOverlay: some View {
        if let rebaseDrag, rebaseDrag.phase == .dragging {
            DAGRebaseGhost(label: rebaseDrag.sourceLabel)
                .position(x: rebaseDrag.location.x + 68, y: rebaseDrag.location.y - 18)
                .allowsHitTesting(false)
        }
    }

    func rebasePreviewText(for change: ChangeInfo) -> String? {
        guard rebasePreviewTargetId == change.commitId,
              let rebaseDrag
        else { return nil }
        return "Rebase \(rebaseDrag.sourceLabel) onto \(DAGRebaseGesturePolicy.displayLabel(for: change))?"
    }

    private func handleRebaseGestureChanged(
        entry: GraphEntry,
        layout: DAGLayout,
        value: DragGesture.Value
    ) {
        // A bookmark-chip drag (started on a child view) wins over the row rebase.
        guard bookmarkDrag == nil else { return }
        let action = DAGRebaseGesturePolicy.changeAction(
            entryIsImmutable: entry.change.isImmutable,
            sourceCommitId: entry.change.commitId,
            rebaseDrag: rebaseDrag,
            location: value.location
        )

        switch action {
            case .ignore:
                break
            case .beginPress:
                beginRebasePress(for: entry, layout: layout, location: value.location)
            case .cancelPress:
                cancelRebaseDrag()
            case .beginDragging:
                beginDraggingIfNeeded()
                updateRebaseDrag(location: value.location)
            case .updateDragging:
                updateRebaseDrag(location: value.location)
        }
    }

    private func handleRebaseGestureEnded(
        entry: GraphEntry,
        layout: DAGLayout,
        value: DragGesture.Value
    ) {
        guard bookmarkDrag == nil else { return }
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: entry.change.isImmutable,
            sourceCommitId: entry.change.commitId,
            rebaseDrag: rebaseDrag,
            startLocation: value.startLocation,
            location: value.location
        )

        switch action {
            case .ignore:
                break
            case .select:
                cancelRebaseDrag()
                selectEntry(entry)
            case .cancel:
                cancelRebaseDrag()
            case .confirmDrop:
                updateRebaseDrag(location: value.location)
                confirmRebaseDrop()
        }
    }

    private func beginRebasePress(for entry: GraphEntry, layout: DAGLayout, location: CGPoint) {
        guard rebaseDrag?.sourceCommitId != entry.change.commitId,
              let seedLocation = rebaseDragSeedLocation(for: entry, layout: layout)
        else { return }

        activePane = .dag
        rebaseArmTask?.cancel()
        rebaseDrag = DAGRebaseDragState(
            sourceCommitId: entry.change.commitId,
            sourceChangeId: entry.change.changeId,
            sourceRev: DAGRebaseGesturePolicy.revision(for: entry.change),
            sourceLabel: DAGRebaseGesturePolicy.displayLabel(for: entry.change),
            startLocation: location,
            armedAt: nil,
            phase: .pressing,
            location: seedLocation,
            hoveredCommitId: nil
        )
        scheduleRebaseArm(for: entry)
    }

    private func scheduleRebaseArm(for entry: GraphEntry) {
        let sourceCommitId = entry.change.commitId
        rebaseArmTask = Task {
            try? await Task.sleep(for: .seconds(DAGRebaseGesturePolicy.armDuration))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard var rebaseDrag,
                      rebaseDrag.sourceCommitId == sourceCommitId,
                      rebaseDrag.phase == .pressing
                else { return }
                rebaseDrag.phase = .armed
                rebaseDrag.armedAt = .now
                self.rebaseDrag = rebaseDrag
            }
        }
    }

    private func beginDraggingIfNeeded() {
        guard var rebaseDrag, rebaseDrag.phase != .dragging else { return }
        rebaseDrag.phase = .dragging
        self.rebaseDrag = rebaseDrag
    }

    private func updateRebaseDrag(location: CGPoint) {
        guard var rebaseDrag else { return }
        let hoveredCommitId = rebaseRowFrames.first(where: { $0.value.contains(location) })?.key
        let normalizedTarget = DAGRebaseGesturePolicy.normalizedTargetCommitId(
            sourceCommitId: rebaseDrag.sourceCommitId,
            hoveredCommitId: hoveredCommitId
        )
        rebaseDrag.location = location
        rebaseDrag.hoveredCommitId = normalizedTarget
        self.rebaseDrag = rebaseDrag
        updateRebasePreviewTarget(normalizedTarget)
    }

    private func updateRebasePreviewTarget(_ commitId: String?) {
        if commitId == rebasePreviewTargetId {
            return
        }

        rebasePreviewTask?.cancel()
        rebasePreviewTask = nil
        rebasePreviewTargetId = nil

        guard let commitId else { return }

        rebasePreviewTask = Task {
            try? await Task.sleep(for: .milliseconds(DAGRebaseGesturePolicy.previewDelayMs))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard rebaseDrag?.hoveredCommitId == commitId else { return }
                rebasePreviewTargetId = commitId
            }
        }
    }

    func confirmRebaseDrop() {
        guard let request = DAGRebaseGesturePolicy.dropRequest(
            rebaseDrag: rebaseDrag,
            previewTargetCommitId: rebasePreviewTargetId,
            hoveredCommitId: rebaseDrag?.hoveredCommitId,
            entries: entries
        ) else {
            cancelRebaseDrag()
            return
        }

        cancelRebaseDrag()

        if let onRequestRebase {
            onRequestRebase(request)
        } else {
            actions?.rebase(rev: request.sourceRev, dest: request.destRev)
        }
    }

    func cancelRebaseDrag() {
        rebaseArmTask?.cancel()
        rebaseArmTask = nil
        rebasePreviewTask?.cancel()
        rebasePreviewTask = nil
        rebasePreviewTargetId = nil
        rebaseDrag = nil
    }

    private func selectEntry(_ entry: GraphEntry) {
        activePane = .dag
        NSApp.keyWindow?.makeFirstResponder(nil)
        let rev = entry.change.selectionRevision
        if NSEvent.modifierFlags.contains(.shift),
           let sel = selectedId, sel != rev
        {
            let selectedRev = entries.first(where: { $0.change.matchesRevision(sel) })?.change.selectionRevision ?? sel
            actions?.compareWith(from: selectedRev, to: rev)
        } else {
            actions?.select(changeId: rev)
        }
    }

    private func rebaseMovementDistance(for rebaseDrag: DAGRebaseDragState, to location: CGPoint) -> CGFloat {
        DAGRebaseGesturePolicy.movementDistance(from: rebaseDrag.startLocation, to: location)
    }

    private func rebaseDragSeedLocation(for entry: GraphEntry, layout: DAGLayout) -> CGPoint? {
        guard let rowFrame = rebaseRowFrames[entry.change.commitId] else { return nil }
        let lane = layout.lane(for: entry.change.commitId)
        return CGPoint(
            x: rowFrame.minX + dagRowLeadingPadding + CGFloat(lane) * laneWidth + laneWidth / 2 + 4,
            y: rowFrame.midY
        )
    }
}

private struct DAGRebaseGhost: View {
    let label: String

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "arrow.up.forward.app")
                .jayjayFont(11, weight: .semibold)
            Text(label)
                .jayjayFont(11, weight: .medium)
                .lineLimit(1)
        }
        .foregroundStyle(.primary)
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.regularMaterial, in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.accentColor.opacity(0.25), lineWidth: 1)
        )
        .shadow(color: .black.opacity(0.12), radius: 8, y: 4)
    }
}

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
            // Anchor the ghost's bottom-right just up-left of the cursor (size-
            // independent), so it sits to the top-left of the pointer.
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
        guard bookmarkPreviewTargetId == change.commitId, let bookmarkDrag else { return nil }
        if bookmarkDrag.bookmarkName == workingCopyDragLabel {
            return "Move working copy here?"
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
        let normalized = hovered == drag.sourceCommitId ? nil : hovered
        drag.location = location
        drag.hoveredCommitId = normalized
        bookmarkDrag = drag
        updateBookmarkPreviewTarget(normalized)
    }

    private func updateBookmarkPreviewTarget(_ commitId: String?) {
        if commitId == bookmarkPreviewTargetId { return }
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
        .background(.regularMaterial, in: Circle())
        .overlay(Circle().stroke(Color.primary.opacity(0.12), lineWidth: 1))
        .shadow(color: .black.opacity(0.12), radius: 6, y: 3)
    }
}
