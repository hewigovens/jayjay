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
    var prHostName: String?
    var onMoveBookmarkForward: ((String) -> Void)?
    var onMoveBookmarkToRev: ((String, String) -> Void)?
    var onMoveWorkingCopyToRev: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onOpenPRForBookmark: ((String) -> Void)?
    var onDeleteBookmark: ((String) -> Void)?
    var onAbandon: ((String) -> Void)?
    var onCreateBookmark: ((String) -> Void)?
    var onCreateStackedPRs: ((String) -> Void)?
    var onLoadMore: (() -> Void)?

    @State private var contextTargetId: String?
    @State private var dagLayout: DAGLayout
    @State private var dagLayoutEntries: [GraphEntry]
    @State var rebaseRowFrames: [String: CGRect] = [:]
    @State var rebaseDrag: DAGRebaseDragState?
    @State var rebaseArmTask: Task<Void, Never>?
    @State var rebasePreviewTargetId: String?
    @State var rebasePreviewTask: Task<Void, Never>?
    @State var bookmarkDrag: BookmarkDragState?
    @State var bookmarkArmTask: Task<Void, Never>?
    @State var bookmarkPreviewTargetId: String?
    @State var bookmarkPreviewTask: Task<Void, Never>?
    @Environment(\.colorScheme) private var colorScheme

    init(
        entries: [GraphEntry],
        selectedId: String?,
        compareFromId: String?,
        actions: (any DAGActions)?,
        onRequestRebase: ((DAGRebaseRequest) -> Void)? = nil,
        activePane: Binding<ActivePane>,
        revealRequest: DAGRevealRequest? = nil,
        prHostName: String? = nil,
        onMoveBookmarkForward: ((String) -> Void)? = nil,
        onMoveBookmarkToRev: ((String, String) -> Void)? = nil,
        onMoveWorkingCopyToRev: ((String) -> Void)? = nil,
        onPushBookmark: ((String) -> Void)? = nil,
        onOpenPRForBookmark: ((String) -> Void)? = nil,
        onDeleteBookmark: ((String) -> Void)? = nil,
        onAbandon: ((String) -> Void)? = nil,
        onCreateBookmark: ((String) -> Void)? = nil,
        onCreateStackedPRs: ((String) -> Void)? = nil,
        onLoadMore: (() -> Void)? = nil
    ) {
        self.entries = entries
        self.selectedId = selectedId
        self.compareFromId = compareFromId
        self.actions = actions
        self.onRequestRebase = onRequestRebase
        _activePane = activePane
        self.revealRequest = revealRequest
        self.prHostName = prHostName
        self.onMoveBookmarkForward = onMoveBookmarkForward
        self.onMoveBookmarkToRev = onMoveBookmarkToRev
        self.onMoveWorkingCopyToRev = onMoveWorkingCopyToRev
        self.onPushBookmark = onPushBookmark
        self.onOpenPRForBookmark = onOpenPRForBookmark
        self.onDeleteBookmark = onDeleteBookmark
        self.onAbandon = onAbandon
        self.onCreateBookmark = onCreateBookmark
        self.onCreateStackedPRs = onCreateStackedPRs
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
            bookmarkDrag: bookmarkDrag,
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
                                let rowId = entry.change.selectionRevision
                                DAGRow(
                                    viewModel: viewModel.rowViewModel(
                                        for: entry,
                                        index: index,
                                        rebasePreviewText: rebasePreviewText(for: entry.change),
                                        bookmarkPreviewText: bookmarkPreviewText(for: entry.change)
                                    ),
                                    prHostName: prHostName,
                                    onMoveBookmarkForward: onMoveBookmarkForward,
                                    onPushBookmark: onPushBookmark,
                                    onOpenPRForBookmark: onOpenPRForBookmark,
                                    onDeleteBookmark: onDeleteBookmark,
                                    onBookmarkDragChanged: { name, sourceCommitId, value in
                                        handleBookmarkDragChanged(
                                            name: name,
                                            sourceCommitId: sourceCommitId,
                                            value: value
                                        )
                                    },
                                    onBookmarkDragEnded: { name, value in
                                        handleBookmarkDragEnded(name: name, value: value)
                                    }
                                )
                                .background(
                                    GeometryReader { geo in
                                        Color.clear.preference(
                                            key: DAGRebaseRowFramePreferenceKey.self,
                                            value: [
                                                entry.change.commitId.id: geo
                                                    .frame(in: .named(DAGRebaseCoordinateSpace.name))
                                            ]
                                        )
                                    }
                                )
                                .id(rowId)
                                .accessibilityElement(children: .combine)
                                .accessibilityIdentifier(AID.DAG.row(String(rowId.prefix(12))))
                                .contentShape(Rectangle())
                                .onHover { hovering in
                                    // Track right-click target via hover (context menu shows on hovered item)
                                    contextTargetId = viewModel.nextContextTargetId(hovering: hovering, entry: entry)
                                }
                                .contextMenu {
                                    rowContextMenu(entry: entry, viewModel: viewModel)
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
                    .scrollIndicators(.never)
                    .coordinateSpace(name: DAGRebaseCoordinateSpace.name)
                    .background(
                        LinearGradient(
                            colors: [Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015), .clear],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .overlay(alignment: .topLeading) { rebaseDragOverlay }
                    .overlay(alignment: .topLeading) { bookmarkDragOverlay }
                    .onPreferenceChange(DAGRebaseRowFramePreferenceKey.self) { rebaseRowFrames = $0 }
                    .onChange(of: entries.map(\.change.commitId)) { _, _ in
                        if viewModel.shouldCancelRebaseDrag(for: rebaseDrag?.hoveredCommitId) {
                            cancelRebaseDrag()
                        }
                        cancelBookmarkDrag()
                    }
                    .onChange(of: revealRequest?.id) { _, _ in
                        guard let changeId = revealRequest?.changeId else { return }
                        let scrollId = viewModel.scrollId(for: changeId)
                        withAnimation(.easeInOut(duration: 0.2)) {
                            proxy.scrollTo(scrollId, anchor: .center)
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
        handleBookmarkKeyDown(event) || handleRebaseKeyDown(event) || handleSelectionKeyDown(event)
    }

    private func handleBookmarkKeyDown(_ event: NSEvent) -> Bool {
        guard let bookmarkDrag, bookmarkDrag.phase != .pressing else { return false }
        switch event.keyCode {
            case 53:
                cancelBookmarkDrag()
                return true
            case 36, 76:
                confirmBookmarkDrop()
                return true
            default:
                return false
        }
    }

    private func moveSelection(by delta: Int) {
        let viewModel = DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            bookmarkDrag: bookmarkDrag,
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

    /// Short commit id + first description line, to tell apart sibling versions of a divergent change in the compare submenu.
    static func divergentSiblingLabel(_ change: ChangeInfo) -> String {
        let shortCommit = String(change.commitId.id.prefix(8))
        let firstLine = change.description
            .split(separator: "\n", maxSplits: 1, omittingEmptySubsequences: false)
            .first
            .map(String.init)?
            .trimmingCharacters(in: .whitespaces) ?? ""
        return firstLine.isEmpty ? shortCommit : "\(shortCommit) — \(firstLine)"
    }
}
