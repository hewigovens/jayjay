import AppKit
import SwiftUI

struct OverviewWindow: View {
    let repoPath: String
    @State private var viewModel: OverviewViewModel
    @State private var windowNumber: Int?
    @Environment(RepoWindowManager.self) private var windowManager

    init(repoPath: String) {
        self.repoPath = repoPath
        _viewModel = State(initialValue: OverviewViewModel(repoPath: repoPath))
    }

    var body: some View {
        OverviewView(viewModel: viewModel)
            .frame(minWidth: 900, minHeight: 500)
            .task {
                viewModel.hasRepoWindow = { windowManager.repoWindow(at: repoPath) != nil }
                await viewModel.open()
            }
            .navigationTitle("Repo Overview")
            .navigationSubtitle(URL(fileURLWithPath: repoPath).repositoryDisplayName)
            .background(WindowFramePersistence(key: AppWindows.overview))
            // The URL feeds the Repository menu; the registrations keep this window out of repo-window lookups.
            .background(WindowConfigurator { window in
                window.representedURL = URL(fileURLWithPath: repoPath)
                windowNumber = window.windowNumber
                windowManager.overviewWindowDidAppear(window, for: repoPath)
                ActiveRepoTracker.shared.registerOverview(window)
            })
            .onReceive(NotificationCenter.default.publisher(for: NSWindow.willCloseNotification)) { notification in
                guard let closing = notification.object as? NSWindow, closing.windowNumber == windowNumber else { return }
                viewModel.close()
            }
            .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
                viewModel.load()
            }
    }
}
