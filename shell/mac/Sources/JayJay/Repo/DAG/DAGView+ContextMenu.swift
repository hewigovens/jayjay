import JayJayCore
import SwiftUI

extension DAGView {
    @ViewBuilder
    func rowContextMenu(entry: GraphEntry, viewModel: DAGViewModel) -> some View {
        let rev = entry.change.isDivergent
            ? entry.change.commitId.id : entry.change.changeId.id
        // Navigation
        Button { actions?.newChange(parent: rev, message: "") } label: {
            Label("New change on top", systemImage: "plus.circle")
        }
        if !entry.change.isImmutable {
            Button { actions?.edit(rev: rev) } label: {
                Label("Edit (modify this commit)", systemImage: "pencil.circle")
            }
            if viewModel.canSquashIntoParent(entry.change) {
                Button { actions?.squash(rev: rev) } label: {
                    Label("Squash into parent", systemImage: "arrow.down.left.circle")
                }
            }
            if !entry.change.isWorkingCopy {
                Button { actions?.squash(rev: rev, into: "@") } label: {
                    Label(
                        "Move changes to working copy",
                        systemImage: "tray.and.arrow.down"
                    )
                }
            }
        }

        divergentCompareSection(entry: entry, rev: rev, viewModel: viewModel)
        selectionActionsSection(entry: entry, rev: rev, viewModel: viewModel)

        Divider()
        Button { onCreateBookmark?(rev) } label: {
            Label("Create bookmark here...", systemImage: "bookmark")
        }
        if !entry.change.isImmutable {
            Button { onCreateStackedPRs?(rev) } label: {
                Label(
                    "Create / Update Stacked PRs…",
                    systemImage: "square.stack.3d.up.fill"
                )
            }
            Button { actions?.showEvolog(rev: rev) } label: {
                Label("Show evolution…", systemImage: "clock.arrow.circlepath")
            }
        }

        Divider()
        Menu {
            Button { actions?.duplicate(rev: rev) } label: {
                Label("Duplicate", systemImage: "doc.on.doc")
            }
            if !entry.change.isImmutable {
                Button { actions?.absorb(rev: rev) } label: {
                    Label("Absorb into ancestors", systemImage: "arrow.down.to.line")
                }
            }
            Button { actions?.revertChange(rev: rev) } label: {
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

    @ViewBuilder
    private func divergentCompareSection(
        entry: GraphEntry, rev: String, viewModel: DAGViewModel
    ) -> some View {
        let divergentSiblings = viewModel.divergentSiblings(of: entry.change)
        if !divergentSiblings.isEmpty {
            Divider()
            if divergentSiblings.count == 1 {
                Button {
                    actions?.compareWith(from: divergentSiblings[0].commitId.id, to: rev)
                } label: {
                    Label(
                        "Compare Divergent Version",
                        systemImage: "arrow.left.arrow.right.square"
                    )
                }
            } else {
                Menu {
                    ForEach(divergentSiblings, id: \.commitId.id) { sibling in
                        Button {
                            actions?.compareWith(from: sibling.commitId.id, to: rev)
                        } label: {
                            Text(Self.divergentSiblingLabel(sibling))
                        }
                    }
                } label: {
                    Label(
                        "Compare Divergent Versions",
                        systemImage: "arrow.left.arrow.right.square"
                    )
                }
            }
        }
    }

    @ViewBuilder
    private func selectionActionsSection(
        entry: GraphEntry, rev: String, viewModel: DAGViewModel
    ) -> some View {
        if let sel = selectedId, sel != entry.change.changeId.id {
            Divider()
            let selRev = viewModel.selectedRevision(for: sel)
            Button { actions?.compareWith(from: selRev, to: rev) } label: {
                Label("Compare with selected", systemImage: "arrow.left.arrow.right")
            }
            if let request = viewModel.bookmarkDiffRequest(from: sel, to: entry.change) {
                Button { actions?.diffBookmark(request) } label: {
                    Label("Diff Bookmark", systemImage: "arrow.left.arrow.right.circle")
                }
            }
            // Rebase and squash rewrite the selected change; this row is only the destination.
            let selectedImmutable = viewModel.change(for: sel)?.isImmutable ?? false
            if !selectedImmutable {
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
            }
            Button { actions?.merge(parents: [selRev, rev]) } label: {
                Label("Merge with selected", systemImage: "arrow.triangle.merge")
            }
        }
    }
}
