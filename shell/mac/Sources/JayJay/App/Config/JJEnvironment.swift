import Foundation

/// Checks for jj installation and provides environment info.
enum JJEnvironment {
    struct Status {
        let isInstalled: Bool
        let version: String?
        let path: String?
    }

    static func check() -> Status {
        let searchPaths = [
            "/opt/homebrew/bin/jj",
            "/usr/local/bin/jj",
            "/usr/bin/jj",
            "\(NSHomeDirectory())/.cargo/bin/jj",
        ]

        for path in searchPaths {
            if FileManager.default.isExecutableFile(atPath: path) {
                let version = shellOutput("\(path) version")
                return Status(isInstalled: true, version: version, path: path)
            }
        }

        // Try PATH
        let whichResult = shellOutput("/usr/bin/which jj")
        if let whichResult, !whichResult.isEmpty {
            let version = shellOutput("jj version")
            return Status(isInstalled: true, version: version, path: whichResult)
        }

        return Status(isInstalled: false, version: nil, path: nil)
    }

    private static func shellOutput(_ command: String) -> String? {
        let proc = Process()
        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = FileHandle.nullDevice
        proc.executableURL = URL(fileURLWithPath: "/bin/bash")
        proc.arguments = ["-c", command]
        try? proc.run()
        proc.waitUntilExit()
        guard proc.terminationStatus == 0 else { return nil }
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let str = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
        return str?.isEmpty == true ? nil : str
    }
}
