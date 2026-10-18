import JayJayCore
import SwiftUI

struct SettingsCLISections: View {
    @State private var cliInstalled = CLIInstaller.isInstalled
    @State private var cliError: String?
    let diagnostics: SettingsSnapshot<[String: CliStatus]>
    let forgeLogins: SettingsSnapshot<[RepoHost: String]>

    var body: some View {
        Group {
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
                ForEach(tokenHosts(), id: \.self) { host in
                    forgeLoginRow(host)
                }
                detectedCliRow("origin", icon: "arrow.triangle.pull", status: diagnostics.value?["origin"])
            }
        }
        .task {
            await diagnostics.load {
                [
                    "jj": checkJjEnvironment(),
                    "origin": checkOriginEnvironment()
                ]
            }
        }
        .task {
            await forgeLogins.load {
                Dictionary(uniqueKeysWithValues: tokenHosts().compactMap { host in
                    hostTokenSource(host: host).map { (host, $0) }
                })
            }
        }
    }

    private func forgeLoginRow(_ host: RepoHost) -> some View {
        HStack {
            SettingsLabel(repoHostName(host: host), icon: "person.crop.circle")
            Spacer()
            if let source = forgeLogins.value?[host] {
                Text(source)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text(forgeLogins.value == nil ? "Checking…" : "Not signed in")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
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
