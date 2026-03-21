import AppKit
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private let settings: AppSettings
    private var controllers: [String: RepoHostWindowController] = [:]
    /// Path of the repo in the main SwiftUI WindowGroup (if any)
    var mainWindowPath: String?

    init(settings: AppSettings) {
        self.settings = settings
    }

    func openRepo(_ path: String) {
        let normalizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        settings.recordOpenedRepo(normalizedPath)

        // If this is the main window's repo, just activate
        if normalizedPath == mainWindowPath {
            NSApp.activate(ignoringOtherApps: true)
            return
        }

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
        // Force window size after SwiftUI has laid out (it tries to shrink to intrinsic size)
        DispatchQueue.main.async {
            guard let window = controller.window else { return }
            window.setContentSize(NSSize(width: 1360, height: 860))
            window.center()
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        }
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

        let hostingView = NSHostingView(rootView: rootView)
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1360, height: 860),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window.contentView = hostingView
        window.minSize = NSSize(width: 900, height: 500)
        window.title = URL(fileURLWithPath: repoPath).lastPathComponent
        window.titleVisibility = .visible
        window.toolbarStyle = .unifiedCompact
        window.toolbar = NSToolbar()
        window.toolbar?.displayMode = .iconOnly
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
