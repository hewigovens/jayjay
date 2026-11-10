import JayJayCore
import SwiftUI

struct UndoView: View {
    let entries: [OpLogEntry]?
    let undoing: String?
    let onRestore: (String) -> Void
    let onDismiss: () -> Void

    @State private var selectedId: String?
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography
    @Environment(\.appThemes) private var appThemes

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header
            Divider()
            if let entries {
                if entries.isEmpty {
                    emptyState
                } else {
                    operationList(entries)
                }
            } else {
                LoadingStatus(label: Localized.Oplog.loadingOperationsEllipsis)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            Divider()
            footer
        }
        .frame(width: 600, height: 480)
        .onChange(of: entries?.map(\.id.id), initial: true) {
            selectedId = entries?.restoreTarget(undoing: undoing)
        }
    }

    private var header: some View {
        HStack {
            Image(systemName: "clock.arrow.circlepath")
                .font(.system(size: 16))
                .foregroundStyle(.secondary)
            Text(Localized.Repo.operationLog)
                .jayjayFont(15, weight: .semibold)
                .accessibilityIdentifier(AID.Panel.operationLog)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Text(Localized.Repo.noOperationsFound)
                .jayjayFont(13)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func operationList(_ entries: [OpLogEntry]) -> some View {
        List(entries, id: \.id.id, selection: $selectedId) { entry in
            HStack(spacing: 10) {
                operationIcon(for: entry.description)
                VStack(alignment: .leading, spacing: 3) {
                    HStack(spacing: 6) {
                        Text(descriptionLabel(entry.description))
                            .jayjayFont(13, weight: entry.isCurrent ? .semibold : .regular)
                            .lineLimit(1)
                        if entry.isCurrent {
                            Text(Localized.Repo.currentBadge)
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.white)
                                .padding(.horizontal, 5)
                                .padding(.vertical, 1)
                                .background(Capsule().fill(Color.accentColor))
                        }
                    }
                    Text(Date.abbreviatedRelativeLabel(millis: entry.timestampMillis))
                        .jayjayFont(11)
                        .foregroundStyle(.tertiary)
                }
                Spacer()
                Text(entry.id.highlighted(
                    font: typography.identifierFont,
                    prefixColor: appThemes.colors(for: colorScheme).changeIdPrefix
                ))
            }
            .padding(.vertical, 2)
            .contentShape(Rectangle())
        }
        .listStyle(.plain)
    }

    private var footer: some View {
        HStack {
            Spacer()
            Button(Localized.Common.cancel) { onDismiss() }
                .keyboardShortcut(.cancelAction)
            Button(Localized.Repo.restore) {
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
        return entries?.first(where: { $0.id.id == id })?.isCurrent == true
    }

    private func descriptionLabel(_ description: String) -> String {
        let trimmed = description.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? Localized.Repo.noDescriptionPlaceholder : trimmed
    }

    @ViewBuilder
    private func operationIcon(for description: String) -> some View {
        let style = OperationIcon.style(for: description)
        Image(systemName: style.symbol)
            .jayjayFont(13)
            .foregroundStyle(style.color)
            .frame(width: 18)
    }
}

extension [OpLogEntry] {
    func restoreTarget(undoing operationId: String?) -> String? {
        guard let entry = first(where: { entry in operationId.map { entry.id.id == $0 } ?? entry.isCurrent }),
              entry.parentIds.count == 1,
              let parent = entry.parentIds.first,
              contains(where: { $0.id.id == parent })
        else { return nil }
        return parent
    }
}
