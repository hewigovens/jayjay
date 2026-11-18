import JayJayCore
import SwiftUI

struct UndoView: View {
    let operations: RecentOperations?
    let undoing: String?
    @Binding var selectedId: String?
    let isShown: Bool
    let onRestore: (String) -> Void

    @State private var contextMenuRequest: SelectionContextMenuRequest?
    @FocusState private var isListFocused: Bool
    @Environment(KeyboardFocus.self) private var focus: KeyboardFocus?
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography
    @Environment(\.appThemes) private var appThemes

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            switch operations {
                case let .entries(entries) where entries.isEmpty:
                    statusMessage(Text(Localized.Repo.noOperationsFound))
                case let .entries(entries):
                    operationList(entries)
                case let .failed(message):
                    statusMessage(Text(message).textSelection(.enabled))
                case nil:
                    LoadingStatus(label: Localized.Oplog.loadingOperationsEllipsis)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Panel.operationLog)
        .onChange(of: operations?.entries.map(\.id.id), initial: true) { _, ids in
            // A refresh must not retarget a pending Restore; a hidden pane keeps following the default.
            if isShown, let selectedId, ids?.contains(selectedId) == true {
                return
            }
            selectedId = operations?.entries.restoreTarget(undoing: undoing)
        }
        .onChange(of: isShown) { _, shown in
            isListFocused = shown
            if !shown {
                selectedId = operations?.entries.restoreTarget(undoing: nil)
            }
        }
    }

    private func statusMessage(_ text: some View) -> some View {
        text
            .jayjayFont(13)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
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
            .selectionContextMenu(for: entry.id.id, request: $contextMenuRequest) {
                Button(Localized.Repo.restore) { onRestore(entry.id.id) }
                    .disabled(entry.isCurrent)
            }
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
        .background(KeyboardOwner())
        .focused($isListFocused)
        .keyboardFocusStop(.operationLogList, isAvailable: isShown) { isListFocused = true }
        .onChange(of: isListFocused) { _, focused in
            focus?.updateInputFocus(.operationLogList, isFocused: focused)
        }
        .onKeyPress(.return, phases: .down) { press in
            guard press.modifiers == .control, let selectedId else { return .ignored }
            contextMenuRequest = SelectionContextMenuRequest(target: selectedId)
            return .handled
        }
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

extension RecentOperations {
    var entries: [OpLogEntry] {
        switch self {
            case let .entries(entries): entries
            case .failed: []
        }
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
