import Foundation

enum CLIInstaller {
    static var installDir: String {
        "\(NSHomeDirectory())/.local/bin"
    }

    static var installPath: String {
        "\(installDir)/jayjay"
    }

    static var isInstalled: Bool {
        FileManager.default.isExecutableFile(atPath: installPath)
    }

    static var isInPATH: Bool {
        let path = ProcessInfo.processInfo.environment["PATH"] ?? ""
        return path.components(separatedBy: ":").contains(installDir)
    }

    static var bundledCLIPath: String? {
        Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("jayjay-cli").path
    }

    static func uninstall() throws {
        try FileManager.default.removeItem(atPath: installPath)
    }

    static func install() throws {
        guard let cli = bundledCLIPath else {
            throw CLIError.notBundled
        }
        guard FileManager.default.isExecutableFile(atPath: cli) else {
            throw CLIError.notExecutable(cli)
        }
        try FileManager.default.createDirectory(atPath: installDir, withIntermediateDirectories: true)
        try? FileManager.default.removeItem(atPath: installPath)
        try FileManager.default.createSymbolicLink(atPath: installPath, withDestinationPath: cli)
    }

    enum CLIError: LocalizedError {
        case notBundled
        case notExecutable(String)

        var errorDescription: String? {
            switch self {
                case .notBundled: "CLI binary not found in app bundle"
                case let .notExecutable(path): "CLI binary not executable: \(path)"
            }
        }
    }

    static var pathHint: String {
        let shell = ProcessInfo.processInfo.environment["SHELL"] ?? "/bin/zsh"
        let rcFile: String
        if shell.hasSuffix("fish") {
            rcFile = "~/.config/fish/config.fish"
            return "Add to \(rcFile):\nfish_add_path \(installDir)"
        } else if shell.hasSuffix("zsh") {
            rcFile = "~/.zshrc"
        } else {
            rcFile = "~/.bashrc"
        }
        return "Add to \(rcFile):\nexport PATH=\"\(installDir):$PATH\""
    }
}
