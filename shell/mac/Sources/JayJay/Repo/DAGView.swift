import AppKit
import JayJayCore
import SwiftUI

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let compareFromId: String?
    let actions: (any DAGActions)?
    @Binding var activePane: ActivePane
    var onMoveBookmarkForward: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onAbandon: ((String) -> Void)?
    var onCreateBookmark: ((String) -> Void)?
    var onLoadMore: (() -> Void)?

    @State private var contextTargetId: String?
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        Group {
            if entries.isEmpty {
                ContentUnavailableView(
                    "No Changes Matched",
                    systemImage: "line.3.horizontal.decrease.circle",
                    description: Text("Try a broader revset or refresh.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                let layout = DAGLayout(entries: entries)
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(entries.enumerated()), id: \.element.change.commitId) { index, entry in
                                DAGRow(
                                    entry: entry, layout: layout, index: index,
                                    isSelected: selectedId == entry.change.changeId,
                                    isCompareSource: compareFromId == entry.change.changeId,
                                    isContextTarget: contextTargetId == entry.change.changeId,
                                    isLast: index == entries.count - 1,
                                    colorScheme: colorScheme,
                                    onMoveBookmarkForward: onMoveBookmarkForward,
                                    onPushBookmark: onPushBookmark
                                )
                                .contentShape(Rectangle())
                                .onTapGesture {
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
                                .onHover { hovering in
                                    // Track right-click target via hover (context menu shows on hovered item)
                                    if hovering, let sel = selectedId, sel != entry.change.changeId {
                                        contextTargetId = entry.change.changeId
                                    } else if !hovering, contextTargetId == entry.change.changeId {
                                        contextTargetId = nil
                                    }
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
                                        let selEntry = entries.first { $0.change.changeId == sel }
                                        let selRev = selEntry?.change.isDivergent == true
                                            ? (selEntry?.change.commitId ?? sel) : sel
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
                    .background(
                        LinearGradient(
                            colors: [Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015), .clear],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .onChange(of: selectedId) { _, newValue in
                        guard let newValue,
                              let entry = entries.first(where: { $0.change.changeId == newValue })
                        else { return }
                        withAnimation(.easeOut(duration: 0.15)) {
                            proxy.scrollTo(entry.change.commitId, anchor: .center)
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
    }

    private func handleKeyDown(_ event: NSEvent) -> Bool {
        switch event.keyCode {
            case 125: moveSelection(by: 1)
                return true // Down arrow
            case 126: moveSelection(by: -1)
                return true // Up arrow
            default: break
        }
        let isCtrl = event.modifierFlags.intersection(.deviceIndependentFlagsMask) == .control
        switch event.charactersIgnoringModifiers {
            case "j": moveSelection(by: 1)
                return true
            case "k": moveSelection(by: -1)
                return true
            case "n" where isCtrl: moveSelection(by: 1)
                return true
            case "p" where isCtrl: moveSelection(by: -1)
                return true
            default: return false
        }
    }

    private func moveSelection(by delta: Int) {
        guard !entries.isEmpty else { return }
        let currentIdx: Int = if let selectedId,
                                 let idx = entries.firstIndex(where: { $0.change.changeId == selectedId })
        {
            idx
        } else {
            delta > 0 ? -1 : entries.count
        }
        let newIdx = max(0, min(entries.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return }
        actions?.select(changeId: entries[newIdx].change.changeId)
    }
}
