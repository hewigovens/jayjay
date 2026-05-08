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
        let placement = rebaseDrag.hoveredPlacement ?? .onto
        return "\(placement.confirmationLabel) \(rebaseDrag.sourceLabel) \(placement.label) \(DAGRebaseGesturePolicy.displayLabel(for: change))?"
    }

    private func handleRebaseGestureChanged(
        entry: GraphEntry,
        layout: DAGLayout,
        value: DragGesture.Value
    ) {
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
            hoveredCommitId: nil,
            hoveredPlacement: nil
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
        let hoveredRow = rebaseRowFrames.first(where: { $0.value.contains(location) })
        let hoveredCommitId = hoveredRow?.key
        var normalizedTarget = DAGRebaseGesturePolicy.normalizedTargetCommitId(
            sourceCommitId: rebaseDrag.sourceCommitId,
            hoveredCommitId: hoveredCommitId
        )
        let targetEntry = entries.first(where: { $0.change.commitId == normalizedTarget })
        let placement = normalizedTarget == nil
            ? nil
            : DAGRebaseGesturePolicy.validPlacement(
                location: location,
                rowFrame: hoveredRow?.value,
                targetIsImmutable: targetEntry?.change.isImmutable ?? false
            )
        if placement == nil {
            normalizedTarget = nil
        }
        rebaseDrag.location = location
        rebaseDrag.hoveredCommitId = normalizedTarget
        rebaseDrag.hoveredPlacement = placement
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
            hoveredPlacement: rebaseDrag?.hoveredPlacement,
            entries: entries
        ) else {
            cancelRebaseDrag()
            return
        }

        cancelRebaseDrag()

        if let onRequestRebase {
            onRequestRebase(request)
        } else {
            actions?.rebase(rev: request.sourceRev, dest: request.destRev, placement: request.placement)
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
        if NSEvent.modifierFlags.contains(.shift),
           let sel = selectedId, sel != entry.change.changeId
        {
            actions?.compareWith(from: sel, to: entry.change.changeId)
        } else {
            actions?.select(changeId: entry.change.changeId)
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
