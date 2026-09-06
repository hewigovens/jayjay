import JayJayCore
import SwiftUI

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let actions: (any BookmarkActions)?
    let onSelect: (String) -> Void

    private var localBookmarks: [BookmarkInfo] {
        bookmarks.filter { $0.hasLocalTarget && !$0.isDeleted }
    }

    private var bookmarkLabel: String {
        let local = localBookmarks
        let total = bookmarks.filter { bookmark in
            !bookmark.isDeleted || bookmark.availableRemotes.contains { !bookmark.trackedRemotes.contains($0) }
        }.count
        if total == 0 {
            return "Bookmarks"
        }
        let untrackedCount = local.filter { !$0.isTrackingRemote }.count
        if untrackedCount == 0 {
            return "Bookmarks (\(total))"
        }
        return "Bookmarks (\(total), \(untrackedCount) local)"
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
            sections.append(PickerSection(id: "tracked", title: "Tracked", rows: trackedBookmarks.map(bookmarkRow)))
        }
        if !localOnlyBookmarks.isEmpty {
            sections.append(PickerSection(id: "local", title: "Local Only", rows: localOnlyBookmarks.map(bookmarkRow)))
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
            sections.append(PickerSection(id: "remote", title: "Remote Only", rows: remoteRows))
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
            HStack(spacing: 4) {
                Image(systemName: "arrow.triangle.branch")
                    .imageScale(.small)
                Text(bookmarkLabel)
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .fixedSize()
        .background(PickerAnchorView(anchor: anchor))
        .help("Filter the graph by a bookmark, or manage bookmarks")
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
    }

    private func togglePanel() {
        guard !panel.isVisible, !panel.wasJustDismissed else {
            panel.dismiss()
            return
        }
        guard let anchorView = anchor.view else { return }
        let sections = sections
        let root = PickerPanelRoot(
            placeholder: "Filter",
            actionLabel: "New",
            onAction: {
                newBookmarkName = ""
                showingCreate = true
            },
            sections: sections,
            emptyText: "No bookmarks yet",
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
            action: { onSelect(bookmark.name) },
            content: { _ in BookmarkRowView(bookmark: bookmark, caption: caption) }
        )
        .withContextMenu { bookmarkContextMenu(bookmark) }
    }

    private func remoteRow(_ bookmark: BookmarkInfo, remote: String) -> PickerRow {
        let name = bookmark.name
        let symbol = "\(name)@\(remote)"
        let revset = "ancestors(remote_bookmarks(exact:\(RevsetExpressions.quotedSymbol(name)), exact:\(RevsetExpressions.quotedSymbol(remote))), \(RepoViewModel.defaultRevsetPageSize))"
        return PickerRow(
            id: "remote-bookmark-\(name.utf8.count):\(name)\(remote)",
            searchText: symbol,
            height: 28,
            action: { onSelect(revset) },
            content: { _ in BookmarkRowView(bookmark: bookmark, caption: nil, remote: remote) }
        )
        .withContextMenu {
            Button("Filter by this bookmark") {
                panel.dismiss()
                onSelect(revset)
            }
            Button("Track \(symbol)") {
                panel.dismiss()
                actions?.trackBookmark(name: name, remote: remote)
            }
        }
    }

    @ViewBuilder
    private func bookmarkContextMenu(_ bookmark: BookmarkInfo) -> some View {
        let untrackedRemotes = bookmark.availableRemotes.filter { !bookmark.trackedRemotes.contains($0) }
        Button("Filter by this bookmark") {
            panel.dismiss()
            onSelect(bookmark.name)
        }
        if bookmark.isTrackingRemote {
            Button {
                panel.dismiss()
                actions?.gitPullBookmark(name: bookmark.name)
            } label: {
                Label("Pull", systemImage: "arrow.down.circle")
            }
        }
        Button {
            panel.dismiss()
            actions?.gitPush(bookmark: bookmark.name)
        } label: {
            Label("Push", systemImage: "arrow.up.circle")
        }
        Button {
            panel.dismiss()
            actions?.moveBookmarkForward(name: bookmark.name)
        } label: {
            Label("Move to @-", systemImage: "arrow.right.circle")
        }
        Button {
            panel.dismiss()
            renameNewName = bookmark.name
            renamingBookmark = bookmark.name
        } label: {
            Label("Rename...", systemImage: "pencil")
        }

        if !bookmark.trackedRemotes.isEmpty {
            Text("Tracking \(bookmark.trackedRemotes.joined(separator: ", "))")
        }

        if !untrackedRemotes.isEmpty {
            Menu("Track Remote") {
                ForEach(untrackedRemotes, id: \.self) { remote in
                    Button(remote) {
                        panel.dismiss()
                        actions?.trackBookmark(name: bookmark.name, remote: remote)
                    }
                }
            }
        } else if bookmark.availableRemotes.isEmpty {
            Text("No remote bookmark available")
        }

        Divider()
        Button(role: .destructive) {
            panel.dismiss()
            actions?.deleteBookmark(name: bookmark.name)
        } label: {
            Label("Delete", systemImage: "trash")
        }
    }

    private var createPopover: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("New Bookmark")
                .jayjayFont(13, weight: .semibold)
            TextField("Bookmark name", text: $newBookmarkName)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(13, design: .monospaced)
                .frame(width: 220)
                .onSubmit { submitCreate() }
            HStack {
                Spacer()
                Button("Cancel") { showingCreate = false }
                    .keyboardShortcut(.cancelAction)
                Button("Create") { submitCreate() }
                    .keyboardShortcut(.defaultAction)
                    .disabled(trimmedNewBookmarkName.isEmpty)
            }
        }
        .padding(14)
    }

    private var renamePopover: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Rename Bookmark")
                .jayjayFont(13, weight: .semibold)
            Text("From: \(renamingBookmark ?? "")")
                .jayjayFont(11, design: .monospaced)
                .foregroundStyle(.secondary)
            TextField("New name", text: $renameNewName)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(13, design: .monospaced)
                .frame(width: 220)
                .onSubmit { submitRename() }
            HStack {
                Spacer()
                Button("Cancel") { renamingBookmark = nil }
                    .keyboardShortcut(.cancelAction)
                Button("Rename") { submitRename() }
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

    private var canSubmitRename: Bool {
        !trimmedRenameName.isEmpty && trimmedRenameName != renamingBookmark
    }

    private func submitCreate() {
        guard !trimmedNewBookmarkName.isEmpty else { return }
        actions?.createBookmark(name: trimmedNewBookmarkName, rev: "@")
        showingCreate = false
    }

    private func submitRename() {
        guard canSubmitRename, let oldName = renamingBookmark else { return }
        actions?.renameBookmark(oldName: oldName, newName: trimmedRenameName)
        renamingBookmark = nil
    }
}
