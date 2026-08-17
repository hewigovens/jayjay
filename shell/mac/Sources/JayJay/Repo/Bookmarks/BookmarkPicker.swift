import JayJayCore
import SwiftUI

/// Toolbar bookmark switcher backed by the shared PickerPanel: filterable Tracked and Local Only sections, click filters the graph to that bookmark, right-click offers the bookmark actions.
struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let actions: (any BookmarkActions)?
    let onSelect: (String) -> Void

    private var localBookmarks: [BookmarkInfo] {
        bookmarks.filter(\.hasLocalTarget)
    }

    private var bookmarkLabel: String {
        let local = localBookmarks
        if local.isEmpty {
            return "Bookmarks"
        }
        let untrackedCount = local.filter { !$0.isTrackingRemote }.count
        if untrackedCount == 0 {
            return "Bookmarks (\(local.count))"
        }
        return "Bookmarks (\(local.count), \(untrackedCount) local)"
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
        var sections: [PickerSection] = []
        if !trackedBookmarks.isEmpty {
            sections.append(PickerSection(id: "tracked", title: "Tracked", rows: trackedBookmarks.map(bookmarkRow)))
        }
        if !localOnlyBookmarks.isEmpty {
            sections.append(PickerSection(id: "local", title: "Local Only", rows: localOnlyBookmarks.map(bookmarkRow)))
        }
        let root = PickerPanelRoot(
            placeholder: "Filter",
            actionLabel: "New Bookmark",
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
        let caption = remoteCaption(bookmark)
        return PickerRow(
            id: "bookmark-\(bookmark.name)",
            searchText: ([bookmark.name] + bookmark.trackedRemotes + bookmark.availableRemotes).joined(separator: " "),
            height: caption == nil ? 28 : 38,
            action: { onSelect(bookmark.name) },
            content: { _ in
                VStack(alignment: .leading, spacing: 1) {
                    HStack(spacing: 6) {
                        Text(bookmark.name)
                            .font(.system(size: 13))
                            .lineLimit(1)
                        Image(systemName: bookmark.isTrackingRemote ? "cloud.fill" : "cloud.slash")
                            .imageScale(.small)
                            .foregroundStyle(bookmark.isTrackingRemote ? .secondary : .tertiary)
                        Spacer(minLength: 8)
                    }
                    if let caption {
                        Text(caption)
                            .font(.system(size: 10))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }
                .padding(.horizontal, 14)
            }
        )
        .withContextMenu { bookmarkContextMenu(bookmark) }
    }

    private func remoteCaption(_ bookmark: BookmarkInfo) -> String? {
        if !bookmark.trackedRemotes.isEmpty {
            return bookmark.trackedRemotes.map { "@\($0)" }.joined(separator: ", ")
        }
        if !bookmark.availableRemotes.isEmpty {
            return "Remote available: \(bookmark.availableRemotes.map { "@\($0)" }.joined(separator: ", "))"
        }
        return nil
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
