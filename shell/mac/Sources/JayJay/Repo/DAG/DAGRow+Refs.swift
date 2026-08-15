import AppKit
import JayJayCore
import SwiftUI

extension DAGRow {
    var refsRow: some View {
        HStack(spacing: 4) {
            changeIdText
                .jayjayFont(11, weight: .semibold, design: .monospaced)
                .lineLimit(1)
            if isDisplayedWorkingCopy { workingCopyTag() }
            ForEach(workspaceNames, id: \.self) { name in
                workspaceTag(name)
            }
            if change.hasConflict { tag("conflict", tint: .red.opacity(0.18)) }
            if change.isDivergent { tag("divergent", tint: FileStatusColors.modified.opacity(0.18)) }
            ForEach(change.bookmarks.prefix(3), id: \.self) {
                bookmarkTag($0)
            }
            if change.bookmarks.count > 3 {
                tag("+\(change.bookmarks.count - 3)", tint: .primary.opacity(0.05))
                    .help(change.bookmarks.joined(separator: ", "))
            }
            ForEach(change.tags.prefix(3), id: \.self) {
                gitTag($0)
            }
            if change.tags.count > 3 {
                tag("+\(change.tags.count - 3)", tint: .primary.opacity(0.05))
                    .help(change.tags.joined(separator: ", "))
            }
        }
    }

    private func tag(_ title: String, tint: Color, systemImage: String? = nil, iconColor: Color? = nil) -> some View {
        HStack(spacing: 3) {
            if let systemImage {
                Image(systemName: systemImage)
                    .jayjayFont(9, weight: .semibold)
                    .foregroundStyle(iconColor ?? .secondary)
            }
            Text(title).jayjayFont(9, weight: .semibold)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .padding(.horizontal, 5).padding(.vertical, 2)
        .background(tint, in: Capsule())
        .fixedSize(horizontal: true, vertical: false)
    }

    private func workspaceTag(_ name: String) -> some View {
        tag(name, tint: .accentColor.opacity(0.14), systemImage: "square.on.square")
            .help("Workspace \(name) is checked out here")
            .accessibilityLabel("Workspace \(name)")
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
        tag(name, tint: .primary.opacity(0.08), systemImage: "bookmark", iconColor: .green)
            .help("Bookmark: \(name) — drag onto a change to move it")
            .accessibilityLabel("Bookmark \(name)")
            .contextMenu {
                Button("Move to @-") {
                    onMoveBookmarkForward?(name)
                }
                Button("Push") {
                    onPushBookmark?(name)
                }
                if !isTrunkBookmark(name) {
                    Button(pullRequestLabel) {
                        onOpenPRForBookmark?(name)
                    }
                }
                Divider()
                Button("Copy Bookmark Name") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(name, forType: .string)
                }
                if !isTrunkBookmark(name) {
                    Divider()
                    Button("Delete Bookmark", role: .destructive) {
                        onDeleteBookmark?(name)
                    }
                }
            }
            .gesture(
                DragGesture(minimumDistance: 0, coordinateSpace: .named(DAGRebaseCoordinateSpace.name))
                    .onChanged { onBookmarkDragChanged?(name, change.commitId.id, $0) }
                    .onEnded { onBookmarkDragEnded?(name, $0) }
            )
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

    /// Change id with its shortest unique prefix highlighted, the remainder dimmed.
    private var changeIdText: Text {
        Text(change.changeId.highlighted(scheme: colorScheme))
    }

    private var pullRequestLabel: String {
        if let prHostName {
            return "Pull Request on \(prHostName)"
        }
        return "Pull Request"
    }
}
