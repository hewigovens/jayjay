import JayJayCore
import SwiftUI

struct SubmoduleAttentionSheet: View {
    let repoPath: String
    let submoduleStatuses: [GitSubmoduleStatus]
    let onClose: () -> Void
    let onAutoCommit: () async -> Bool

    @Environment(AppSettings.self) private var settings
    @State private var autoCommitSafeSubmodules = false
    @State private var isProcessing = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Submodule changes need attention")
                .jayjayFont(14, weight: .semibold)
            Text(
                "JayJay found Git submodule changes. Review the items below, then optionally let JayJay handle the safe ones."
            )
            .jayjayFont(12)
            .foregroundStyle(.secondary)
            Text("To disable this alert, turn off Enable Git submodule support in Settings > Diff > Git.")
                .jayjayFont(11)
                .foregroundStyle(.secondary)

            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(submoduleStatuses.sorted(by: { $0.path < $1.path }), id: \.path) { status in
                        row(for: status)
                    }
                }
            }
            .frame(minHeight: 120, maxHeight: 220)

            if hasSafeAutoCommitStatuses {
                Toggle(isOn: $autoCommitSafeSubmodules) {
                    Text("Auto-commit safe submodule updates")
                        .jayjayFont(12)
                }
            }

            HStack {
                Spacer()
                Button("Close", action: onClose)
                    .keyboardShortcut(autoCommitSafeSubmodules ? .cancelAction : .defaultAction)
                if autoCommitSafeSubmodules {
                    Button {
                        isProcessing = true
                        Task {
                            let success = await onAutoCommit()
                            isProcessing = false
                            if success {
                                onClose()
                            }
                        }
                    } label: {
                        if isProcessing {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Text("Commit Safe Updates")
                        }
                    }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(isProcessing)
                }
            }
        }
        .padding(20)
        .frame(width: 520)
    }

    @ViewBuilder
    private func row(for status: GitSubmoduleStatus) -> some View {
        let absolutePath = URL(fileURLWithPath: repoPath).appendingPathComponent(status.path).path

        HStack(spacing: 10) {
            Image(systemName: "square.stack.3d.up")
                .foregroundStyle(.secondary)

            VStack(alignment: .leading, spacing: 2) {
                Text(status.path)
                    .jayjayFont(12, weight: .medium, design: .monospaced)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(statusLabel(for: status))
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 8)

            actionButton(systemImage: "curlybraces") {
                settings.openInEditor(filePath: ".", repoPath: absolutePath)
            }
            .help("Open submodule in editor")

            actionButton(systemImage: "terminal") {
                settings.openInTerminal(at: absolutePath)
            }
            .help("Open submodule in terminal")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func actionButton(systemImage: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: systemImage)
                .frame(width: 24, height: 24)
        }
        .buttonStyle(.plain)
        .foregroundStyle(.secondary)
    }

    private var hasSafeAutoCommitStatuses: Bool {
        submoduleStatuses.contains { status in
            status.hasNewCommits && !status.hasModifiedContent && !status.hasUntrackedContent
        }
    }

    private func statusLabel(for status: GitSubmoduleStatus) -> String {
        var parts = [String]()
        if status.hasNewCommits {
            parts.append("new commits")
        }
        if status.hasModifiedContent {
            parts.append("modified content")
        }
        if status.hasUntrackedContent {
            parts.append("untracked content")
        }
        if parts.isEmpty {
            return "submodule change"
        }
        if status.hasNewCommits, !status.hasModifiedContent, !status.hasUntrackedContent {
            return "new commits"
        }
        return parts.joined(separator: ", ")
    }
}
