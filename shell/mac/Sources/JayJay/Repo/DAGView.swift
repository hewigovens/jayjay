import JayJayCore
import SwiftUI

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let compareFromId: String?
    let actions: (any DAGActions)?
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
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(entries.enumerated()), id: \.element.change.changeId) { index, entry in
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
                                // Navigation
                                Button { actions?.newChange(parent: entry.change.changeId, message: "") } label: {
                                    Label("New change on top", systemImage: "plus.circle")
                                }
                                Button { actions?.edit(rev: entry.change.changeId) } label: {
                                    Label("Edit (modify this commit)", systemImage: "pencil.circle")
                                }
                                if !entry.change.isImmutable {
                                    Button { actions?.squash(rev: entry.change.changeId) } label: {
                                        Label("Squash into parent", systemImage: "arrow.down.left.circle")
                                    }
                                }

                                if let sel = selectedId, sel != entry.change.changeId {
                                    Divider()
                                    // Selection actions
                                    Button { actions?.compareWith(from: sel, to: entry.change.changeId) } label: {
                                        Label("Compare with selected", systemImage: "arrow.left.arrow.right")
                                    }
                                    Button { actions?.rebase(rev: sel, dest: entry.change.changeId) } label: {
                                        Label("Rebase selected onto this", systemImage: "arrow.uturn.up")
                                    }
                                    if !entry.change.isImmutable {
                                        Button { actions?.squash(rev: sel, into: entry.change.changeId) } label: {
                                            Label("Squash selected into this", systemImage: "arrow.down.left.circle")
                                        }
                                    }
                                    Button { actions?.merge(parents: [sel, entry.change.changeId]) } label: {
                                        Label("Merge with selected", systemImage: "arrow.triangle.merge")
                                    }
                                }

                                Divider()
                                Button { onCreateBookmark?(entry.change.changeId) } label: {
                                    Label("Create bookmark here...", systemImage: "bookmark")
                                }

                                Divider()
                                Menu {
                                    Button { actions?.graft(rev: entry.change.changeId) } label: {
                                        Label("Cherry-pick (graft)", systemImage: "doc.on.clipboard")
                                    }
                                    Button { actions?.duplicate(rev: entry.change.changeId) } label: {
                                        Label("Duplicate", systemImage: "doc.on.doc")
                                    }
                                    if !entry.change.isImmutable {
                                        Button { actions?.absorb(rev: entry.change.changeId) } label: {
                                            Label("Absorb into ancestors", systemImage: "arrow.down.to.line")
                                        }
                                    }
                                    Button { actions?.backout(rev: entry.change.changeId) } label: {
                                        Label("Revert change", systemImage: "arrow.uturn.backward")
                                    }
                                } label: {
                                    Label("More Actions", systemImage: "ellipsis.circle")
                                }

                                if !entry.change.isImmutable {
                                    Divider()
                                    Button(role: .destructive) { onAbandon?(entry.change.changeId) } label: {
                                        Label("Abandon", systemImage: "trash")
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
            }
        }
    }
}
