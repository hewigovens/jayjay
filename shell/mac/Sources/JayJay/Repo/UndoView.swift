import JayJayBindings
import SwiftUI

struct UndoView: View {
    let entries: [OpLogEntry]
    let onRestore: (String) -> Void
    let onDismiss: () -> Void

    @State private var selectedId: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header
            Divider()
            if entries.isEmpty {
                emptyState
            } else {
                operationList
            }
            Divider()
            footer
        }
        .frame(width: 480, height: 380)
    }

    private var header: some View {
        HStack {
            Image(systemName: "clock.arrow.circlepath")
                .font(.system(size: 16))
                .foregroundStyle(.secondary)
            Text("Operation Log")
                .jayjayFont(14, weight: .semibold)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Text("No operations found")
                .jayjayFont(13)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var operationList: some View {
        List(entries, id: \.id, selection: $selectedId) { entry in
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 3) {
                    HStack(spacing: 6) {
                        Text(descriptionLabel(entry.description))
                            .jayjayFont(12, weight: entry.isCurrent ? .semibold : .regular)
                            .lineLimit(1)
                        if entry.isCurrent {
                            Text("current")
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.white)
                                .padding(.horizontal, 5)
                                .padding(.vertical, 1)
                                .background(Capsule().fill(Color.accentColor))
                        }
                    }
                    Text(entry.timestamp)
                        .jayjayFont(10, design: .monospaced)
                        .foregroundStyle(.tertiary)
                }
                Spacer()
                Text(String(entry.id.prefix(12)))
                    .jayjayFont(10, design: .monospaced)
                    .foregroundStyle(.tertiary)
            }
            .padding(.vertical, 2)
            .contentShape(Rectangle())
        }
        .listStyle(.plain)
    }

    private var footer: some View {
        HStack {
            Spacer()
            Button("Cancel") { onDismiss() }
                .keyboardShortcut(.cancelAction)
            Button("Restore") {
                guard let id = selectedId else { return }
                onRestore(id)
                onDismiss()
            }
            .keyboardShortcut(.defaultAction)
            .disabled(selectedId == nil || isCurrentSelected)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private var isCurrentSelected: Bool {
        guard let id = selectedId else { return false }
        return entries.first(where: { $0.id == id })?.isCurrent == true
    }

    private func descriptionLabel(_ description: String) -> String {
        let trimmed = description.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? "(no description)" : trimmed
    }
}
