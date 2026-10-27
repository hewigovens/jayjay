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

    static func refreshLinkIfInstalled() {
        guard let cli = bundledCLIPath,
              let current = try? FileManager.default.destinationOfSymbolicLink(atPath: installPath),
              current != cli
        else { return }
        try? install()
    }

    static func installWithFeedback() {
        let alreadyLinked = isInstalled && (try? FileManager.default.destinationOfSymbolicLink(atPath: installPath)) ==
            bundledCLIPath
        let alert = NSAlert()
        alert.alertStyle = .informational
        if alreadyLinked {
            alert.messageText = Localized.Settings.cliAlreadyInstalled
            alert.informativeText = isInPATH
                ? Localized.Settings.cliAlreadyLinkedAt(path: installPath)
                : Localized.Settings.cliLinkedNotInPath(path: installPath, hint: pathHint)
            alert.addButton(withTitle: Localized.Repo.ok)
            alert.runModal()
            return
        }
        do {
            try install()
            alert.messageText = Localized.Settings.cliInstalled
            alert.informativeText = isInPATH
                ? Localized.Settings.cliLinkedAt(path: installPath)
                : Localized.Settings.cliWasLinkedNotInPath(path: installPath, hint: pathHint)
            alert.addButton(withTitle: Localized.Repo.ok)
            alert.runModal()
        } catch {
            alert.alertStyle = .warning
            alert.messageText = Localized.Settings.cliInstallFailed
            alert.informativeText = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
            alert.addButton(withTitle: Localized.Repo.ok)
            alert.runModal()
        }
    }

    enum CLIError: LocalizedError {
        case notBundled
        case notExecutable(String)

        var errorDescription: String? {
            switch self {
                case .notBundled: Localized.Settings.cliBinaryNotFoundInApp
                case let .notExecutable(path): Localized.Settings.cliNotExecutable(path: path)
            }
        }
    }

    static var pathHint: String {
        let shell = loginShell
        let rcFile: String
        if shell.hasSuffix("fish") {
            rcFile = "~/.config/fish/config.fish"
            return Localized.Settings.cliAddToRcFish(file: rcFile, dir: installDir)
        } else if shell.hasSuffix("zsh") {
            rcFile = "~/.zshrc"
        } else {
            rcFile = "~/.bashrc"
        }
        return Localized.Settings.cliAddToRc(file: rcFile, dir: installDir)
    }
}
