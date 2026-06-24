import JayJayCore
import SwiftUI

struct BookmarkManagerView: View {
    let bookmarks: [BookmarkInfo]
    let actions: (any BookmarkActions)?
    let repo: JayJayRepo?
    let prHostName: String?
    let onFilter: (String) -> Void
    let onDiffBookmark: (BookmarkDiffRequest) -> Void
    let onDismiss: () -> Void

    @State private var filter = ""
    @State private var showDeleted = false

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
        bookmarks.filter { $0.hasLocalTarget && !$0.isTrackingRemote && !$0.isDeleted }.count
    }

    private var remoteOnlyCount: Int {
        bookmarks.filter { !$0.hasLocalTarget && !$0.isDeleted }.count
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            statsBar
            Divider()
            bookmarkList
            Divider()
            footer
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
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    // MARK: - Footer

    private var footer: some View {
        HStack {
            Spacer()
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
            if remoteOnlyCount > 0 {
                statBadge("\(remoteOnlyCount) remote-only", color: .blue)
            }
            Spacer()
            Toggle("Show deleted", isOn: $showDeleted)
                .jayjayFont(11)
                .toggleStyle(.checkbox)
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
                    prHostName: prHostName,
                    actions: self
                )
            }
        }
        .listStyle(.plain)
    }
}

extension BookmarkManagerView: BookmarkManagerRowActions {
    func filterBookmark(_ bookmark: BookmarkInfo) {
        let endpoint = RevsetExpressions.bookmarkEndpoint(for: bookmark)
        onFilter(endpoint.rev)
    }

    func diffBookmark(_ bookmark: BookmarkInfo) {
        let endpoint = RevsetExpressions.bookmarkEndpoint(for: bookmark)
        onDiffBookmark(RevsetExpressions.bookmarkDiffRequest(head: endpoint))
    }

    func deleteBookmark(_ bookmark: BookmarkInfo) {
        actions?.deleteBookmark(name: bookmark.name)
    }

    func forgetBookmark(_ bookmark: BookmarkInfo) {
        actions?.forgetBookmark(name: bookmark.name)
    }

    func pushBookmark(_ bookmark: BookmarkInfo) {
        actions?.gitPush(bookmark: bookmark.name)
    }

    func resolveBookmarkConflict(_ bookmark: BookmarkInfo) {
        try? repo?.moveBookmark(name: bookmark.name, toRev: "@-")
        actions?.gitFetch()
    }

    func openPullRequest(for bookmark: BookmarkInfo) {
        actions?.openPR(bookmark: bookmark.name)
    }

    func trackBookmark(_ bookmark: BookmarkInfo, remote: String) {
        actions?.trackBookmark(name: bookmark.name, remote: remote)
    }
}
