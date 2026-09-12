import Foundation
import JayJayCore

struct JjConfigSnapshot: Sendable {
    let path: String
    let sections: [ConfigSection]

    static func load() -> Self {
        let status = checkJjEnvironment()
        guard status.isInstalled, !status.path.isEmpty else {
            return Self(path: "", sections: [])
        }
        let raw = run(status.path, args: ["config", "list"])
        let path = run(status.path, args: ["config", "path", "--user"])
        return Self(path: path, sections: ConfigSection.parse(raw))
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
