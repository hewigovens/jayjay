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
        ForEach(selectionCommands(), id: \.self) { command in
            if command == .abandon {
                bisectBetweenButton(viewModel)
                Divider()
            }
            selectionCommandItem(command, revisions: revisions, viewModel: viewModel)
        }
    }

    @ViewBuilder
    private func selectionCommandItem(_ command: SelectionCommand, revisions: [String], viewModel: DAGViewModel) -> some View {
        let available = viewModel.capabilities.isAvailable(command)
        let fixUnavailableReason = command == .fix ? actions?.fixUnavailableReason : nil
        if command == .merge, available {
            Menu {
                Section(Localized.Menu.firstParent) {
                    ForEach(viewModel.mergeParentChoices, id: \.parents) { choice in
                        Button(choice.label) { command.perform(on: choice.parents, actions: actions, onRequest: onRequest) }
                    }
                }
            } label: {
                Label(command.title(count: revisions.count), systemImage: command.systemImage)
            }
            .help(selectionCommandHelp(command: command, available: true))
        } else {
            Button(role: command == .abandon ? .destructive : nil) {
                command.perform(on: revisions, actions: actions, onRequest: onRequest)
            } label: {
                Label(command.title(count: revisions.count), systemImage: command.systemImage)
            }
            .keyboardShortcut(command.accelerator.map { KeyboardShortcut($0, modifiers: []) })
            .disabled(!available || fixUnavailableReason != nil)
            .help(fixUnavailableReason ?? selectionCommandHelp(command: command, available: available))
        }
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
        let available = availableChangeCommands(change: entry.change)
        ForEach([ChangeCommand.newChange, .newChangeBefore, .newChangeAfter], id: \.self) { command in
            commandButton(command, change: entry.change, available: available)
        }
        Divider()
        ForEach([ChangeCommand.edit, .describe, .squashIntoParent, .moveChangesToWorkingCopy, .rebaseOntoTrunk, .fix], id: \.self) { command in
            commandButton(command, change: entry.change, available: available)
        }

        divergentCompareSection(entry: entry, rev: rev, viewModel: viewModel)
        selectionActionsSection(entry: entry, rev: rev, viewModel: viewModel)

        Divider()
        commandButton(.createBookmark, change: entry.change, available: available)
        commandButton(.createTag, change: entry.change, available: available)
        if !entry.change.isImmutable {
            reviewActionsSection(rev: rev)
        }
        Divider()
        bisectFromButton(entry)
        ForEach(entry.change.tags, id: \.self) { name in
            Menu(Localized.Repo.tagNamed(name: name)) {
                TagMenuItems(name: name, isOnRemote: remoteTagNames.contains(name), actions: actions, onRequest: onRequest)
            }
        }
        Button { onRequest?(.showAncestors(changeId: entry.change.changeId.id, commitId: entry.change.commitId.id)) } label: {
            Label(Localized.Menu.showAncestorsEllipsis, systemImage: "arrow.triangle.branch")
        }
        commandButton(.showEvolution, change: entry.change, available: available)

        identifierCopySection(change: entry.change)
        Menu {
            ForEach([ChangeCommand.duplicate, .absorb, .revert], id: \.self) { command in
                commandButton(command, change: entry.change, available: available)
            }
        } label: {
            Label(Localized.Menu.moreActions, systemImage: "ellipsis.circle")
        }
        if available.contains(.abandon) {
            Divider()
            commandButton(.abandon, change: entry.change, available: available)
        }
    }

    @ViewBuilder
    private func commandButton(_ command: ChangeCommand, change: ChangeInfo, available: [ChangeCommand]) -> some View {
        if available.contains(command) {
            let resolvesDivergence = command == .abandon && change.isDivergent
            let fixUnavailableReason = command == .fix ? actions?.fixUnavailableReason : nil
            Button(role: command == .abandon ? .destructive : nil) {
                command.perform(on: change, actions: actions, onRequest: onRequest)
            } label: {
                Label(
                    resolvesDivergence ? Localized.Menu.abandonResolveDivergence : command.title,
                    systemImage: resolvesDivergence ? "arrow.triangle.merge" : command.systemImage
                )
            }
            .keyboardShortcut(command.accelerator.map { KeyboardShortcut($0, modifiers: []) })
            .disabled(fixUnavailableReason != nil)
            .help(command == .fix ? fixUnavailableReason ?? Localized.Menu.rewriteTheseChangesAndTheirDescendants : "")
        }
    }

    @ViewBuilder
    private func reviewActionsSection(rev: String) -> some View {
        Button { onRequest?(.createStackedPRs(rev: rev)) } label: {
            Label(
                Localized.Menu.createUpdateStackedPrsEllipsis,
                systemImage: "square.stack.3d.up.fill"
            )
        }
        if let uploadTitle {
            Button { actions?.upload(rev: rev) } label: {
                Label(uploadTitle, systemImage: "icloud.and.arrow.up")
            }
        }
        if let gerritUpdateTitle {
            Button { actions?.updateFromGerrit(rev: rev) } label: {
                Label(gerritUpdateTitle, systemImage: "icloud.and.arrow.down")
            }
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

    private func copyToPasteboard(_ value: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(value, forType: .string)
    }

    @ViewBuilder
    private func divergentCompareSection(
        entry: GraphEntry, rev: String, viewModel: DAGViewModel
    ) -> some View {
        if entry.change.isDivergent {
            Divider()
            Button { onRequest?(.converge(rev: entry.change.changeId.id)) } label: {
                Label(Localized.Converge.menu, systemImage: "arrow.triangle.merge")
            }
        }
        let divergentSiblings = viewModel.divergentSiblings(of: entry.change)
        if !divergentSiblings.isEmpty {
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
            Button {
                onRequest?(.rebaseCommand(.selection(revisions, onto: rev, destLabel: DAGRebaseGesturePolicy.displayLabel(for: entry.change))))
            } label: {
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
        } else if let sel = selectedId, sel != entry.change.selectionRevision {
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
                Button {
                    onRequest?(.rebaseCommand(RepoRebaseCommand(
                        sourceRev: selRev,
                        sourceLabel: viewModel.change(for: sel).map(DAGRebaseGesturePolicy.displayLabel(for:)) ?? selRev,
                        destRev: rev,
                        destLabel: DAGRebaseGesturePolicy.displayLabel(for: entry.change)
                    )))
                } label: {
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
