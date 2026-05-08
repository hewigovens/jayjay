import AppKit
import JayJayCore
import SwiftUI

extension PaletteRoot {
    var isJJ: Bool {
        jjCommandBody(query: query) != nil
    }

    var jjCmd: String {
        jjCommandBody(query: query) ?? ""
    }

    @ViewBuilder
    var jjSection: some View {
        if isRunning {
            ProgressView().controlSize(.small).frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let result = jjResult {
            jjResultView(result)
        } else if let jjError {
            VStack(alignment: .leading, spacing: 10) {
                Label(jjError, systemImage: "exclamationmark.triangle")
                    .font(.system(size: 12))
                    .foregroundStyle(.red)
                jjDiscovery
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(12)
        } else {
            jjPreview
        }
    }

    var jjDiscovery: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Raw jj commands run in this repository. Use ↑/↓ for history.")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
            HStack(spacing: 8) {
                ForEach(["status", "log -r @", "diff --stat", "op log"], id: \.self) { suggestion in
                    Button("jj \(suggestion)") { query = "jj \(suggestion)" }
                        .controlSize(.small)
                }
            }
            if !history.isEmpty {
                Text("Recent")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(.secondary)
                ForEach(history.prefix(5), id: \.self) { command in
                    Button { query = "jj \(command)" } label: {
                        HStack {
                            Image(systemName: "clock.arrow.circlepath")
                                .foregroundStyle(.secondary)
                            Text("jj \(command)")
                                .font(.system(size: 11, design: .monospaced))
                                .lineLimit(1)
                            Spacer()
                        }
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }

    func execute() {
        if isJJ {
            executeJJ()
        } else if selectedIndex < filtered.count {
            filtered[selectedIndex].action()
            onDismiss()
        }
    }

    func recallHistory(older: Bool) {
        let next = CommandPaletteHistory.recall(
            history: history,
            historyIndex: historyIndex,
            older: older
        )
        guard let next else { return }
        historyIndex = next.historyIndex
        isRecallingHistory = true
        query = next.query
    }

    private func jjResultView(_ result: JjCommandRun) -> some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: result.success ? "checkmark.circle.fill" : "xmark.circle.fill")
                    .foregroundStyle(result.success ? .green : .red)
                Text(result.display)
                    .font(.system(size: 12, design: .monospaced))
                    .lineLimit(1)
                Spacer()
                Text(result.success ? "exit 0" : "exit \(result.exitCode)")
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(.secondary)
                Button("Copy Output") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(result.output, forType: .string)
                }
                .controlSize(.small)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            Divider()
            ScrollView {
                Text(result.output)
                    .font(.system(size: 11, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(12)
            }
        }
    }

    private var jjPreview: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("jj \(jjCmd)")
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundStyle(.secondary)
                Spacer()
                if !jjCmd.isEmpty {
                    Text("Enter ↵")
                        .font(.system(size: 10))
                        .foregroundStyle(.tertiary)
                }
            }
            jjDiscovery
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(12)
    }

    private func executeJJ() {
        guard !jjCmd.isEmpty else { return }
        guard parseJjCommandArgs(command: jjCmd) != nil else {
            jjError = "Unclosed quote in jj command."
            return
        }
        isRunning = true
        jjResult = nil
        jjError = nil
        let path = repoPath
        let command = jjCmd
        Task.detached {
            let result = Result { try runJjCommandInRepoPath(repoPath: path, command: command) }
            await MainActor.run {
                switch result {
                    case let .success(commandResult):
                        jjResult = commandResult
                        history = CommandPaletteHistory.record(command)
                    case let .failure(error):
                        jjError = error.localizedDescription
                }
                isRunning = false
            }
        }
    }
}
