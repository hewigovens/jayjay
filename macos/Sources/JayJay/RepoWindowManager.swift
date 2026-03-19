import AppKit
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private let settings: AppSettings
    private var controllers: [String: RepoHostWindowController] = [:]

    init(settings: AppSettings) {
        self.settings = settings
    }

    func openRepo(_ path: String) {
        let normalizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        settings.recordOpenedRepo(normalizedPath)

        if let existing = controllers[normalizedPath] {
            existing.showWindow(nil)
            existing.window?.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let windowView = ThemedRepoRootView(repoPath: normalizedPath)
            .environment(settings)
            .environment(self)

        let controller = RepoHostWindowController(
            repoPath: normalizedPath,
            rootView: windowView
        ) { [weak self] closedPath in
            self?.controllers.removeValue(forKey: closedPath)
        }

        controllers[normalizedPath] = controller
        controller.showWindow(nil)
        controller.window?.center()
        controller.window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}

private struct ThemedRepoRootView: View {
    let repoPath: String

    @Environment(AppSettings.self) private var settings

    var body: some View {
        RepoWindow(repoPath: repoPath)
            .environment(\.jayjayFontScale, settings.fontScale)
            .preferredColorScheme(settings.appearanceMode.colorScheme)
    }
}

private final class RepoHostWindowController: NSWindowController, NSWindowDelegate {
    private let repoPath: String
    private let onClose: (String) -> Void

    init<Content: View>(
        repoPath: String,
        rootView: Content,
        onClose: @escaping (String) -> Void
    ) {
        self.repoPath = repoPath
        self.onClose = onClose

        let hostingController = NSHostingController(rootView: rootView)
        let window = NSWindow(contentViewController: hostingController)
        window.setContentSize(NSSize(width: 1360, height: 860))
        window.title = URL(fileURLWithPath: repoPath).lastPathComponent
        window.titleVisibility = .visible
        window.toolbarStyle = .unifiedCompact
        window.delegate = nil

        super.init(window: window)
        window.delegate = self
        shouldCascadeWindows = true
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        nil
    }

    func windowWillClose(_ notification: Notification) {
        onClose(repoPath)
    }
}
