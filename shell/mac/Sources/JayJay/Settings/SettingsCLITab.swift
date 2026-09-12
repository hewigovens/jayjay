import JayJayCore
import SwiftUI

struct SettingsCLITab: View {
    @State private var cliInstalled = CLIInstaller.isInstalled
    @State private var cliError: String?
    @State private var diagnostics = SettingsSnapshot<[String: CliStatus]>()

    var body: some View {
        Form {
            Section("Version control") {
                HStack {
                    SettingsLabel("jayjay", icon: "bird")
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
                HStack {
                    SettingsLabel("jj tool configuration", icon: "doc.on.doc")
                    Spacer()
                    Text("diff, edit & merge")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                    CopyIconButton(
                        value: jjToolConfig(),
                        help: "Copy jj tool configuration",
                        label: "Copy Config"
                    )
                    .accessibilityIdentifier(AID.Settings.copyJJToolConfig)
                }
                detectedCliRow("jj", icon: "arrow.triangle.branch", status: diagnostics.value?["jj"])
            }

            Section("Forges") {
                detectedCliRow("gh", icon: "arrow.triangle.pull", status: diagnostics.value?["gh"])
                detectedCliRow("glab", icon: "arrow.triangle.merge", status: diagnostics.value?["glab"])
                detectedCliRow("origin", icon: "arrow.triangle.pull", status: diagnostics.value?["origin"])
            }
        }
        .formStyle(.grouped)
        .task {
            await diagnostics.load {
                [
                    "jj": checkJjEnvironment(),
                    "gh": checkGhEnvironment(),
                    "glab": checkGlabEnvironment(),
                    "origin": checkOriginEnvironment()
                ]
            }
        }
    }

    private func detectedCliRow(_ name: String, icon: String, status: CliStatus?) -> some View {
        HStack {
            SettingsLabel(name, icon: icon)
            Spacer()
            if let status, status.isInstalled {
                Text(status.path)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
                    .help("\(name) \(status.version)")
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text(status == nil ? "Checking…" : "Not installed")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
        }
    }
}
