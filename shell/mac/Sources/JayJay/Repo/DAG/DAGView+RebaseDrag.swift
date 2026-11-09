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
        if let drag = rebaseDrag.state, drag.phase == .dragging {
            DAGRebaseGhost(label: drag.sourceLabel)
                .position(x: drag.location.x + 68, y: drag.location.y - 18)
                .allowsHitTesting(false)
        }
    }

    func rebasePreviewText(for change: ChangeInfo) -> String? {
        guard rebaseDrag.previewTargetId == change.commitId.id,
              let drag = rebaseDrag.state
        else { return nil }
        if let refusal = drag.targetRefusal {
            return refusal
        }
        return Localized.Repo.rebaseOntoQuestion(source: drag.sourceLabel, destination: DAGRebaseGesturePolicy.displayLabel(for: change))
    }

    private func handleRebaseGestureChanged(
        entry: GraphEntry,
        layout: DAGLayout,
        value: DragGesture.Value
    ) {
        // A bookmark-chip drag (started on a child view) wins over the row rebase.
        guard bookmarkDrag.state == nil else { return }
        let action = DAGRebaseGesturePolicy.changeAction(
            entryIsImmutable: entry.change.isImmutable,
            sourceCommitId: entry.change.commitId.id,
            rebaseDrag: rebaseDrag.state,
            location: value.location
        )

        switch action {
            case .ignore:
                break
            case .beginPress:
                beginRebasePress(for: entry, layout: layout, location: value.location)
            case .cancelPress:
                rebaseDrag.end()
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
        guard bookmarkDrag.state == nil else { return }
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: entry.change.isImmutable,
            sourceCommitId: entry.change.commitId.id,
            rebaseDrag: rebaseDrag.state,
            startLocation: value.startLocation,
            location: value.location
        )

        switch action {
            case .ignore:
                break
            case .select:
                rebaseDrag.end()
                selectEntry(entry)
            case .cancel:
                rebaseDrag.end()
            case .confirmDrop:
                updateRebaseDrag(location: value.location)
                confirmRebaseDrop()
        }
    }

    private func beginRebasePress(for entry: GraphEntry, layout: DAGLayout, location: CGPoint) {
        guard rebaseDrag.state?.sourceCommitId != entry.change.commitId.id else { return }
        let seedLocation = rebaseDragSeedLocation(for: entry, layout: layout) ?? location

        activePane = .dag
        rebaseDrag.armTask?.cancel()
        let selection = selectionCommitIds(draggedWith: entry)
        rebaseDrag.state = DAGRebaseDragState(
            sourceCommitId: entry.change.commitId.id,
            sourceChangeId: entry.change.changeId.id,
            sourceRev: DAGRebaseGesturePolicy.revision(for: entry.change),
            sourceLabel: selection.isEmpty
                ? DAGRebaseGesturePolicy.displayLabel(for: entry.change)
                : Localized.Repo.changeCount(count: selection.count),
            sourceParents: entry.change.parents,
            startLocation: location,
            phase: .pressing,
            location: seedLocation,
            hoveredCommitId: nil,
            selectionCommitIds: selection
        )
        scheduleRebaseArm(for: entry)
    }

    private func selectionCommitIds(draggedWith entry: GraphEntry) -> [String] {
        guard selectedIds.count > 1, selectedIds.contains(entry.change.selectionRevision) else { return [] }
        return entries.filter { selectedIds.contains($0.change.selectionRevision) }.map(\.change.commitId.id)
    }

    private func scheduleRebaseArm(for entry: GraphEntry) {
        let sourceCommitId = entry.change.commitId.id
        rebaseDrag.armTask = Task {
            try? await Task.sleep(for: .seconds(DAGRebaseGesturePolicy.armDuration))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard var drag = rebaseDrag.state,
                      drag.sourceCommitId == sourceCommitId,
                      drag.phase == .pressing
                else { return }
                drag.phase = .armed
                rebaseDrag.state = drag
            }
        }
    }

    private func beginDraggingIfNeeded() {
        guard var drag = rebaseDrag.state, drag.phase != .dragging else { return }
        drag.phase = .dragging
        if drag.selectionCommitIds.isEmpty {
            drag.descendantCommitIds = Set(descendantCommitIds(entries: entries, commitId: drag.sourceCommitId))
        } else {
            drag.selectionTargets = Set(
                entries.lazy.filter { capabilities.canRebase(onto: $0.change) }.map(\.change.commitId.id)
            )
        }
        rebaseDrag.state = drag
    }

    private func updateRebaseDrag(location: CGPoint) {
        guard var drag = rebaseDrag.state else { return }
        let hoveredCommitId = rowFrameCache.frames.first(where: { $0.value.contains(location) })?.key
        let normalizedTarget = DAGRebaseGesturePolicy.normalizedTargetCommitId(
            sourceCommitId: drag.sourceCommitId,
            hoveredCommitId: hoveredCommitId
        )
        drag.location = location
        if normalizedTarget != drag.hoveredCommitId {
            drag.hoveredCommitId = normalizedTarget
            drag.targetRefusal = normalizedTarget.flatMap {
                DAGRebaseGesturePolicy.targetRefusal(rebaseDrag: drag, targetCommitId: $0)
            }
        }
        rebaseDrag.state = drag
        // A refusal shows at once; only a valid target waits for the preview delay.
        updateRebasePreviewTarget(normalizedTarget, delayed: drag.targetRefusal == nil)
    }

    private func updateRebasePreviewTarget(_ commitId: String?, delayed: Bool) {
        if commitId == rebaseDrag.previewTargetId {
            return
        }
        rebaseDrag.clearPreview()
        guard let commitId else { return }
        guard delayed else {
            rebaseDrag.previewTargetId = commitId
            return
        }

        rebaseDrag.previewTask = Task {
            try? await Task.sleep(for: .milliseconds(DAGRebaseGesturePolicy.previewDelayMs))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard rebaseDrag.state?.hoveredCommitId == commitId else { return }
                rebaseDrag.previewTargetId = commitId
            }
        }
    }

    func confirmRebaseDrop() {
        guard let request = DAGRebaseGesturePolicy.dropRequest(
            rebaseDrag: rebaseDrag.state,
            previewTargetCommitId: rebaseDrag.previewTargetId,
            hoveredCommitId: rebaseDrag.state?.hoveredCommitId,
            entries: entries
        ) else {
            rebaseDrag.end()
            return
        }

        rebaseDrag.end()
        onRequest?(.rebase(request))
    }

    private func selectEntry(_ entry: GraphEntry) {
        activePane = .dag
        NSApp.keyWindow?.makeFirstResponder(nil)
        let rev = entry.change.selectionRevision
        let click = SelectionClick(modifiers: NSEvent.modifierFlags)
        switch click {
            case .toggle, .extend:
                actions?.updateSelection(changeId: rev, click: click)
            case .replace:
                actions?.select(changeId: rev)
        }
    }

    private func rebaseMovementDistance(for rebaseDrag: DAGRebaseDragState, to location: CGPoint) -> CGFloat {
        DAGRebaseGesturePolicy.movementDistance(from: rebaseDrag.startLocation, to: location)
    }

    private func rebaseDragSeedLocation(for entry: GraphEntry, layout: DAGLayout) -> CGPoint? {
        guard let rowFrame = rowFrameCache.frames[entry.change.commitId.id], let rowIndex = entries.firstIndex(where: { $0.change.commitId.id == entry.change.commitId.id }) else { return nil }
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
