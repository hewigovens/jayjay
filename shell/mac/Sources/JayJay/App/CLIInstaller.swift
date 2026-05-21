import AppKit
import Foundation
import JayJayCore

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

    static let loginShell: String = JayJayCore.loginShell()

    static let loginShellPATH: String? = JayJayCore.loginShellPath()

    static var isInPATH: Bool {
        let path = loginShellPATH ?? ProcessInfo.processInfo.environment["PATH"] ?? ""
        return path.split(separator: ":").contains { $0 == installDir }
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

    static func installWithFeedback() {
        let alreadyLinked = isInstalled && (try? FileManager.default.destinationOfSymbolicLink(atPath: installPath)) ==
            bundledCLIPath
        let alert = NSAlert()
        alert.alertStyle = .informational
        if alreadyLinked {
            alert.messageText = "CLI Already Installed"
            alert.informativeText = isInPATH ? "jayjay is already linked at \(installPath)." : "jayjay is linked at \(installPath), but the directory is not in your PATH.\n\n\(pathHint)"
            alert.addButton(withTitle: "OK")
            alert.runModal()
            return
        }
        do {
            try install()
            alert.messageText = "CLI Installed"
            alert.informativeText = isInPATH ? "jayjay is linked at \(installPath)." : "jayjay was linked at \(installPath), but the directory is not in your PATH.\n\n\(pathHint)"
            alert.addButton(withTitle: "OK")
            alert.runModal()
        } catch {
            alert.alertStyle = .warning
            alert.messageText = "CLI Install Failed"
            alert.informativeText = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
            alert.addButton(withTitle: "OK")
            alert.runModal()
        }
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
        let shell = loginShell
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
