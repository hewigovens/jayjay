import SwiftUI
import JayJayBindings

struct DetailView: View {
    let detail: ChangeDetail?
    let onDescribe: (String, String) -> Void

    var body: some View {
        if let detail = detail {
            ChangeDetailView(detail: detail, onDescribe: onDescribe)
        } else {
            ContentUnavailableView(
                "Select a Change",
                systemImage: "doc.text",
                description: Text("Choose a change from the list to see its details.")
            )
        }
    }
}

struct ChangeDetailView: View {
    let detail: ChangeDetail
    let onDescribe: (String, String) -> Void

    @State private var editingDescription = false
    @State private var descriptionText = ""

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                headerSection
                descriptionSection
                diffSection
            }
            .padding()
        }
        .onAppear {
            resetEditorState()
        }
        .onChange(of: detail.info.commitId) {
            resetEditorState()
        }
    }

    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            LabeledRow("Change", value: detail.info.changeId)
            LabeledRow("Commit", value: String(detail.info.commitId.prefix(12)))
            LabeledRow("Author", value: "\(detail.info.author) <\(detail.info.email)>")
            LabeledRow("Date", value: formatTimestamp(detail.info.timestampMillis))

            if !detail.info.parents.isEmpty {
                LabeledRow("Parents", value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
            }

            if !detail.info.bookmarks.isEmpty {
                HStack(spacing: 4) {
                    Text("Bookmarks")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        Text(name)
                            .font(.caption.monospaced())
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(.tint.opacity(0.15), in: .capsule)
                    }
                }
            }
        }
    }

    private var descriptionSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Description")
                    .font(.headline)
                Spacer()
                if editingDescription {
                    Button("Save") {
                        onDescribe(detail.info.changeId, descriptionText)
                        editingDescription = false
                    }
                    .keyboardShortcut("s")
                    Button("Cancel") {
                        descriptionText = detail.info.description
                        editingDescription = false
                    }
                    .keyboardShortcut(.cancelAction)
                } else {
                    Button("Edit") {
                        editingDescription = true
                    }
                }
            }

            if editingDescription {
                TextEditor(text: $descriptionText)
                    .font(.body.monospaced())
                    .frame(minHeight: 80)
                    .border(.separator)
            } else if detail.info.description.isEmpty {
                Text("(no description)")
                    .foregroundStyle(.tertiary)
                    .italic()
            } else {
                Text(detail.info.description)
                    .font(.body.monospaced())
                    .textSelection(.enabled)
            }
        }
    }

    @ViewBuilder
    private var diffSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Changed Files")
                .font(.headline)
            if detail.diff.isEmpty {
                Text("No file changes in this revision.")
                    .foregroundStyle(.secondary)
            } else {
                ForEach(detail.diff, id: \.path) { hunk in
                    DiffHunkView(hunk: hunk)
                }
            }
        }
    }

    private func formatTimestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000.0)
        return date.formatted(.dateTime.year().month().day().hour().minute())
    }

    private func resetEditorState() {
        descriptionText = detail.info.description
        editingDescription = false
    }
}

struct LabeledRow: View {
    let label: String
    let value: String

    init(_ label: String, value: String) {
        self.label = label
        self.value = value
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
                .frame(width: 70, alignment: .trailing)
            Text(value)
                .font(.caption.monospaced())
                .textSelection(.enabled)
        }
    }
}
