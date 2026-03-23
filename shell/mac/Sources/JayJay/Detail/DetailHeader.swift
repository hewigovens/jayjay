import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var headerSection: some View {
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
                    Text("Bookmarks").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        Text(name).jayjayFont(11, design: .monospaced)
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(.tint.opacity(0.15), in: .capsule)
                    }
                }
            }
        }
    }

    var descriptionSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Description").jayjayFont(17, weight: .semibold)
                Spacer()
                if editingDescription {
                    Button("Save") { onDescribe(detail.info.changeId, descriptionText)
                        editingDescription = false
                    }
                    .keyboardShortcut("s")
                    Button("Cancel") { descriptionText = detail.info.description
                        editingDescription = false
                    }
                } else {
                    Button("Edit") { editingDescription = true }
                }
            }
            if editingDescription {
                TextEditor(text: $descriptionText)
                    .jayjayFont(13, design: .monospaced)
                    .frame(minHeight: 60, maxHeight: 120)
                    .scrollContentBackground(.hidden)
                    .padding(6)
                    .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.primary.opacity(0.1)))
            } else {
                Text(detail.info.description).jayjayFont(13, design: .monospaced).textSelection(.enabled)
            }
        }
    }

    var compareBanner: some View {
        HStack(spacing: 8) {
            Image(systemName: "arrow.left.arrow.right")
                .foregroundStyle(.orange)
            Text("Comparing")
                .jayjayFont(12, weight: .medium)
            Text(String(compareFromId?.prefix(8) ?? ""))
                .jayjayFont(12, weight: .semibold, design: .monospaced)
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            Text(String(detail.info.changeId.prefix(8)))
                .jayjayFont(12, weight: .semibold, design: .monospaced)
            Spacer()
            Text("\(detail.diff.count) files changed")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                onClearCompare?()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .help("Exit compare mode")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.08))
    }

    func formatTimestamp(_ millis: Int64) -> String {
        Date(timeIntervalSince1970: Double(millis) / 1000.0).formatted(.dateTime.year().month().day().hour().minute())
    }

    func resetState() {
        descriptionText = detail.info.description
        editingDescription = false
        selectedPath = detail.diff.first?.path
        fileFilter = ""
        annotateLines = nil
        annotatePath = nil
        fileHistory = nil
        fileHistoryPath = nil
        DiffSection.clearCache()
    }
}
