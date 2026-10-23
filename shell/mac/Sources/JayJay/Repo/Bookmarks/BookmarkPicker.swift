import JayJayCore
import SwiftUI

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let isLoaded: Bool
    let actions: (any BookmarkActions)?

    private var localBookmarks: [BookmarkInfo] {
        bookmarks.filter { $0.hasLocalTarget && !$0.isDeleted }
    }

    private var bookmarkCount: Int {
        bookmarks.filter { bookmark in
            !bookmark.isDeleted || bookmark.availableRemotes.contains { !bookmark.trackedRemotes.contains($0) }
        }.count
    }

    private var trackedBookmarks: [BookmarkInfo] {
        localBookmarks
            .filter(\.isTrackingRemote)
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    private var localOnlyBookmarks: [BookmarkInfo] {
        localBookmarks
            .filter { !$0.isTrackingRemote }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    var sections: [PickerSection] {
        var sections: [PickerSection] = []
        if !trackedBookmarks.isEmpty {
            sections.append(PickerSection(id: "tracked", title: Localized.Repo.tracked, rows: trackedBookmarks.map(bookmarkRow)))
        }
        if !localOnlyBookmarks.isEmpty {
            sections.append(PickerSection(id: "local", title: Localized.Repo.localOnly, rows: localOnlyBookmarks.map(bookmarkRow)))
        }
        let remoteRows = bookmarks
            .filter { !$0.hasLocalTarget }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
            .flatMap { bookmark in
                bookmark.availableRemotes
                    .filter { !bookmark.trackedRemotes.contains($0) }
                    .sorted()
                    .map { remoteRow(bookmark, remote: $0) }
            }
        if !remoteRows.isEmpty {
            sections.append(PickerSection(id: "remote", title: Localized.Repo.remoteOnly, rows: remoteRows))
        }
        return sections
    }

    @State private var anchor = PickerAnchor()
    @State private var panel = PickerPanel()
    @State private var showingCreate = false
    @State private var newBookmarkName = ""
    @State private var renamingBookmark: String?
    @State private var renameNewName = ""

    var body: some View {
        Button(action: togglePanel) {
            HStack(spacing: 6) {
                Image(systemName: "bookmark")
                    .imageScale(.small)
                Text(Localized.Repo.bookmarks)
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
                if bookmarkCount > 0 {
                    Text("\(bookmarkCount)")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 5)
                        .background(Color.primary.opacity(0.07), in: Capsule())
                }
                Image(systemName: "chevron.down")
                    .imageScale(.small)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 6)
            .frame(height: 26)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .fixedSize()
        .background(PickerAnchorView(anchor: anchor))
        .accessibilityLabel(bookmarkCount == 0 ? Localized.Repo.bookmarks : Localized.Repo.bookmarksCount(count: bookmarkCount))
        .help(Localized.Repo.showABookmarksChangeOrManage)
        .popover(isPresented: $showingCreate) {
            createPopover
        }
        .popover(isPresented: .init(
            get: { renamingBookmark != nil },
            set: {
                if !$0 {
                    renamingBookmark = nil
                }
            }
        )) {
            renamePopover
        }
        // A panel opened while loading shows the bookmarks once they arrive.
        .onChange(of: isLoaded) { refreshVisiblePanel() }
        .onChange(of: bookmarks) { refreshVisiblePanel() }
    }

    private func togglePanel() {
        guard !panel.isVisible, !panel.wasJustDismissed else {
            panel.dismiss()
            return
        }
        showPanel()
    }

    private func refreshVisiblePanel() {
        if panel.isVisible {
            showPanel()
        }
    }

    private func showPanel() {
        guard let anchorView = anchor.view else { return }
        let sections = sections
        let root = PickerPanelRoot(
            placeholder: Localized.Repo.filter,
            actionLabel: Localized.Repo.new,
            onAction: {
                newBookmarkName = ""
                showingCreate = true
            },
            sections: sections,
            emptyText: Localized.Repo.noBookmarksYet,
            loadingText: isLoaded ? nil : Localized.Repo.loadingBookmarksEllipsis,
            onDismiss: { [weak panel] in panel?.dismiss() }
        )
        panel.show(under: anchorView, size: PickerPanelRoot.idealSize(sections: sections, width: 280), content: root)
    }

    private func bookmarkRow(_ bookmark: BookmarkInfo) -> PickerRow {
        let caption = BookmarkRowView.caption(for: bookmark)
        return PickerRow(
            id: "bookmark-\(bookmark.name)",
            searchText: ([bookmark.name] + bookmark.trackedRemotes + bookmark.availableRemotes).joined(separator: " "),
            height: caption == nil ? 28 : 38,
            action: target(bookmark, remote: nil).map { target in { actions?.revealBookmark(target) } },
            content: { _ in BookmarkRowView(bookmark: bookmark, caption: caption) }
        )
        .withContextMenu { bookmarkContextMenu(bookmark) }
    }

    private func remoteRow(_ bookmark: BookmarkInfo, remote: String) -> PickerRow {
        let name = bookmark.name
        let symbol = "\(name)@\(remote)"
        let target = target(bookmark, remote: remote)
        return PickerRow(
            id: "remote-bookmark-\(name.utf8.count):\(name)\(remote)",
            searchText: symbol,
            height: 28,
            action: target.map { target in { actions?.revealBookmark(target) } },
            content: { _ in BookmarkRowView(bookmark: bookmark, caption: nil, remote: remote) }
        )
        .withContextMenu {
            if let target {
                Button(Localized.Repo.filterByThisBookmark) {
                    panel.dismiss()
                    actions?.filterByBookmark(target)
                }
            }
            Button(Localized.Repo.trackSymbol(symbol: symbol)) {
                panel.dismiss()
                actions?.trackBookmark(name: name, remote: remote)
            }
        }
    }

    private func target(_ bookmark: BookmarkInfo, remote: String?) -> BookmarkFilterTarget? {
        let name = remote.map { "\(bookmark.name)@\($0)" } ?? bookmark.name
        return bookmarkFilterTargets(bookmarks: [bookmark]).first { $0.name == name }
    }

    @ViewBuilder
    private func bookmarkContextMenu(_ bookmark: BookmarkInfo) -> some View {
        let untrackedRemotes = bookmark.availableRemotes.filter { !bookmark.trackedRemotes.contains($0) }
        if let target = target(bookmark, remote: nil) {
            Button(Localized.Repo.filterByThisBookmark) {
                panel.dismiss()
                actions?.filterByBookmark(target)
            }
        }
        if bookmark.isTrackingRemote {
            Button {
                panel.dismiss()
                actions?.gitPullBookmark(name: bookmark.name)
            } label: {
                Label(Localized.Menu.pull, systemImage: "arrow.down.circle")
            }
        }
        Button {
            panel.dismiss()
            actions?.gitPush(bookmark: bookmark.name)
        } label: {
            Label(Localized.Menu.push, systemImage: "arrow.up.circle")
        }
        Button {
            panel.dismiss()
            actions?.moveBookmarkForward(name: bookmark.name)
        } label: {
            Label(Localized.Menu.moveTo, systemImage: "arrow.right.circle")
        }
        Button {
            panel.dismiss()
            renameNewName = bookmark.name
            renamingBookmark = bookmark.name
        } label: {
            Label(Localized.Menu.renameEllipsis, systemImage: "pencil")
        }

        if !bookmark.trackedRemotes.isEmpty {
            Text(Localized.Repo.trackingRemotes(remotes: bookmark.trackedRemotes.joined(separator: ", ")))
        }

        if !untrackedRemotes.isEmpty {
            Menu(Localized.Menu.trackRemote) {
                ForEach(untrackedRemotes, id: \.self) { remote in
                    Button(remote) {
                        panel.dismiss()
                        actions?.trackBookmark(name: bookmark.name, remote: remote)
                    }
                }
            }
        } else if bookmark.availableRemotes.isEmpty {
            Text(Localized.Repo.noRemoteBookmarkAvailable)
        }

        if canDeleteBookmark(name: bookmark.name, conflicted: bookmark.isConflicted) {
            Divider()
            Button(role: .destructive) {
                panel.dismiss()
                actions?.deleteBookmark(name: bookmark.name)
            } label: {
                Label(Localized.Repo.delete, systemImage: "trash")
            }
        }
    }

    private var createPopover: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(Localized.Repo.newBookmark)
                .jayjayFont(13, weight: .semibold)
            TextField(Localized.Repo.bookmarkName, text: $newBookmarkName)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(13, design: .monospaced)
                .frame(width: 220)
                .onSubmit { submitCreate() }
            HStack {
                Spacer()
                Button(Localized.Common.cancel) { showingCreate = false }
                    .keyboardShortcut(.cancelAction)
                Button(Localized.Repo.create) { submitCreate() }
                    .keyboardShortcut(.defaultAction)
                    .disabled(!canSubmitCreate)
            }
        }
        .padding(14)
    }

    private var renamePopover: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(Localized.Repo.renameBookmark)
                .jayjayFont(13, weight: .semibold)
            Text(Localized.Repo.renameFrom(name: renamingBookmark ?? ""))
                .jayjayFont(11, design: .monospaced)
                .foregroundStyle(.secondary)
            TextField(Localized.Repo.newName, text: $renameNewName)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(13, design: .monospaced)
                .frame(width: 220)
                .onSubmit { submitRename() }
            HStack {
                Spacer()
                Button(Localized.Common.cancel) { renamingBookmark = nil }
                    .keyboardShortcut(.cancelAction)
                Button(Localized.Repo.rename) { submitRename() }
                    .keyboardShortcut(.defaultAction)
                    .disabled(!canSubmitRename)
            }
        }
        .padding(14)
    }

    private var trimmedNewBookmarkName: String {
        newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var trimmedRenameName: String {
        renameNewName.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var canSubmitCreate: Bool {
        isValidBookmarkName(name: trimmedNewBookmarkName)
    }

    private var canSubmitRename: Bool {
        isValidBookmarkName(name: trimmedRenameName) && trimmedRenameName != renamingBookmark
    }

    private func submitCreate() {
        guard canSubmitCreate else { return }
        actions?.createBookmark(name: trimmedNewBookmarkName, rev: "@")
        showingCreate = false
    }

    private func submitRename() {
        guard canSubmitRename, let oldName = renamingBookmark else { return }
        actions?.renameBookmark(oldName: oldName, newName: trimmedRenameName)
        renamingBookmark = nil
    }
}
