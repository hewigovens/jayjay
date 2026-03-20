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

    @State private var showingCreate = false
    @State private var newBookmarkName = ""

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
    }

    private func submitCreate() {
        let name = newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        onCreate(name)
        showingCreate = false
    }
}
