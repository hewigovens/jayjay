import AppKit
import JayJayCore
import SwiftUI

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let compareFromId: String?
    let actions: (any DAGActions)?
    var onRequestRebase: ((DAGRebaseRequest) -> Void)?
    @Binding var activePane: ActivePane
    var revealRequest: DAGRevealRequest?
    var onMoveBookmarkForward: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onOpenPRForBookmark: ((String) -> Void)?
    var onAbandon: ((String) -> Void)?
    var onCreateBookmark: ((String) -> Void)?
    var onLoadMore: (() -> Void)?

    @State private var contextTargetId: String?
    @State private var dagLayout: DAGLayout
    @State private var dagLayoutEntries: [GraphEntry]
    @State var rebaseRowFrames: [String: CGRect] = [:]
    @State var rebaseDrag: DAGRebaseDragState?
    @State var rebaseArmTask: Task<Void, Never>?
    @State var rebasePreviewTargetId: String?
    @State var rebasePreviewTask: Task<Void, Never>?
    @Environment(\.colorScheme) private var colorScheme

    init(
        entries: [GraphEntry],
        selectedId: String?,
        compareFromId: String?,
        actions: (any DAGActions)?,
        onRequestRebase: ((DAGRebaseRequest) -> Void)? = nil,
        activePane: Binding<ActivePane>,
        revealRequest: DAGRevealRequest? = nil,
        onMoveBookmarkForward: ((String) -> Void)? = nil,
        onPushBookmark: ((String) -> Void)? = nil,
        onOpenPRForBookmark: ((String) -> Void)? = nil,
        onAbandon: ((String) -> Void)? = nil,
        onCreateBookmark: ((String) -> Void)? = nil,
        onLoadMore: (() -> Void)? = nil
    ) {
        self.entries = entries
        self.selectedId = selectedId
        self.compareFromId = compareFromId
        self.actions = actions
        self.onRequestRebase = onRequestRebase
        _activePane = activePane
        self.revealRequest = revealRequest
        self.onMoveBookmarkForward = onMoveBookmarkForward
        self.onPushBookmark = onPushBookmark
        self.onOpenPRForBookmark = onOpenPRForBookmark
        self.onAbandon = onAbandon
        self.onCreateBookmark = onCreateBookmark
        self.onLoadMore = onLoadMore
        _dagLayout = State(initialValue: DAGLayout(entries: entries))
        _dagLayoutEntries = State(initialValue: entries)
    }

    var body: some View {
        let viewModel = DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            colorScheme: colorScheme,
            layout: currentLayout
        )
        Group {
            if viewModel.isEmpty {
                ContentUnavailableView(
                    "No Changes Matched",
                    systemImage: "line.3.horizontal.decrease.circle",
                    description: Text("Try a broader revset or refresh.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(entries.enumerated()), id: \.element.change.commitId) { index, entry in
                                DAGRow(
                                    viewModel: viewModel.rowViewModel(
                                        for: entry,
                                        index: index,
                                        previewText: rebasePreviewText(for: entry.change)
                                    ),
                                    onMoveBookmarkForward: onMoveBookmarkForward,
                                    onPushBookmark: onPushBookmark,
                                    onOpenPRForBookmark: onOpenPRForBookmark
                                )
                                .background(
                                    GeometryReader { geo in
                                        Color.clear.preference(
                                            key: DAGRebaseRowFramePreferenceKey.self,
                                            value: [entry.change.commitId: geo
                                                .frame(in: .named(DAGRebaseCoordinateSpace.name))]
                                        )
                                    }
                                )
                                .id(entry.change.changeId)
                                .accessibilityIdentifier(AID.DAG.row(String(entry.change.changeId.prefix(12))))
                                .contentShape(Rectangle())
                                .onHover { hovering in
                                    // Track right-click target via hover (context menu shows on hovered item)
                                    contextTargetId = viewModel.nextContextTargetId(hovering: hovering, entry: entry)
                                }
                                .contextMenu {
                                    let rev = entry.change.isDivergent
                                        ? entry.change.commitId : entry.change.changeId
                                    // Navigation
                                    Button { actions?.newChange(parent: rev, message: "") } label: {
                                        Label("New change on top", systemImage: "plus.circle")
                                    }
                                    Button { actions?.edit(rev: rev) } label: {
                                        Label("Edit (modify this commit)", systemImage: "pencil.circle")
                                    }
                                    if !entry.change.isImmutable {
                                        Button { actions?.squash(rev: rev) } label: {
                                            Label("Squash into parent", systemImage: "arrow.down.left.circle")
                                        }
                                    }

                                    if let sel = selectedId, sel != entry.change.changeId {
                                        Divider()
                                        let selRev = viewModel.selectedRevision(for: sel)
                                        // Selection actions
                                        Button { actions?.compareWith(from: selRev, to: rev) } label: {
                                            Label("Compare with selected", systemImage: "arrow.left.arrow.right")
                                        }
                                        Button { actions?.rebase(rev: selRev, dest: rev) } label: {
                                            Label("Rebase selected onto this", systemImage: "arrow.uturn.up")
                                        }
                                        if !entry.change.isImmutable {
                                            Button { actions?.squash(rev: selRev, into: rev) } label: {
                                                Label(
                                                    "Squash selected into this",
                                                    systemImage: "arrow.down.left.circle"
                                                )
                                            }
                                        }
                                        Button { actions?.merge(parents: [selRev, rev]) } label: {
                                            Label("Merge with selected", systemImage: "arrow.triangle.merge")
                                        }
                                    }

                                    Divider()
                                    Button { onCreateBookmark?(rev) } label: {
                                        Label("Create bookmark here...", systemImage: "bookmark")
                                    }

                                    Divider()
                                    Menu {
                                        Button { actions?.graft(rev: rev) } label: {
                                            Label("Cherry-pick (graft)", systemImage: "doc.on.clipboard")
                                        }
                                        Button { actions?.duplicate(rev: rev) } label: {
                                            Label("Duplicate", systemImage: "doc.on.doc")
                                        }
                                        if !entry.change.isImmutable {
                                            Button { actions?.absorb(rev: rev) } label: {
                                                Label("Absorb into ancestors", systemImage: "arrow.down.to.line")
                                            }
                                        }
                                        Button { actions?.backout(rev: rev) } label: {
                                            Label("Revert change", systemImage: "arrow.uturn.backward")
                                        }
                                    } label: {
                                        Label("More Actions", systemImage: "ellipsis.circle")
                                    }

                                    if !entry.change.isImmutable {
                                        Divider()
                                        if entry.change.isDivergent {
                                            Button(role: .destructive) { onAbandon?(rev) } label: {
                                                Label(
                                                    "Abandon (resolve divergence)",
                                                    systemImage: "arrow.triangle.merge"
                                                )
                                            }
                                        } else {
                                            Button(role: .destructive) { onAbandon?(rev) } label: {
                                                Label("Abandon", systemImage: "trash")
                                            }
                                        }
                                    }
                                }
                                .simultaneousGesture(rebaseGesture(for: entry, layout: viewModel.layout))
                            }
                            if let onLoadMore {
                                Button {
                                    onLoadMore()
                                } label: {
                                    HStack {
                                        Spacer()
                                        Label("Load More", systemImage: "arrow.down.circle")
                                            .jayjayFont(12, weight: .medium)
                                            .foregroundStyle(.secondary)
                                        Spacer()
                                    }
                                    .padding(.vertical, 8)
                                    .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.vertical, 6)
                    }
                    .coordinateSpace(name: DAGRebaseCoordinateSpace.name)
                    .background(
                        LinearGradient(
                            colors: [Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015), .clear],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .overlay(alignment: .topLeading) { rebaseDragOverlay }
                    .onPreferenceChange(DAGRebaseRowFramePreferenceKey.self) { rebaseRowFrames = $0 }
                    .onChange(of: entries.map(\.change.commitId)) { _, _ in
                        if viewModel.shouldCancelRebaseDrag(for: rebaseDrag?.hoveredCommitId) {
                            cancelRebaseDrag()
                        }
                    }
                    .onChange(of: revealRequest?.id) { _, _ in
                        guard let changeId = revealRequest?.changeId else { return }
                        withAnimation(.easeInOut(duration: 0.2)) {
                            proxy.scrollTo(changeId, anchor: .center)
                        }
                    }
                }
            }
        }
        .background(
            KeyDownMonitor(
                isActive: { activePane == .dag },
                onKeyDown: { event in handleKeyDown(event) }
            )
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
        .onChange(of: entries) { _, _ in
            updateDagLayout()
        }
    }

    private func handleKeyDown(_ event: NSEvent) -> Bool {
        handleRebaseKeyDown(event) || handleSelectionKeyDown(event)
    }

    private func moveSelection(by delta: Int) {
        let viewModel = DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            colorScheme: colorScheme,
            layout: currentLayout
        )
        guard let changeId = viewModel.selectedChangeId(afterMovingBy: delta) else { return }
        actions?.select(changeId: changeId)
    }

    private var currentLayout: DAGLayout {
        dagLayoutEntries == entries ? dagLayout : DAGLayout(entries: entries)
    }

    private func updateDagLayout() {
        guard dagLayoutEntries != entries else { return }
        dagLayout = DAGLayout(entries: entries)
        dagLayoutEntries = entries
    }

    private func handleRebaseKeyDown(_ event: NSEvent) -> Bool {
        guard let rebaseDrag, rebaseDrag.phase != .pressing else { return false }
        switch event.keyCode {
            case 53:
                cancelRebaseDrag()
                return true
            case 36, 76:
                confirmRebaseDrop()
                return true
            default:
                return false
        }
    }

    private func handleSelectionKeyDown(_ event: NSEvent) -> Bool {
        let isCtrl = event.modifierFlags.intersection(.deviceIndependentFlagsMask) == .control
        guard let delta = DAGViewModel.selectionDelta(
            keyCode: event.keyCode,
            charactersIgnoringModifiers: event.charactersIgnoringModifiers,
            controlPressed: isCtrl
        ) else { return false }
        moveSelection(by: delta)
        return true
    }
}
