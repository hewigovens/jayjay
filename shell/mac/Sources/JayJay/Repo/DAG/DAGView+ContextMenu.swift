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
        if viewModel.canMergeSelection {
            Menu {
                Section(Localized.Menu.firstParent) {
                    ForEach(viewModel.mergeParentChoices, id: \.parents) { choice in
                        Button(choice.label) { actions?.merge(parents: choice.parents) }
                    }
                }
            } label: {
                Label(Localized.Menu.mergeSelected(count: revisions.count), systemImage: "arrow.triangle.merge")
            }
            .help(Localized.Menu.chooseTheFirstParentForThe)
        } else {
            Button {} label: {
                Label(Localized.Menu.mergeSelected(count: revisions.count), systemImage: "arrow.triangle.merge")
            }
            .disabled(true)
            .help(Localized.Menu.mergeRequiresIndependentHeads)
        }

        Button { onRequest?(.squashSelection(revisions: revisions)) } label: {
            Label(Localized.Menu.squashSelected(count: revisions.count), systemImage: "arrow.down.left.circle")
        }
        .disabled(!viewModel.canSquashSelection)
        .help(
            viewModel.canSquashSelection
                ? Localized.Menu.combineTheSelectedRangeIntoIts
                : Localized.Menu.squashRequiresAConsecutiveLinearRange
        )

        Button { actions?.parallelize(revs: revisions) } label: {
            Label(Localized.Menu.parallelizeSelected(count: revisions.count), systemImage: "arrow.triangle.branch")
        }
        .disabled(!viewModel.canParallelizeSelection)
        .help(
            viewModel.canParallelizeSelection
                ? Localized.Menu.makeTheSelectedChangesIndependentSiblings
                : Localized.Menu.parallelizeRequiresTwoOrMoreMutable
        )

        fixButton(
            Localized.Menu.fixSelected(count: revisions.count),
            revs: revisions,
            selectionCanFix: viewModel.canFixSelection
        )

        bisectBetweenButton(viewModel)

        Divider()
        Button(role: .destructive) { onRequest?(.abandonSelection(revisions: revisions)) } label: {
            Label(Localized.Menu.abandonSelected(count: revisions.count), systemImage: "trash")
        }
        .keyboardShortcut("a", modifiers: [])
        .disabled(!viewModel.canAbandonSelection)
        .help(
            viewModel.canAbandonSelection
                ? Localized.Menu.abandonTheSelectedChanges
                : Localized.Menu.immutableChangesCannotBeAbandoned
        )
    }

    @ViewBuilder
    private func bisectBetweenButton(_ viewModel: DAGViewModel) -> some View {
        let selected = viewModel.entries.filter { viewModel.isSelected($0.change) }
        if selected.count == 2 {
            let first = selected[0].change.commitId.id
            let second = selected[1].change.commitId.id
            Button { onRequest?(.bisectBetween(first: first, second: second)) } label: {
                Label(Localized.Menu.bisectBetween, systemImage: "arrow.left.and.right")
            }
        }
    }

    private func bisectFromButton(_ entry: GraphEntry) -> some View {
        Button { onRequest?(.bisectFrom(commitId: entry.change.commitId.id)) } label: {
            Label(Localized.Menu.bisectFromHereEllipsis, systemImage: "arrow.left.and.right")
        }
    }

    @ViewBuilder
    private func singleChangeContextMenu(entry: GraphEntry, viewModel: DAGViewModel) -> some View {
        let rev = entry.change.isDivergent
            ? entry.change.commitId.id : entry.change.changeId.id
        if entry.change.isWorkingCopy, refreshMode == .updateWorkspace {
            Button { actions?.updateStaleWorkspace() } label: {
                Label(Localized.Menu.updateWorkspace, systemImage: "exclamationmark.triangle")
            }
            Divider()
        }
        // Navigation
        if entry.change.newChange.onTop {
            Button { actions?.newChange(parent: rev, message: "") } label: {
                Label(Localized.Menu.newChangeOnTop, systemImage: "plus.circle")
            }
            .keyboardShortcut("n", modifiers: [])
        }
        if entry.change.newChange.before {
            Button { actions?.insertChange(rev: rev, position: .before) } label: {
                Label(Localized.Menu.newChangeBefore, systemImage: "arrow.down.circle")
            }
        }
        if entry.change.newChange.after {
            Button { actions?.insertChange(rev: rev, position: .after) } label: {
                Label(Localized.Menu.newChangeAfter, systemImage: "arrow.up.circle")
            }
        }
        Divider()
        rewriteActionsSection(entry: entry, rev: rev, viewModel: viewModel)

        divergentCompareSection(entry: entry, rev: rev, viewModel: viewModel)
        selectionActionsSection(entry: entry, rev: rev, viewModel: viewModel)

        Divider()
        Button { onRequest?(.createBookmark(rev: rev)) } label: {
            Label(Localized.Menu.createBookmarkHereEllipsis, systemImage: "bookmark")
        }
        .keyboardShortcut("b", modifiers: [])
        Button { onRequest?(.createTag(rev: entry.change.commitId.id)) } label: {
            Label(Localized.Menu.createTagHereEllipsis, systemImage: "tag")
        }
        .keyboardShortcut("t", modifiers: [])
        if !entry.change.isImmutable {
            Button { onRequest?(.createStackedPRs(rev: rev)) } label: {
                Label(
                    Localized.Menu.createUpdateStackedPrsEllipsis,
                    systemImage: "square.stack.3d.up.fill"
                )
            }
        }
        Divider()
        bisectFromButton(entry)
        ForEach(entry.change.tags, id: \.self) { name in
            Menu(Localized.Repo.tagNamed(name: name)) {
                TagMenuItems(name: name, isOnRemote: remoteTagNames.contains(name), actions: actions, onRequest: onRequest)
            }
        }
        historySection(entry: entry, rev: rev)

        identifierCopySection(change: entry.change)
        moreActionsMenu(entry: entry, rev: rev)
        if !entry.change.isImmutable {
            Divider()
            abandonButton(entry: entry, rev: rev)
        }
    }

    @ViewBuilder
    private func rewriteActionsSection(entry: GraphEntry, rev: String, viewModel: DAGViewModel) -> some View {
        if !entry.change.isImmutable {
            Button { actions?.edit(rev: rev) } label: {
                Label(Localized.Menu.editModifyThisChange, systemImage: "pencil.circle")
            }
            .keyboardShortcut("e", modifiers: [])
            if !entry.change.isWorkingCopy {
                Button { onRequest?(.editDescription(rev: rev, description: entry.change.description)) } label: {
                    Label(Localized.Menu.describeEllipsis, systemImage: "square.and.pencil")
                }
                .keyboardShortcut("d", modifiers: [])
            }
            if viewModel.canSquashIntoParent(entry.change) {
                Button { actions?.squash(rev: rev) } label: {
                    Label(Localized.Menu.squashIntoParent, systemImage: "arrow.down.left.circle")
                }
                .keyboardShortcut("s", modifiers: [])
            }
            if !entry.change.isWorkingCopy {
                Button { actions?.squash(rev: rev, into: "@") } label: {
                    Label(
                        Localized.Menu.moveChangesToWorkingCopy,
                        systemImage: "tray.and.arrow.down"
                    )
                }
                .keyboardShortcut("m", modifiers: [])
            }
            Button { actions?.rebase(rev: rev, dest: "trunk()", mode: .branch) } label: {
                Label(Localized.Menu.rebaseOntoTrunk, systemImage: "arrow.uturn.up")
            }
            .keyboardShortcut("r", modifiers: [])
            fixButton(Localized.Menu.runFormattersJjFix, revs: [rev])
        }
    }

    @ViewBuilder
    private func historySection(entry: GraphEntry, rev: String) -> some View {
        Button { onRequest?(.showAncestors(changeId: entry.change.changeId.id, commitId: entry.change.commitId.id)) } label: {
            Label(Localized.Menu.showAncestorsEllipsis, systemImage: "arrow.triangle.branch")
        }
        Button { actions?.showEvolog(rev: rev) } label: {
            Label(Localized.Menu.showEvolutionEllipsis, systemImage: "clock.arrow.circlepath")
        }
        .keyboardShortcut("v", modifiers: [])
    }

    private func moreActionsMenu(entry: GraphEntry, rev: String) -> some View {
        Menu {
            Button { actions?.duplicate(rev: rev) } label: {
                Label(Localized.Menu.duplicate, systemImage: "doc.on.doc")
            }
            if !entry.change.isImmutable {
                Button { actions?.absorb(rev: rev) } label: {
                    Label(Localized.Menu.absorbIntoAncestors, systemImage: "arrow.down.to.line")
                }
            }
            Button { actions?.revertChange(rev: rev) } label: {
                Label(Localized.Menu.revertChange, systemImage: "arrow.uturn.backward")
            }
        } label: {
            Label(Localized.Menu.moreActions, systemImage: "ellipsis.circle")
        }
    }

    @ViewBuilder
    private func identifierCopySection(change: ChangeInfo) -> some View {
        Divider()
        Button { copyToPasteboard(change.changeId.id) } label: {
            Label(Localized.Menu.copyChangeId, systemImage: "doc.on.doc")
        }
        .keyboardShortcut("c", modifiers: [])
        Button { copyToPasteboard(change.commitId.id) } label: {
            Label(Localized.Menu.copyCommitId, systemImage: "doc.on.doc")
        }
        Divider()
    }

    private func fixButton(_ title: String, revs: [String], selectionCanFix: Bool = true) -> some View {
        Button { actions?.fix(revs: revs) } label: {
            Label(title, systemImage: "wand.and.stars")
        }
        .disabled(!selectionCanFix || actions?.fixUnavailableReason != nil)
        .help(
            actions?.fixUnavailableReason ?? (
                selectionCanFix
                    ? Localized.Menu.rewriteTheseChangesAndTheirDescendants
                    : Localized.Menu.immutableChangesCannotBeRewritten
            )
        )
    }

    private func abandonButton(entry: GraphEntry, rev: String) -> some View {
        Button(role: .destructive) { onRequest?(.abandon(rev: rev)) } label: {
            if entry.change.isDivergent {
                Label(Localized.Menu.abandonResolveDivergence, systemImage: "arrow.triangle.merge")
            } else {
                Label(Localized.Menu.abandon, systemImage: "trash")
            }
        }
        .keyboardShortcut("a", modifiers: [])
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
                        Localized.Menu.compareDivergentVersion,
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
                        Localized.Menu.compareDivergentVersions,
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
                    Localized.Menu.rebaseCountSelectedOntoThis(count: revisions.count),
                    systemImage: "arrow.uturn.up"
                )
            }
            .disabled(!viewModel.canRebaseSelection(onto: entry.change))
            .help(
                viewModel.canRebaseSelection(onto: entry.change)
                    ? Localized.Menu.rebaseTheSelectedChangesWhilePreserving
                    : Localized.Menu.rebaseRequiresMutableChangesAndA
            )
        } else if let sel = selectedId, sel != entry.change.changeId.id {
            Divider()
            let selRev = viewModel.selectedRevision(for: sel)
            Button { actions?.compareWith(from: selRev, to: rev) } label: {
                Label(Localized.Menu.compareWithSelected, systemImage: "arrow.left.arrow.right")
            }
            if let request = viewModel.bookmarkDiffRequest(from: sel, to: entry.change) {
                Button { actions?.diffBookmark(request) } label: {
                    Label(Localized.Menu.diffBookmark, systemImage: "arrow.left.arrow.right.circle")
                }
            }
            // Rebase and squash rewrite the selected change; this row is only the destination.
            let selectedImmutable = viewModel.change(for: sel)?.isImmutable ?? false
            if !selectedImmutable {
                Button { actions?.rebase(rev: selRev, dest: rev, mode: .source) } label: {
                    Label(Localized.Menu.rebaseSelectedOntoThis, systemImage: "arrow.uturn.up")
                }
                if !entry.change.isImmutable {
                    Button { actions?.squash(rev: selRev, into: rev) } label: {
                        Label(
                            Localized.Menu.squashSelectedIntoThis,
                            systemImage: "arrow.down.left.circle"
                        )
                    }
                }
            }
            let canMerge = viewModel.canMergeSelectedChange(with: entry.change)
            Button { actions?.merge(parents: [selRev, rev]) } label: {
                Label(Localized.Menu.mergeWithSelected, systemImage: "arrow.triangle.merge")
            }
            .disabled(!canMerge)
            .help(
                canMerge
                    ? Localized.Menu.createANewChangeWithBoth
                    : Localized.Menu.mergeRequiresIndependentHeads
            )
        }
    }
}
