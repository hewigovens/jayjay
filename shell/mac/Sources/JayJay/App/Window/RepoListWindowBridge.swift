import SwiftUI

struct RepoListWindowBridge: View {
    @Binding var repoPath: String?
    let windowManager: RepoWindowManager

    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Color.clear
            .frame(width: 0, height: 0)
            .onAppear {
                windowManager.setWindowActions(
                    openRepo: { openWindow(id: AppWindows.repo, value: $0) },
                    showRepoList: {
                        repoPath = nil
                        openWindow(id: AppWindows.main)
                    }
                )
            }
    }
}
