import JayJayCore
import SwiftUI

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let actions: (any BookmarkActions)?
    let onSelect: (String) -> Void

    private var bookmarkLabel: String {
        if bookmarks.isEmpty {
            return "Bookmarks"
        }
        let untrackedCount = bookmarks.filter { !$0.isTrackingRemote }.count
        if untrackedCount == 0 {
            return "Bookmarks (\(bookmarks.count))"
        }
        return "Bookmarks (\(bookmarks.count), \(untrackedCount) local)"
    }

    private var trackedBookmarks: [BookmarkInfo] {
        bookmarks
            .filter(\.isTrackingRemote)
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    private var localOnlyBookmarks: [BookmarkInfo] {
        bookmarks
            .filter { !$0.isTrackingRemote }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    @State private var showingCreate = false
    @State private var newBookmarkName = ""
    @State private var renamingBookmark: String?
    @State private var renameNewName = ""

    var body: some View {
        Menu {
            if !trackedBookmarks.isEmpty {
                Section("Tracked") {
                    ForEach(trackedBookmarks, id: \.name) { bookmark in
                        bookmarkMenu(bookmark)
                    }
                }
            }

            if !localOnlyBookmarks.isEmpty {
                Section("Local Only") {
                    ForEach(localOnlyBookmarks, id: \.name) { bookmark in
                        bookmarkMenu(bookmark)
                    }
                }
            }

            Button("New Bookmark...") {
                newBookmarkName = ""
                showingCreate = true
            }

        } label: {
            HStack(spacing: 4) {
                Image(systemName: "arrow.triangle.branch")
                    .imageScale(.small)
                Text(bookmarkLabel)
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
            }
        }
        .menuStyle(.borderlessButton)
        .popover(isPresented: $showingCreate) {
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
                        .disabled(newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
            .padding(14)
        }
        .popover(isPresented: .init(
            get: { renamingBookmark != nil },
            set: { if !$0 { renamingBookmark = nil } }
        )) {
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
                        .disabled({
                            let n = renameNewName.trimmingCharacters(in: .whitespacesAndNewlines)
                            return n.isEmpty || n == renamingBookmark
                        }())
                }
            }
            .padding(14)
        }
    }

    @ViewBuilder
    private func bookmarkMenu(_ bookmark: BookmarkInfo) -> some View {
        let untrackedRemotes = bookmark.availableRemotes.filter { !bookmark.trackedRemotes.contains($0) }
        Menu {
            Button("Filter by this bookmark") {
                onSelect(bookmark.name)
            }
            if bookmark.isTrackingRemote {
                Button {
                    actions?.gitPullBookmark(name: bookmark.name)
                } label: {
                    Label("Pull", systemImage: "arrow.down.circle")
                }
            }
            Button {
                actions?.gitPush(bookmark: bookmark.name)
            } label: {
                Label("Push", systemImage: "arrow.up.circle")
            }
            Button {
                actions?.moveBookmarkForward(name: bookmark.name)
            } label: {
                Label("Move to @-", systemImage: "arrow.right.circle")
            }
            Button {
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
                            actions?.trackBookmark(name: bookmark.name, remote: remote)
                        }
                    }
                }
            } else if bookmark.availableRemotes.isEmpty {
                Text("No remote bookmark available")
            }

            Divider()
            Button(role: .destructive) {
                actions?.deleteBookmark(name: bookmark.name)
            } label: {
                Label("Delete", systemImage: "trash")
            }
        } label: {
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(bookmark.name)
                    Image(systemName: bookmark.isTrackingRemote ? "cloud.fill" : "cloud.slash")
                        .foregroundStyle(bookmark.isTrackingRemote ? .secondary : .tertiary)
                }
                if !bookmark.trackedRemotes.isEmpty {
                    Text(bookmark.trackedRemotes.map { "@\($0)" }.joined(separator: ", "))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else if !bookmark.availableRemotes.isEmpty {
                    Text("Remote available: \(bookmark.availableRemotes.map { "@\($0)" }.joined(separator: ", "))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    private func submitCreate() {
        let name = newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        actions?.createBookmark(name: name, rev: "@")
        showingCreate = false
    }

    private func submitRename() {
        let newName = renameNewName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !newName.isEmpty, let oldName = renamingBookmark, newName != oldName else { return }
        actions?.renameBookmark(oldName: oldName, newName: newName)
        renamingBookmark = nil
    }
}
