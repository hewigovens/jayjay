import JayJayCore
import SwiftUI

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
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
            HStack(spacing: 6) {
                Image(systemName: "bookmark")
                    .imageScale(.small)
                Text("Bookmarks")
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
        .accessibilityLabel(bookmarkCount == 0 ? "Bookmarks" : "Bookmarks (\(bookmarkCount))")
        .help("Show a bookmark's change, or manage bookmarks")
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
                Button("Filter by This Bookmark") {
                    panel.dismiss()
                    actions?.filterByBookmark(target)
                }
            }
            Button("Track \(symbol)") {
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
            Button("Filter by This Bookmark") {
                panel.dismiss()
                actions?.filterByBookmark(target)
            }
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

        if canDeleteBookmark(name: bookmark.name, conflicted: bookmark.isConflicted) {
            Divider()
            Button(role: .destructive) {
                panel.dismiss()
                actions?.deleteBookmark(name: bookmark.name)
            } label: {
                Label("Delete", systemImage: "trash")
            }
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
