import AppKit
import JayJayCore
import SwiftUI

struct DAGView: View {
    let entries: [GraphEntry]
    let layout: DAGLayout
    let selectedId: String?
    let selectedIds: [String]
    let compareFromId: String?
    let actions: (any DAGActions)?
    var onRequestRebase: ((DAGRebaseRequest) -> Void)?
    @Binding var activePane: ActivePane
    var revealRequest: DAGRevealRequest?
    var prHostName: String?
    var onMoveBookmarkToRev: ((String, String) -> Void)?
    var onMoveWorkingCopyToRev: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onOpenPRForBookmark: ((String) -> Void)?
    var onDeleteBookmark: ((String, String) -> Void)?
    var conflictedBookmarkNames: Set<String> = []
    var onAbandon: ((String) -> Void)?
    var onAbandonSelection: (([String]) -> Void)?
    var onSquashSelection: (([String]) -> Void)?
    var onCreateBookmark: ((String) -> Void)?
    var onCreateStackedPRs: ((String) -> Void)?
    var onLoadMore: (() -> Void)?

    @State private var contextTargetId: String?
    @State private var sidebarWidth: CGFloat = 0
    @State var rebaseRowFrames: [String: CGRect] = [:]
    @State var rebaseDrag: DAGRebaseDragState?
    @State var rebaseArmTask: Task<Void, Never>?
    @State var rebasePreviewTargetId: String?
    @State var rebasePreviewTask: Task<Void, Never>?
    @State var bookmarkDrag: BookmarkDragState?
    @State var bookmarkArmTask: Task<Void, Never>?
    @State var bookmarkPreviewTargetId: String?
    @State var bookmarkPreviewTask: Task<Void, Never>?
    @State private var keyboardReveal: DAGRevealRequest?
    @Environment(\.colorScheme) private var colorScheme

    init(
        entries: [GraphEntry],
        layout: DAGLayout,
        selectedId: String?,
        selectedIds: [String],
        compareFromId: String?,
        actions: (any DAGActions)?,
        onRequestRebase: ((DAGRebaseRequest) -> Void)? = nil,
        activePane: Binding<ActivePane>,
        revealRequest: DAGRevealRequest? = nil,
        prHostName: String? = nil,
        onMoveBookmarkToRev: ((String, String) -> Void)? = nil,
        onMoveWorkingCopyToRev: ((String) -> Void)? = nil,
        onPushBookmark: ((String) -> Void)? = nil,
        onOpenPRForBookmark: ((String) -> Void)? = nil,
        onDeleteBookmark: ((String, String) -> Void)? = nil,
        conflictedBookmarkNames: Set<String> = [],
        onAbandon: ((String) -> Void)? = nil,
        onAbandonSelection: (([String]) -> Void)? = nil,
        onSquashSelection: (([String]) -> Void)? = nil,
        onCreateBookmark: ((String) -> Void)? = nil,
        onCreateStackedPRs: ((String) -> Void)? = nil,
        onLoadMore: (() -> Void)? = nil
    ) {
        self.entries = entries
        self.layout = layout
        self.selectedId = selectedId
        self.selectedIds = selectedIds
        self.compareFromId = compareFromId
        self.actions = actions
        self.onRequestRebase = onRequestRebase
        _activePane = activePane
        self.revealRequest = revealRequest
        self.prHostName = prHostName
        self.onMoveBookmarkToRev = onMoveBookmarkToRev
        self.onMoveWorkingCopyToRev = onMoveWorkingCopyToRev
        self.onPushBookmark = onPushBookmark
        self.onOpenPRForBookmark = onOpenPRForBookmark
        self.onDeleteBookmark = onDeleteBookmark
        self.conflictedBookmarkNames = conflictedBookmarkNames
        self.onAbandon = onAbandon
        self.onAbandonSelection = onAbandonSelection
        self.onSquashSelection = onSquashSelection
        self.onCreateBookmark = onCreateBookmark
        self.onCreateStackedPRs = onCreateStackedPRs
        self.onLoadMore = onLoadMore
    }

    var body: some View {
        let viewModel = DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            selectedIds: selectedIds,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            bookmarkDrag: bookmarkDrag,
            colorScheme: colorScheme,
            layout: layout,
            geometry: DAGGeometry(logicalColumnCount: layout.logicalColumnCount, availableSidebarWidth: sidebarWidth)
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
                                let rowViewModel = viewModel.rowViewModel(
                                    for: entry,
                                    index: index,
                                    rebasePreviewText: rebasePreviewText(for: entry.change),
                                    bookmarkPreviewText: bookmarkPreviewText(for: entry.change)
                                )
                                DAGRow(
                                    viewModel: rowViewModel,
                                    prHostName: prHostName,
                                    onMoveBookmarkToRev: onMoveBookmarkToRev,
                                    onPushBookmark: onPushBookmark,
                                    onOpenPRForBookmark: onOpenPRForBookmark,
                                    onDeleteBookmark: { name in
                                        onDeleteBookmark?(name, entry.change.commitId.id)
                                    },
                                    conflictedBookmarkNames: conflictedBookmarkNames,
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
                                .background(rebaseFrameReader(for: entry.change.commitId.id))
                                .id(rowId)
                                .accessibilityElement(children: .combine)
                                .accessibilityIdentifier(AID.DAG.row(String(rowId.prefix(12))))
                                .accessibilityAddTraits(
                                    rowViewModel.isSelectionHighlighted ? .isSelected : []
                                )
                                .contentShape(Rectangle())
                                .onHover { hovering in
                                    // Track right-click target via hover (context menu shows on hovered item)
                                    contextTargetId = viewModel.nextContextTargetId(hovering: hovering, entry: entry)
                                }
                                .contextMenu {
                                    rowContextMenu(entry: entry, viewModel: viewModel)
                                }
                                .simultaneousGesture(
                                    rebaseGesture(for: entry, layout: viewModel.layout, geometry: viewModel.geometry)
                                )
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
                    .onChange(of: keyboardReveal?.id) { _, _ in
                        guard let changeId = keyboardReveal?.changeId else { return }
                        proxy.scrollTo(viewModel.scrollId(for: changeId), anchor: nil)
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
        .background(sidebarWidthReader)
    }

    /// One `DAGGeometry` is shared by every row, so it must come from the whole view's width, not a per-row measurement — otherwise rows could disagree on lane pitch.
    private var sidebarWidthReader: some View {
        GeometryReader { geo in
            Color.clear
                .onAppear { sidebarWidth = geo.size.width }
                .onChange(of: geo.size.width) { _, newValue in sidebarWidth = newValue }
        }
    }

    /// Bookmark drags can begin and end within one SwiftUI update, so row targets must exist before the gesture starts.
    private func rebaseFrameReader(for commitId: String) -> some View {
        GeometryReader { geo in
            Color.clear.preference(
                key: DAGRebaseRowFramePreferenceKey.self,
                value: [commitId: geo.frame(in: .named(DAGRebaseCoordinateSpace.name))]
            )
        }
    }

    private func handleKeyDown(_ event: NSEvent) -> Bool {
        handleBookmarkKeyDown(event) || handleRebaseKeyDown(event) || handleSelectionKeyDown(event)
    }

    private func handleBookmarkKeyDown(_ event: NSEvent) -> Bool {
        guard let bookmarkDrag, bookmarkDrag.phase != .pressing else { return false }
        switch event.keyCode {
            case KeyCode.escape:
                cancelBookmarkDrag()
                return true
            case KeyCode.returnKey, KeyCode.keypadEnter:
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
            selectedIds: selectedIds,
            compareFromId: compareFromId,
            contextTargetId: contextTargetId,
            rebaseDrag: rebaseDrag,
            bookmarkDrag: bookmarkDrag,
            colorScheme: colorScheme,
            layout: layout,
            geometry: DAGGeometry(logicalColumnCount: layout.logicalColumnCount, availableSidebarWidth: sidebarWidth)
        )
        guard let changeId = viewModel.selectedChangeId(afterMovingBy: delta) else { return }
        actions?.select(changeId: changeId, coalescing: true)
        keyboardReveal = DAGRevealRequest(changeId: changeId)
    }

    private func handleRebaseKeyDown(_ event: NSEvent) -> Bool {
        guard let rebaseDrag, rebaseDrag.phase != .pressing else { return false }
        switch event.keyCode {
            case KeyCode.escape:
                cancelRebaseDrag()
                return true
            case KeyCode.returnKey, KeyCode.keypadEnter:
                confirmRebaseDrop()
                return true
            default:
                return false
        }
    }

    private func handleSelectionKeyDown(_ event: NSEvent) -> Bool {
        if event.keyCode == KeyCode.escape, selectedIds.count > 1 {
            actions?.select(changeId: selectedId)
            return true
        }
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
