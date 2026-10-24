import AppKit
import JayJayCore
import SwiftUI

extension DAGRow {
    /// Longest chip line that fits the pane: the change id always stays, the chips that do not fit spill into a trailing `+N`, and as a last resort the first chip truncates instead of vanishing.
    var refsRow: some View {
        let chips = DAGRefChip.chips(for: change)
        return ViewThatFits(in: .horizontal) {
            refsLine(chips, visible: chips.count)
            refsLine(chips, visible: 4)
            refsLine(chips, visible: 3)
            refsLine(chips, visible: 2)
            refsLine(chips, visible: 1)
            refsLine(chips, visible: 1, truncatesFirst: true)
        }
    }

    private func refsLine(_ chips: [DAGRefChip], visible: Int, truncatesFirst: Bool = false) -> some View {
        let hidden = chips.dropFirst(visible)
        return HStack(alignment: .firstTextBaseline, spacing: 4) {
            changeIdText
                .lineLimit(1)
                .layoutPriority(1)
            ForEach(Array(chips.prefix(visible).enumerated()), id: \.element) { index, chip in
                chipView(chip)
                    .fixedSize(horizontal: !(truncatesFirst && index == 0), vertical: false)
            }
            if !hidden.isEmpty {
                tag("+\(hidden.count)", tint: .primary.opacity(0.05))
                    .help(hidden.map(\.label).joined(separator: ", "))
                    .fixedSize(horizontal: true, vertical: false)
            }
        }
    }

    @ViewBuilder
    private func chipView(_ chip: DAGRefChip) -> some View {
        switch chip {
            case .workingCopy: workingCopyTag()
            case .conflict: tag(Localized.Repo.conflictBadge, tint: .red.opacity(0.18))
            case .divergent: tag(Localized.Repo.divergentBadge, tint: .orange.opacity(0.18))
            case let .bookmark(name): bookmarkTag(name)
            case let .gitTag(name): gitTag(name)
            case let .workspace(name): workspaceChip(name)
        }
    }

    private func tag(_ title: String, tint: Color, systemImage: String? = nil, iconColor: Color? = nil) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 3) {
            if let systemImage {
                // An SF Symbol's box is taller than the label's line box, by a different amount per symbol; the small scale keeps every chip one height.
                Image(systemName: systemImage)
                    .jayjayFont(10, weight: .semibold)
                    .imageScale(.small)
                    .foregroundStyle(iconColor ?? .secondary)
            }
            Text(title).jayjayFont(10, weight: .semibold)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .padding(.horizontal, 5).padding(.vertical, 2)
        .background(tint, in: Capsule())
    }

    private func workingCopyTag() -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 4) {
            tag("@", tint: AppColors.workspace(colorScheme).opacity(colorScheme == .dark ? 0.15 : 0.18))
                .foregroundStyle(AppColors.workspace(colorScheme))
                .accessibilityLabel(Localized.Repo.workingCopyLabel)
                .help(Localized.Repo.workingCopyDragOntoAChange)
                .gesture(
                    DragGesture(minimumDistance: 0, coordinateSpace: .named(DAGRebaseCoordinateSpace.name))
                        .onChanged { onBookmarkDragChanged?(workingCopyDragLabel, change.commitId.id, $0) }
                        .onEnded { onBookmarkDragEnded?(workingCopyDragLabel, $0) }
                )
            if refreshMode == .updateWorkspace {
                Image(systemName: "exclamationmark.triangle.fill")
                    .jayjayFont(10, weight: .semibold)
                    .foregroundStyle(.orange)
                    .help(Localized.Repo.workingCopyIsStaleAnotherWorkspace)
                    .accessibilityIdentifier(AID.DAG.staleWorkingCopy)
            }
        }
    }

    private func bookmarkTag(_ name: String) -> some View {
        let conflicted = conflictedBookmarkNames.contains(name)
        return tag(
            name,
            tint: conflicted ? .orange.opacity(0.18)
                : AppColors.bookmark(colorScheme).opacity(colorScheme == .dark ? 0.16 : 0.14),
            systemImage: conflicted ? "exclamationmark.triangle.fill" : "bookmark",
            iconColor: conflicted ? .orange : AppColors.bookmark(colorScheme)
        )
        .help(
            conflicted
                ? Localized.Repo.conflictedBookmarkDragHelp(name: name)
                : Localized.Repo.bookmarkDragHelp(name: name)
        )
        .accessibilityLabel(conflicted ? Localized.Repo.conflictedBookmarkLabel(name: name) : Localized.Repo.bookmarkLabel(name: name))
        .accessibilityIdentifier(AID.DAG.bookmark(name))
        .selectionContextMenuExclusion()
        .contextMenu {
            Button(conflicted ? Localized.Menu.resolveConflictSetTo : Localized.Menu.moveTo) {
                actions?.moveBookmark(name: name, toRev: conflicted ? "@" : "@-")
            }
            Button(Localized.Menu.push) {
                actions?.gitPush(bookmark: name)
            }
            if !isTrunkBookmark(name: name) {
                Button(pullRequestLabel) {
                    actions?.openPR(bookmark: name)
                }
            }
            Divider()
            Button(Localized.Menu.copyBookmarkName) {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(name, forType: .string)
            }
            if canRemoveBookmarkFromChip(name: name, conflicted: conflicted) {
                Divider()
                Button(conflicted ? Localized.Menu.removeFromThisChange : Localized.Menu.deleteBookmark, role: .destructive) {
                    actions?.removeBookmark(name: name, fromRev: change.commitId.id)
                }
            }
        }
        .gesture(
            DragGesture(minimumDistance: 0, coordinateSpace: .named(DAGRebaseCoordinateSpace.name))
                .onChanged { onBookmarkDragChanged?(name, change.commitId.id, $0) }
                .onEnded { onBookmarkDragEnded?(name, $0) }
        )
    }

    @ViewBuilder
    private func workspaceChip(_ name: String) -> some View {
        let chip = tag("\(name)@", tint: AppColors.workspace(colorScheme).opacity(colorScheme == .dark ? 0.15 : 0.16))
            .foregroundStyle(AppColors.workspace(colorScheme))
            .help(Localized.Repo.workspaceWorkingCopy(name: name))
            .accessibilityLabel(Localized.Repo.workspaceLabel(name: name))
        chip.selectionContextMenuExclusion().contextMenu {
            if let workspace = workspacesByName[name] {
                WorkspaceMenuItems(
                    workspace: workspace,
                    onFilter: { actions?.filterByWorkspace(name: name) },
                    onOpen: { onRequest?(.openWorkspace(workspace)) }
                )
            } else {
                Button(Localized.Menu.filterByThisWorkspace) { actions?.filterByWorkspace(name: name) }
            }
        }
    }

    private func gitTag(_ name: String) -> some View {
        tag(
            name,
            tint: AppColors.commitIdPrefix(colorScheme).opacity(colorScheme == .dark ? 0.16 : 0.12),
            systemImage: "tag",
            iconColor: AppColors.commitIdPrefix(colorScheme)
        )
        .help(Localized.Repo.tagNamed(name: name))
        .accessibilityLabel(Localized.Repo.tagLabel(name: name))
        .selectionContextMenuExclusion()
        .contextMenu {
            TagMenuItems(name: name, isOnRemote: remoteTagNames.contains(name), actions: actions, onRequest: onRequest)
        }
    }

    private var changeIdText: Text {
        Text(change.changeId.highlighted(
            scheme: colorScheme,
            font: typography.identifierFont
        ))
    }

    private var pullRequestLabel: String {
        if let prHostName {
            return Localized.Repo.pullRequestOn(host: prHostName)
        }
        return Localized.Menu.pullRequest
    }
}
