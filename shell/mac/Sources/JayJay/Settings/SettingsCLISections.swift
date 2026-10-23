import JayJayCore
import SwiftUI

struct SettingsCLISections: View {
    @State private var cliInstalled = CLIInstaller.isInstalled
    @State private var cliError: String?
    let diagnostics: SettingsSnapshot<[String: CliStatus]>
    let forgeLogins: SettingsSnapshot<[RepoHost: HostLogin]>
    @State private var signingIn: RepoHost?
    @State private var signOutError: String?

    var body: some View {
        Group {
            Section(Localized.Settings.versionControl) {
                HStack {
                    SettingsLabel("jayjay", icon: "bird")
                    Spacer()
                    Text(CLIInstaller.installPath)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    if cliInstalled {
                        Button(Localized.Common.uninstall) {
                            try? CLIInstaller.uninstall()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Button(Localized.Common.install) {
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
                    SettingsLabel(Localized.Settings.jjToolConfig, icon: "doc.on.doc")
                    Spacer()
                    Text(Localized.Settings.jjToolConfigDetail)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                    CopyIconButton(
                        value: jjToolConfig(),
                        help: Localized.Settings.copyJjToolConfig,
                        label: Localized.Settings.copyConfig
                    )
                    .accessibilityIdentifier(AID.Settings.copyJJToolConfig)
                }
                detectedCliRow("jj", icon: "arrow.triangle.branch", status: diagnostics.value?["jj"])
            }

            Section(Localized.Settings.forges) {
                ForEach(tokenHosts(), id: \.self) { host in
                    forgeLoginRow(host)
                }
                if let signOutError {
                    Text(signOutError)
                        .font(.system(size: 11))
                        .foregroundStyle(.red)
                }
            }
        }
        .task {
            await diagnostics.load {
                ["jj": checkJjEnvironment()]
            }
        }
        .task { await forgeLogins.load(Self.readLogins) }
        .sheet(item: $signingIn) { host in
            ForgeSignInSheet(
                host: host,
                onDismiss: { signingIn = nil },
                onSignedIn: { Task { await forgeLogins.reload(Self.readLogins) } }
            )
        }
    }

    private static func readLogins() -> [RepoHost: HostLogin] {
        Dictionary(uniqueKeysWithValues: tokenHosts().map { ($0, hostLogin(host: $0)) })
    }

    private func signOut(_ host: RepoHost) {
        signOutError = nil
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { try signOutOfHost(host: host) }
            }.value
            if case let .failure(error) = result {
                signOutError = error.friendlyDescription
            }
            await forgeLogins.reload(Self.readLogins)
        }
    }

    private func forgeLoginRow(_ host: RepoHost) -> some View {
        HStack {
            SettingsLabel(repoHostName(host: host), icon: "person.crop.circle")
            Spacer()
            if let login = forgeLogins.value?[host] {
                if let source = login.source {
                    Text(source)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.secondary)
                } else {
                    Text(Localized.Settings.notSignedIn)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                }
                if login.signedIn {
                    Button(Localized.Common.signOut) { signOut(host) }
                } else if login.canSignIn {
                    Button(Localized.Common.signInEllipsis) {
                        signOutError = nil
                        signingIn = host
                    }
                    .accessibilityIdentifier(AID.Settings.forgeSignIn)
                }
                if login.source != nil {
                    Spacer().frame(width: 8)
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                }
            } else {
                Text(Localized.Common.checking)
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
                Text(status == nil ? Localized.Common.checking : Localized.Common.notInstalled)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
        }
    }
}
