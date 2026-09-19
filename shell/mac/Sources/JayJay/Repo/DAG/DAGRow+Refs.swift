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
            case .conflict: tag("conflict", tint: .red.opacity(0.18))
            case .divergent: tag("divergent", tint: FileStatusColors.modified.opacity(0.18))
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
                    .jayjayFont(9, weight: .semibold)
                    .imageScale(.small)
                    .foregroundStyle(iconColor ?? .secondary)
            }
            Text(title).jayjayFont(9, weight: .semibold)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .padding(.horizontal, 5).padding(.vertical, 2)
        .background(tint, in: Capsule())
    }

    private func workingCopyTag() -> some View {
        tag("@", tint: .accentColor.opacity(0.18))
            .help("Working copy — drag onto a change to move it here")
            .gesture(
                DragGesture(minimumDistance: 0, coordinateSpace: .named(DAGRebaseCoordinateSpace.name))
                    .onChanged { onBookmarkDragChanged?(workingCopyDragLabel, change.commitId.id, $0) }
                    .onEnded { onBookmarkDragEnded?(workingCopyDragLabel, $0) }
            )
    }

    private func bookmarkTag(_ name: String) -> some View {
        let conflicted = conflictedBookmarkNames.contains(name)
        return tag(
            name,
            tint: conflicted ? .orange.opacity(0.18) : .primary.opacity(0.08),
            systemImage: conflicted ? "exclamationmark.triangle.fill" : "bookmark",
            iconColor: conflicted ? .orange : .green
        )
        .help(
            conflicted
                ? "Conflicted bookmark: \(name) — points at more than one change. Drag onto a change to set it there."
                : "Bookmark: \(name) — drag onto a change to move it"
        )
        .accessibilityLabel(conflicted ? "Conflicted bookmark \(name)" : "Bookmark \(name)")
        .accessibilityIdentifier(AID.DAG.bookmark(name))
        .contextMenu {
            Button(conflicted ? "Resolve conflict (set to @)" : "Move to @-") {
                actions?.moveBookmark(name: name, toRev: conflicted ? "@" : "@-")
            }
            Button("Push") {
                actions?.gitPush(bookmark: name)
            }
            if !isTrunkBookmark(name: name) {
                Button(pullRequestLabel) {
                    actions?.openPR(bookmark: name)
                }
            }
            Divider()
            Button("Copy Bookmark Name") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(name, forType: .string)
            }
            if canRemoveBookmarkFromChip(name: name, conflicted: conflicted) {
                Divider()
                Button(conflicted ? "Remove from This Change" : "Delete Bookmark", role: .destructive) {
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
        let chip = tag("\(name)@", tint: .accentColor.opacity(0.10))
            .help("Working copy of the \(name) workspace")
            .accessibilityLabel("Workspace \(name)")
        if let workspace = workspacesByName[name] {
            chip.contextMenu {
                WorkspaceMenuItems(workspace: workspace) { onRequest?(.openWorkspace(workspace)) }
            }
        } else {
            chip
        }
    }

    private func gitTag(_ name: String) -> some View {
        tag(name, tint: .primary.opacity(0.08), systemImage: "tag", iconColor: .blue)
            .help("Tag: \(name)")
            .accessibilityLabel("Tag \(name)")
            .contextMenu {
                Button("Copy Tag Name") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(name, forType: .string)
                }
            }
    }

    private var changeIdText: Text {
        Text(change.changeId.highlighted(
            scheme: colorScheme,
            font: fontFamily.identifierFont(baseSize: baseFontSize)
        ))
    }

    private var pullRequestLabel: String {
        if let prHostName {
            return "Pull Request on \(prHostName)"
        }
        return "Pull Request"
    }
}
