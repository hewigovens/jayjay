import JayJayCore
import SwiftUI

extension SettingsView {
    var cliTab: some View {
        Form {
            Section("Version control") {
                HStack {
                    settingsLabel("jayjay", icon: "bird")
                    Spacer()
                    Text(CLIInstaller.installPath)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    if cliInstalled {
                        Button("Uninstall") {
                            try? CLIInstaller.uninstall()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Button("Install") {
                            do {
                                try CLIInstaller.install()
                                cliError = nil
                            } catch {
                                cliError = error.localizedDescription
                            }
                            cliInstalled = CLIInstaller.isInstalled
                        }
                    }
                }
                if let cliError {
                    Text(cliError)
                        .font(.system(size: 11))
                        .foregroundStyle(.red)
                }
                detectedCliRow("jj", icon: "arrow.triangle.branch", status: checkJjEnvironment())
            }

            Section("Forges") {
                detectedCliRow("gh", icon: "arrow.triangle.pull", status: checkGhEnvironment())
                detectedCliRow("glab", icon: "arrow.triangle.merge", status: checkGlabEnvironment())
            }
        }
        .formStyle(.grouped)
    }

    private func detectedCliRow(_ name: String, icon: String, status: CliStatus) -> some View {
        HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if status.isInstalled {
                Text(status.path)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
                    .help("\(name) \(status.version)")
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text("Not installed")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
        }
    }
}
