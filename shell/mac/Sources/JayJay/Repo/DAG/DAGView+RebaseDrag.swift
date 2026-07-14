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
        guard rebasePreviewTargetId == change.commitId.id,
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
            sourceCommitId: entry.change.commitId.id,
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
            sourceCommitId: entry.change.commitId.id,
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
        guard rebaseDrag?.sourceCommitId != entry.change.commitId.id else { return }
        // Row frames only mount once a drag state exists, so the first press seeds from the pointer; the ghost re-anchors from live frames on the first drag movement.
        let seedLocation = rebaseDragSeedLocation(for: entry, layout: layout) ?? location

        activePane = .dag
        rebaseArmTask?.cancel()
        rebaseDrag = DAGRebaseDragState(
            sourceCommitId: entry.change.commitId.id,
            sourceChangeId: entry.change.changeId.id,
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
        let sourceCommitId = entry.change.commitId.id
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
        guard let rowFrame = rebaseRowFrames[entry.change.commitId.id] else { return nil }
        guard let rowIndex = entries.firstIndex(where: { $0.change.commitId.id == entry.change.commitId.id }) else { return nil }
        let lane = layout.lane(for: entry.change.commitId.id)
        return CGPoint(
            x: rowFrame.minX + dagRowLeadingPadding + layout.xPosition(for: lane, at: rowIndex),
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
        .glassEffect(in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.accentColor.opacity(0.25), lineWidth: 1)
        )
        .shadow(color: .black.opacity(0.12), radius: 8, y: 4)
    }
}
