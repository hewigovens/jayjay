import AppKit
import JayJayCore
import SwiftUI

struct JJConfigView: View {
    @State private var sections: [ConfigSection] = []
    @State private var configPath = ""
    @State private var isLoading = true

    private static var cachedConfig: String?
    private static var cachedPath: String?

    var body: some View {
        if isLoading {
            ProgressView()
                .controlSize(.small)
                .frame(maxWidth: .infinity, minHeight: 80)
                .task { await loadConfig() }
        } else {
            configPathRow
            ForEach(sections) { section in
                Section(section.name) {
                    ForEach(section.entries) { entry in
                        configRow(key: entry.key, value: entry.value, icon: entry.icon)
                    }
                }
            }
        }
    }

    private var configPathRow: some View {
        HStack {
            Text(configPath)
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
            Spacer()
            Button("Open") {
                NSWorkspace.shared.open(URL(fileURLWithPath: configPath))
            }
            .controlSize(.small)
        }
    }

    private func configRow(key: String, value: String, icon: String) -> some View {
        HStack {
            HStack(spacing: 6) {
                Image(systemName: icon)
                    .frame(width: 16, alignment: .center)
                    .foregroundStyle(.secondary)
                Text(key)
            }
            Spacer()
            Text(value)
                .font(.system(size: 12, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }

    private func loadConfig() async {
        let raw: String
        if let cached = Self.cachedConfig {
            raw = cached
            configPath = Self.cachedPath ?? ""
        } else {
            let status = checkJjEnvironment()
            guard status.isInstalled, !status.path.isEmpty else {
                isLoading = false
                return
            }
            let jj = status.path
            raw = Self.run(jj, args: ["config", "list"])
            let path = Self.run(jj, args: ["config", "path", "--user"])
            Self.cachedConfig = raw
            Self.cachedPath = path
            configPath = path
        }
        sections = ConfigSection.parse(raw)
        isLoading = false
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
