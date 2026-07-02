import SwiftUI

/// Item-based sheet payload so the file list is populated on the sheet's first render, not after the first re-render (issue #102).
struct SplitSheetRequest: Identifiable {
    let id = UUID()
    let paths: [String]
}

/// Split sheet: message field on top so focus never jumps, file list below it capped and scrollable; message/parallel are local state so a cancelled split never leaks into the next one.
struct SplitSheetView: View {
    let paths: [String]
    let onCancel: () -> Void
    let onConfirm: (_ message: String, _ parallel: Bool) -> Void

    @State private var message = ""
    @State private var parallel = false

    private static let maxVisibleFiles = 10

    var body: some View {
        SheetContainer(
            title: "Split \(paths.count) \(paths.count == 1 ? "file" : "files") to new change",
            cancelLabel: "Cancel",
            confirmLabel: "Split",
            confirmDisabled: message.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: onCancel,
            onConfirm: { onConfirm(message, parallel) },
            content: {
                TextField("Description for split change", text: $message)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier(AID.SplitSheet.messageField)
                Toggle("Parallel split", isOn: $parallel)
                    .jayjayFont(12)
                fileList
            }
        )
    }

    @ViewBuilder private var fileList: some View {
        let sorted = paths.sorted()
        if sorted.count > Self.maxVisibleFiles {
            ScrollView {
                fileRows(sorted)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(height: 150)
        } else {
            fileRows(sorted)
        }
    }

    private func fileRows(_ sorted: [String]) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            ForEach(sorted, id: \.self) { path in
                Text(path)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .accessibilityIdentifier(AID.SplitSheet.fileRow(path))
            }
        }
    }
}
