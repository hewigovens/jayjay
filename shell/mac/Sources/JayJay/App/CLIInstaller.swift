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

    static func uninstall() {
        try? FileManager.default.removeItem(atPath: installPath)
    }

    static func install() {
        guard let cli = bundledCLIPath, FileManager.default.isExecutableFile(atPath: cli) else { return }
        try? FileManager.default.createDirectory(atPath: installDir, withIntermediateDirectories: true)
        try? FileManager.default.removeItem(atPath: installPath)
        try? FileManager.default.createSymbolicLink(atPath: installPath, withDestinationPath: cli)
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
