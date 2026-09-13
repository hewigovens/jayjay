import Foundation
import JayJayCore

enum JjConfigSnapshot: Sendable {
    case notInstalled
    case missing
    case found(path: String, sections: [ConfigSection])

    /// `jj config path --user` reports where the file would live even before it exists, and `jj config list` then holds only environment-derived values, so an absent file is an empty state rather than a listing.
    static func load() -> Self {
        let status = checkJjEnvironment()
        guard status.isInstalled, !status.path.isEmpty else {
            return .notInstalled
        }
        let path = run(status.path, args: ["config", "path", "--user"])
        guard FileManager.default.fileExists(atPath: path) else {
            return .missing
        }
        let raw = run(status.path, args: ["config", "list"])
        return .found(path: path, sections: ConfigSection.parse(raw))
    }

    private static func run(_ binary: String, args: [String]) -> String {
        let process = Process()
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        process.executableURL = URL(fileURLWithPath: binary)
        process.arguments = args
        do {
            try process.run()
        } catch {
            return ""
        }
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        return (String(data: data, encoding: .utf8) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
