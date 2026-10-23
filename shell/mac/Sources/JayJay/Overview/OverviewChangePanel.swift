import JayJayCore
import SwiftUI

struct OverviewChangePanel: View {
    let change: OverviewChange
    let files: [FileDiffStats]?
    let onClose: () -> Void
    let onShowInGraph: () -> Void

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography

    private var descriptionBody: String {
        let lines = change.fullDescription.split(separator: "\n", omittingEmptySubsequences: false).dropFirst()
        return lines.joined(separator: "\n").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        OverviewPanel(accessibilityIdentifier: AID.Overview.changePanel) {
            OverviewPanelHeader(onClose: onClose) {
                Text(change.title)
                    .foregroundStyle(change.description.isEmpty ? .tertiary : .primary)
                    .textSelection(.enabled)
            }
            HStack(spacing: 8) {
                Text(change.changeId.highlighted(scheme: colorScheme, font: idFont))
                Text(change.commitId.highlighted(
                    scheme: colorScheme,
                    font: idFont,
                    prefixColor: AppColors.commitIdPrefix(colorScheme)
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
                        OverviewChip(text: "\(name)@", tint: AppColors.workspace(colorScheme))
                    }
                    if change.hasConflict {
                        OverviewChip(text: "conflict", tint: .red)
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
        if let files {
            if files.isEmpty {
                Text(Localized.Overview.noFileChanges)
                    .jayjayFont(11)
                    .foregroundStyle(.tertiary)
            } else {
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
            }
        } else {
            LoadingStatus(label: Localized.Diff.loadingFilesEllipsis)
        }
    }
}
