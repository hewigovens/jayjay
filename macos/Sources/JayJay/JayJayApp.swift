import SwiftUI

@main
struct JayJayApp: App {
    @State private var repoPath: String?

    var body: some Scene {
        WindowGroup {
            if let path = repoPath {
                RepoWindow(repoPath: path)
            } else {
                WelcomeView(onOpen: { path in
                    repoPath = path
                })
            }
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open Repository...") {
                    openRepo()
                }
                .keyboardShortcut("o")
            }
        }
    }

    private func openRepo() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.message = "Choose a Jujutsu repository"
        if panel.runModal() == .OK, let url = panel.url {
            repoPath = url.path
        }
    }
}

struct WelcomeView: View {
    let onOpen: (String) -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "arrow.triangle.branch")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("JayJay")
                .font(.largeTitle.bold())
            Text("A native GUI for Jujutsu")
                .foregroundStyle(.secondary)
            Button("Open Repository...") {
                let panel = NSOpenPanel()
                panel.canChooseFiles = false
                panel.canChooseDirectories = true
                panel.allowsMultipleSelection = false
                if panel.runModal() == .OK, let url = panel.url {
                    onOpen(url.path)
                }
            }
            .keyboardShortcut(.defaultAction)
        }
        .frame(minWidth: 400, minHeight: 300)
    }
}
