import JayJayCore
import SwiftUI

/// The selected change: description and changed files with line counts. The diff itself opens in the repository window.
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
        ScrollView {
            panel
        }
        .frame(width: 400)
        .accessibilityIdentifier(AID.Overview.changePanel)
    }

    private var panel: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top, spacing: 8) {
                Text(change.title)
                    .jayjayFont(13, weight: .semibold)
                    .foregroundStyle(change.description.isEmpty ? .tertiary : .primary)
                    .lineLimit(4)
                Spacer(minLength: 0)
                Button(action: onClose) {
                    Image(systemName: "xmark")
                        .jayjayFont(10, weight: .semibold)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Close")
            }
            HStack(spacing: 8) {
                Text(change.changeId.highlighted(scheme: colorScheme, font: fontFamily.identifierFont(baseSize: baseFontSize)))
                Text(change.commitId.highlighted(
                    scheme: colorScheme,
                    font: fontFamily.identifierFont(baseSize: baseFontSize),
                    prefixColor: AppColors.commitIdPrefix(colorScheme)
                ))
                Text(Date.relativeLabel(millis: change.timestampMillis))
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            if !change.workspaces.isEmpty || change.hasConflict {
                HStack(spacing: 4) {
                    ForEach(change.workspaces, id: \.self) { name in
                        OverviewChip(text: "\(name)@", tint: AppColors.workspace(colorScheme))
                    }
                    if change.hasConflict {
                        OverviewChip(text: "conflict", tint: .red)
                    }
                }
            }
            if !descriptionBody.isEmpty {
                Text(descriptionBody)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
            }
            fileList
            Button("Show in Graph", action: onShowInGraph)
                .controlSize(.small)
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    @ViewBuilder
    private var fileList: some View {
        if let files {
            if files.isEmpty {
                Text("No file changes")
                    .jayjayFont(11)
                    .foregroundStyle(.tertiary)
            } else {
                VStack(alignment: .leading, spacing: 3) {
                    Text("\(files.count) file\(files.count == 1 ? "" : "s")")
                        .jayjayFont(10, weight: .semibold)
                        .foregroundStyle(.secondary)
                        .textCase(.uppercase)
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
