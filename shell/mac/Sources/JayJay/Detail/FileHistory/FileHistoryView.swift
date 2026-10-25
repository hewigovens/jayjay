import JayJayCore
import SwiftUI

struct FileHistoryView: View {
    @Environment(\.colorScheme) var colorScheme
    @Environment(\.jayjayTypography) var typography
    let history: [ChangeInfo]
    let path: String
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let onSelectChange: (String) -> Void
    let onDismiss: () -> Void

    @State var comparison = FileHistoryComparison()
    @State private var listWidth: CGFloat = 320

    var body: some View {
        VStack(spacing: 0) {
            comparisonBar
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
            Divider()
            if history.isEmpty {
                ContentUnavailableView(
                    Localized.Repo.noHistory,
                    systemImage: "clock",
                    description: Text(Localized.Repo.noRevisionsModifiedThisFile)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ResizableSplit(
                    width: $listWidth,
                    range: PaneLayout.evologEntryListRange(detailWidth:),
                    onEnded: { _ in },
                    dividerIdentifier: AID.FileHistory.divider,
                    leading: { historyList },
                    trailing: {
                        comparisonDiff
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                )
            }
        }
        .background(
            KeyDownMonitor(onKeyDown: { event in
                guard event.keyCode == KeyCode.escape else { return false }
                if comparison.isPinned {
                    comparison.clear()
                } else {
                    onDismiss()
                }
                return true
            })
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
        .onChange(of: history.first?.commitId.id, initial: true) { _, _ in
            selectNewestEntry()
        }
    }

    private var historyList: some View {
        List(Array(history.enumerated()), id: \.element.commitId.id, selection: selectionBinding) { index, change in
            historyRow(change)
                .accessibilityIdentifier(AID.FileHistory.row(index))
        }
        .listStyle(.plain)
        .contextMenu(forSelectionType: String.self) { ids in
            if let change = entry(for: ids) {
                Button(Localized.Repo.pinAsBase) { pin(change) }
                Button(Localized.Repo.showInGraph) { reveal(change) }
            }
        } primaryAction: { ids in
            if let change = entry(for: ids) {
                reveal(change)
            }
        }
    }

    private func entry(for ids: Set<String>) -> ChangeInfo? {
        guard let id = ids.first else { return nil }
        return history.first { $0.commitId.id == id }
    }

    private var selectionBinding: Binding<String?> {
        Binding(
            get: { comparison.target?.commitId.id },
            set: { commitId in
                guard let commitId,
                      let entry = history.first(where: { $0.commitId.id == commitId })
                else { return }
                comparison.select(endpoint(for: entry))
            }
        )
    }

    private func historyRow(_ change: ChangeInfo) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                Text(change.changeId.highlighted(
                    scheme: colorScheme,
                    font: typography.identifierFont
                ))
                Text(change.author.name)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                Spacer()
                roleBadge(for: change)
                Text(formatTimestamp(change.author.timestampMillis))
                    .jayjayFont(10)
                    .foregroundStyle(.tertiary)
            }
            if !change.description.isEmpty {
                Text(change.description.components(separatedBy: "\n").first ?? "")
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
            } else {
                Text(Localized.Repo.noDescriptionPlaceholder)
                    .jayjayFont(12)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
    }

    private func selectNewestEntry() {
        guard comparison.target == nil, let entry = history.first else { return }
        comparison.select(endpoint(for: entry))
    }

    private func formatTimestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let formatter = RelativeDateTimeFormatter()
        formatter.locale = Localized.locale
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}
