import JayJayCore
import SwiftUI

struct OverviewChangePanel: View {
    let change: OverviewChange
    let files: [FileDiffStats]?
    let onClose: () -> Void
    let onShowInGraph: () -> Void

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

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
            Button("Show in Graph", action: onShowInGraph)
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
        fontFamily.scaledFont(11, baseSize: baseFontSize, design: .monospaced)
    }

    @ViewBuilder
    private var fileList: some View {
        if let files {
            if files.isEmpty {
                Text("No file changes")
                    .jayjayFont(11)
                    .foregroundStyle(.tertiary)
            } else {
                OverviewPanelSection(title: "\(files.count) file\(files.count == 1 ? "" : "s")") {
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
            Text("Loading files…")
                .jayjayFont(11)
                .foregroundStyle(.tertiary)
        }
    }
}
