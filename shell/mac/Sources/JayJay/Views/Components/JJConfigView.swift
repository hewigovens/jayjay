import AppKit
import SwiftUI

struct JJConfigView: View {
    @State private var configText: String?
    @State private var configPath = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if !configPath.isEmpty {
                HStack {
                    Text(configPath)
                        .jayjayFont(11, design: .monospaced)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    Spacer()
                    Button("Open") {
                        NSWorkspace.shared.open(URL(fileURLWithPath: configPath))
                    }
                    .controlSize(.small)
                }
            }

            Group {
                if let config = configText {
                    Text(config)
                        .jayjayFont(12, design: .monospaced)
                        .textSelection(.enabled)
                } else {
                    ProgressView()
                        .controlSize(.small)
                }
            }
            .padding(10)
            .frame(maxWidth: .infinity, minHeight: 80, alignment: .topLeading)
            .background(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(Color.primary.opacity(0.04))
            )
        }
        .task { await loadConfig() }
    }

    private func loadConfig() async {
        let config = Self.runShell("jj config list")
        let path = Self.runShell("jj config path --user")
        configText = config
        configPath = path
    }

    private static func runShell(_ command: String) -> String {
        let proc = Process()
        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = pipe
        proc.executableURL = URL(fileURLWithPath: "/bin/bash")
        proc.arguments = ["-c", command]
        try? proc.run()
        proc.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        return (String(data: data, encoding: .utf8) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
