import SwiftUI
import JayJayBindings

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let onSelect: (String) -> Void
    let onCreate: (String) -> Void
    let onDelete: (String) -> Void
    var onPush: ((String) -> Void)?
    var onFetch: (() -> Void)?
    var onMoveForward: ((String) -> Void)?
    var onRename: ((String, String) -> Void)?
    var onTrack: ((String) -> Void)?

    @State private var showingCreate = false
    @State private var newBookmarkName = ""
    @State private var renamingBookmark: String?
    @State private var renameNewName = ""

    var body: some View {
        Menu {
            if !bookmarks.isEmpty {
                ForEach(bookmarks, id: \.name) { bookmark in
                    Menu {
                        Button("Filter by this bookmark") {
                            onSelect(bookmark.name)
                        }
                        Button {
                            onPush?(bookmark.name)
                        } label: {
                            Label("Push", systemImage: "arrow.up.circle")
                        }
                        Button {
                            onMoveForward?(bookmark.name)
                        } label: {
                            Label("Move to @-", systemImage: "arrow.right.circle")
                        }
                        Button {
                            renameNewName = bookmark.name
                            renamingBookmark = bookmark.name
                        } label: {
                            Label("Rename...", systemImage: "pencil")
                        }
                        if !bookmark.isTrackingRemote {
                            Button {
                                onTrack?(bookmark.name)
                            } label: {
                                Label("Track remote", systemImage: "arrow.triangle.pull")
                            }
                        }
                        Divider()
                        Button(role: .destructive) {
                            onDelete(bookmark.name)
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }
                    } label: {
                        HStack {
                            Text(bookmark.name)
                            if bookmark.isTrackingRemote {
                                Image(systemName: "cloud")
                            }
                        }
                    }
                }

                Divider()
            }

            Button {
                onFetch?()
            } label: {
                Label("Fetch All", systemImage: "arrow.down.circle")
            }

            Button {
                onPush?("")
            } label: {
                Label("Push All", systemImage: "arrow.up.circle")
            }

            Divider()

            Button("New Bookmark...") {
                newBookmarkName = ""
                showingCreate = true
            }
        } label: {
            HStack(spacing: 4) {
                Image(systemName: "arrow.triangle.branch")
                    .imageScale(.small)
                if let current = bookmarks.first {
                    Text(current.name)
                        .jayjayFont(12, weight: .medium)
                        .lineLimit(1)
                } else {
                    Text("Bookmarks")
                        .jayjayFont(12, weight: .medium)
                }
            }
        }
        .menuStyle(.borderlessButton)
        .fixedSize()
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

    private func submitCreate() {
        let name = newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        onCreate(name)
        showingCreate = false
    }

    private func submitRename() {
        let newName = renameNewName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !newName.isEmpty, let oldName = renamingBookmark, newName != oldName else { return }
        onRename?(oldName, newName)
        renamingBookmark = nil
    }
}
