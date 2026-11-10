import JayJayCore
import SwiftUI

struct OverviewChangePanel: View {
    let change: OverviewChange
    let files: Result<[FileDiffStats], any Error>?
    let onClose: () -> Void
    let onShowInGraph: () -> Void

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography
    @Environment(\.appThemes) private var appThemes

    private var descriptionBody: String {
        let lines = change.fullDescription.split(separator: "\n", omittingEmptySubsequences: false).dropFirst()
        return lines.joined(separator: "\n").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        let colors = appThemes.colors(for: colorScheme)
        OverviewPanel(accessibilityIdentifier: AID.Overview.changePanel) {
            OverviewPanelHeader(onClose: onClose) {
                Text(change.title)
                    .foregroundStyle(change.description.isEmpty ? .tertiary : .primary)
                    .textSelection(.enabled)
            }
            HStack(spacing: 8) {
                Text(change.changeId.highlighted(font: idFont, prefixColor: colors.changeIdPrefix))
                Text(change.commitId.highlighted(
                    font: idFont,
                    prefixColor: colors.commitIdPrefix
                ))
                Text("·").foregroundStyle(.tertiary)
                Text(Date.relativeLabel(millis: change.timestampMillis))
                    .foregroundStyle(.secondary)
            }
            .jayjayFont(11)
            .textSelection(.enabled)
            if !change.workspaces.isEmpty || change.hasConflict {
                FlowLayout(spacing: 4) {
                    ForEach(change.workspaces, id: \.self) { name in
                        OverviewChip(text: "\(name)@", tint: colors.workspace)
                    }
                    if change.hasConflict {
                        OverviewChip(text: Localized.Repo.conflictBadge, tint: .red)
                    }
                }
            }
            Button(Localized.Repo.showInGraph, action: onShowInGraph)
                .controlSize(.small)
            if !descriptionBody.isEmpty {
                Text(descriptionBody)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
            }
            fileList
        }
    }

    private var idFont: Font {
        typography.font(11, design: .monospaced)
    }

    @ViewBuilder
    private var fileList: some View {
        switch files {
            case let .failure(error):
                Text(error.friendlyDescription)
                    .jayjayFont(11)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            case let .success(files) where files.isEmpty:
                Text(Localized.Overview.noFileChanges)
                    .jayjayFont(11)
                    .foregroundStyle(.tertiary)
            case let .success(files):
                OverviewPanelSection(title: Localized.Repo.fileCount(count: files.count)) {
                    ForEach(files, id: \.path) { file in
                        HStack(spacing: 6) {
                            Text(file.path)
                                .jayjayFont(11)
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .help(file.path)
                            Spacer(minLength: 4)
                            if file.insertions > 0 {
                                Text("+\(file.insertions)")
                                    .jayjayFont(10, design: .monospaced)
                                    .foregroundStyle(.green)
                            }
                            if file.deletions > 0 {
                                Text("−\(file.deletions)")
                                    .jayjayFont(10, design: .monospaced)
                                    .foregroundStyle(.red)
                            }
                        }
                    }
                }
            case nil:
                LoadingStatus(label: Localized.Diff.loadingFilesEllipsis)
        }
    }
}
