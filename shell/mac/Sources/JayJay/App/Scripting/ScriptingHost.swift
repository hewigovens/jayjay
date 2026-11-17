/// NSApp.delegate is SwiftUI's wrapper, so script commands reach the window manager here instead of casting the delegate.
@MainActor
enum ScriptingHost {
    static weak var windowManager: RepoWindowManager?
}
