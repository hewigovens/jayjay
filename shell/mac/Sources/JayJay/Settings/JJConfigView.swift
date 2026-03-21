import AppKit
import SwiftUI
import JayJayBindings

struct JJConfigView: View {
    @State private var configText: String?
    @State private var configPath = ""

    private static var cachedConfig: String?
    private static var cachedPath: String?

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
        if let cached = Self.cachedConfig {
            configText = cached
            configPath = Self.cachedPath ?? ""
            return
        }
        let status = checkJjEnvironment()
        guard status.isInstalled, !status.path.isEmpty else {
            configText = "jj not found"
            return
        }
        let jj = status.path
        let config = Self.run(jj, args: ["config", "list"])
        let path = Self.run(jj, args: ["config", "path", "--user"])
        Self.cachedConfig = config
        Self.cachedPath = path
        configText = config
        configPath = path
    }

    private static func run(_ binary: String, args: [String]) -> String {
        let proc = Process()
        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = FileHandle.nullDevice
        proc.executableURL = URL(fileURLWithPath: binary)
        proc.arguments = args
        try? proc.run()
        proc.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        return (String(data: data, encoding: .utf8) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
