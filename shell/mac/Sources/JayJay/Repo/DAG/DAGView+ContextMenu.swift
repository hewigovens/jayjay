import AppKit
import JayJayCore
import SwiftUI

extension DAGView {
    @ViewBuilder
    func rowContextMenu(entry: GraphEntry, viewModel: DAGViewModel) -> some View {
        if viewModel.hasMultipleSelection, viewModel.isSelected(entry.change) {
            multiSelectionContextMenu(viewModel: viewModel)
        } else {
            singleChangeContextMenu(entry: entry, viewModel: viewModel)
        }
    }

    @ViewBuilder
    private func multiSelectionContextMenu(viewModel: DAGViewModel) -> some View {
        let revisions = viewModel.selectedRevisions
        Button { actions?.merge(parents: revisions) } label: {
            Label("Merge \(revisions.count) selected", systemImage: "arrow.triangle.merge")
        }
        .disabled(!viewModel.canMergeSelection)
        .help(
            viewModel.canMergeSelection
                ? "Create a new change with the selected changes as parents"
                : "Merge requires independent heads"
        )

        Button { onSquashSelection?(revisions) } label: {
            Label("Squash \(revisions.count) selected…", systemImage: "arrow.down.left.circle")
        }
        .disabled(!viewModel.canSquashSelection)
        .help(
            viewModel.canSquashSelection
                ? "Combine the selected range into its oldest change"
                : "Squash requires a consecutive linear range of mutable changes"
        )

        Divider()
        Button(role: .destructive) { onAbandonSelection?(revisions) } label: {
            Label("Abandon \(revisions.count) selected…", systemImage: "trash")
        }
        .disabled(!viewModel.canAbandonSelection)
        .help(
            viewModel.canAbandonSelection
                ? "Abandon the selected changes"
                : "Immutable changes cannot be abandoned"
        )
    }

    @ViewBuilder
    private func singleChangeContextMenu(entry: GraphEntry, viewModel: DAGViewModel) -> some View {
        let rev = entry.change.isDivergent
            ? entry.change.commitId.id : entry.change.changeId.id
        // Navigation
        if entry.change.newChange.onTop {
            Button { actions?.newChange(parent: rev, message: "") } label: {
                Label("New change on top", systemImage: "plus.circle")
            }
        }
        if entry.change.newChange.before {
            Button { actions?.insertChange(rev: rev, position: .before) } label: {
                Label("New change before", systemImage: "arrow.down.circle")
            }
        }
        if entry.change.newChange.after {
            Button { actions?.insertChange(rev: rev, position: .after) } label: {
                Label("New change after", systemImage: "arrow.up.circle")
            }
        }
        Divider()
        if !entry.change.isImmutable {
            Button { actions?.edit(rev: rev) } label: {
                Label("Edit (modify this change)", systemImage: "pencil.circle")
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
        }
        Button { actions?.showEvolog(rev: rev) } label: {
            Label("Show evolution…", systemImage: "clock.arrow.circlepath")
        }

        identifierCopySection(change: entry.change)
        moreActionsMenu(entry: entry, rev: rev)
        if !entry.change.isImmutable {
            Divider()
            abandonButton(entry: entry, rev: rev)
        }
    }

    private func moreActionsMenu(entry: GraphEntry, rev: String) -> some View {
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
    }

    @ViewBuilder
    private func identifierCopySection(change: ChangeInfo) -> some View {
        Divider()
        Button { copyToPasteboard(change.changeId.id) } label: {
            Label("Copy Change ID", systemImage: "doc.on.doc")
        }
        Button { copyToPasteboard(change.commitId.id) } label: {
            Label("Copy Commit ID", systemImage: "doc.on.doc")
        }
        Divider()
    }

    private func abandonButton(entry: GraphEntry, rev: String) -> some View {
        Button(role: .destructive) { onAbandon?(rev) } label: {
            if entry.change.isDivergent {
                Label("Abandon (resolve divergence)", systemImage: "arrow.triangle.merge")
            } else {
                Label("Abandon", systemImage: "trash")
            }
        }
    }

    private func copyToPasteboard(_ value: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(value, forType: .string)
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
        if viewModel.hasMultipleSelection {
            Divider()
            let revisions = viewModel.selectedRevisions
            Button { actions?.rebase(revs: revisions, dest: rev) } label: {
                Label(
                    "Rebase \(revisions.count) selected onto this",
                    systemImage: "arrow.uturn.up"
                )
            }
            .disabled(!viewModel.canRebaseSelection(onto: entry.change))
            .help(
                viewModel.canRebaseSelection(onto: entry.change)
                    ? "Rebase the selected changes while preserving their dependencies"
                    : "Rebase requires mutable changes and a destination outside their descendants"
            )
        } else if let sel = selectedId, sel != entry.change.changeId.id {
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
            let canMerge = viewModel.canMergeSelectedChange(with: entry.change)
            Button { actions?.merge(parents: [selRev, rev]) } label: {
                Label("Merge with selected", systemImage: "arrow.triangle.merge")
            }
            .disabled(!canMerge)
            .help(
                canMerge
                    ? "Create a new change with both changes as parents"
                    : "Merge requires independent heads"
            )
        }
    }
}
