import JayJayCore
import SwiftUI

struct BookmarkManagerView: View {
    let bookmarks: [BookmarkInfo]
    let actions: (any BookmarkActions)?
    let repo: JayJayRepo?
    let onCleanUp: () -> Void
    let onFilter: (String) -> Void
    let onDismiss: () -> Void

    @State private var filter = ""
    @State private var showDeleted = true

    private var filteredBookmarks: [BookmarkInfo] {
        bookmarks
            .filter { showDeleted || !$0.isDeleted }
            .filter { filter.isEmpty || $0.name.localizedCaseInsensitiveContains(filter) }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    private var activeCount: Int {
        bookmarks.filter { !$0.isDeleted }.count
    }

    private var deletedCount: Int {
        bookmarks.filter(\.isDeleted).count
    }

    private var conflictedCount: Int {
        bookmarks.filter(\.isConflicted).count
    }

    private var localOnlyCount: Int {
        bookmarks.filter { !$0.isTrackingRemote && !$0.isDeleted }.count
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            statsBar
            Divider()
            bookmarkList
        }
        .frame(width: 600, height: 480)
    }

    // MARK: - Header

    private var header: some View {
        HStack(spacing: 12) {
            Label("Bookmark Manager", systemImage: "bookmark.fill")
                .jayjayFont(15, weight: .semibold)
            Spacer()
            HStack(spacing: 4) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Filter bookmarks", text: $filter)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 180)
            }
            Button("Done", action: onDismiss)
                .keyboardShortcut(.cancelAction)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    // MARK: - Stats

    private var statsBar: some View {
        HStack(spacing: 16) {
            statBadge("\(activeCount) active", color: .green)
            if deletedCount > 0 {
                statBadge("\(deletedCount) deleted", color: .red)
            }
            if conflictedCount > 0 {
                statBadge("\(conflictedCount) conflicted", color: .orange)
            }
            if localOnlyCount > 0 {
                statBadge("\(localOnlyCount) local-only", color: .secondary)
            }
            Spacer()
            Toggle("Show deleted", isOn: $showDeleted)
                .jayjayFont(11)
                .toggleStyle(.checkbox)
            Button {
                onCleanUp()
            } label: {
                Label("Clean Up", systemImage: "trash")
                    .jayjayFont(11)
                    .foregroundStyle(.red)
            }
            .controlSize(.small)
            .help(
                "Fetch + prune remote refs, delete local git branches whose remote is gone, then forget stale jj bookmarks"
            )
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .background(Color.primary.opacity(0.02))
    }

    private func statBadge(_ text: String, color: Color) -> some View {
        Text(text)
            .jayjayFont(10, weight: .semibold)
            .foregroundStyle(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(color.opacity(0.1), in: Capsule())
    }

    // MARK: - List

    private var bookmarkList: some View {
        List {
            ForEach(filteredBookmarks, id: \.name) { bookmark in
                BookmarkManagerRow(
                    bookmark: bookmark,
                    onFilter: { onFilter(bookmark.name) },
                    onDelete: { actions?.deleteBookmark(name: bookmark.name) },
                    onForget: { actions?.deleteBookmark(name: bookmark.name) },
                    onPush: { actions?.gitPush(bookmark: bookmark.name) },
                    onResolve: {
                        try? repo?.moveBookmark(name: bookmark.name, toRev: "@-")
                        actions?.gitFetch() // refresh
                    }
                )
            }
        }
        .listStyle(.plain)
    }
}

// MARK: - Row

private struct BookmarkManagerRow: View {
    let bookmark: BookmarkInfo
    let onFilter: () -> Void
    let onDelete: () -> Void
    let onForget: () -> Void
    let onPush: () -> Void
    let onResolve: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            statusIcon
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(bookmark.name)
                        .jayjayFont(13, weight: .medium, design: .monospaced)
                        .lineLimit(1)
                    if bookmark.isDeleted {
                        badge("deleted", color: .red)
                    }
                    if bookmark.isConflicted {
                        badge("conflicted", color: .orange)
                    }
                    if !bookmark.isTrackingRemote, !bookmark.isDeleted {
                        badge("local", color: .secondary)
                    }
                }
                HStack(spacing: 6) {
                    if !bookmark.description.isEmpty {
                        Text(bookmark.description)
                            .jayjayFont(11)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                    if !bookmark.trackedRemotes.isEmpty {
                        Text(bookmark.trackedRemotes.joined(separator: ", "))
                            .jayjayFont(10, design: .monospaced)
                            .foregroundStyle(.tertiary)
                    }
                }
            }
            Spacer()
            if !bookmark.changeId.isEmpty {
                Text(String(bookmark.changeId.prefix(8)))
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 4)
        .contextMenu {
            if !bookmark.isDeleted, !bookmark.changeId.isEmpty {
                Button { onFilter() } label: {
                    Label("Filter in DAG", systemImage: "line.3.horizontal.decrease.circle")
                }
            }
            if bookmark.isConflicted {
                Button { onResolve() } label: {
                    Label("Resolve conflict (set to @)", systemImage: "arrow.triangle.merge")
                }
            }
            if bookmark.isTrackingRemote, !bookmark.isDeleted {
                Button { onPush() } label: {
                    Label("Push", systemImage: "arrow.up.circle")
                }
            }
            Divider()
            if bookmark.isDeleted {
                Button { onForget() } label: {
                    Label("Forget (remove from jj)", systemImage: "bookmark.slash")
                }
            } else {
                Button(role: .destructive) { onDelete() } label: {
                    Label("Delete", systemImage: "trash")
                }
            }
        }
    }

    private var statusIcon: some View {
        Group {
            if bookmark.isDeleted {
                Image(systemName: "bookmark.slash.fill")
                    .foregroundStyle(.red)
            } else if bookmark.isConflicted {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
            } else if bookmark.isTrackingRemote {
                Image(systemName: "cloud.fill")
                    .foregroundStyle(.blue)
            } else {
                Image(systemName: "bookmark.fill")
                    .foregroundStyle(.secondary)
            }
        }
        .jayjayFont(12)
        .frame(width: 16)
    }

    private func badge(_ text: String, color: Color) -> some View {
        Text(text)
            .jayjayFont(9, weight: .semibold)
            .foregroundStyle(color)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.12), in: Capsule())
    }
}
