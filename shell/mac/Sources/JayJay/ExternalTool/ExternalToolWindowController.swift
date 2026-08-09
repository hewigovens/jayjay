import AppKit
import JayJayCore
import SwiftUI

@MainActor
final class ExternalToolExitState {
    private var failedToLoad = false

    func markLoadFailure() {
        failedToLoad = true
    }

    func exitCode(for invocation: ExternalToolInvocation) -> Int32 {
        failedToLoad ? 1 : externalToolCancelExitCode(invocation: invocation)
    }
}

@MainActor
final class ExternalToolWindowController: NSWindowController, NSWindowDelegate {
    private static let preferredContentSize = NSSize(width: 1180, height: 820)
    private let invocation: ExternalToolInvocation
    private let exitState: ExternalToolExitState

    init(invocation: ExternalToolInvocation) {
        let exitState = ExternalToolExitState()
        self.invocation = invocation
        self.exitState = exitState
        let hostingController = NSHostingController(rootView: ExternalToolRootView(
            invocation: invocation,
            onLoadFailure: exitState.markLoadFailure
        ))
        let window = NSWindow(contentViewController: hostingController)
        window.title = invocation.windowTitle
        window.identifier = NSUserInterfaceItemIdentifier("external-tool")
        window.styleMask = [.titled, .closable, .miniaturizable, .resizable]
        window.contentMinSize = NSSize(width: 900, height: 620)
        window.setContentSize(Self.preferredContentSize)
        window.center()
        window.isReleasedWhenClosed = false
        super.init(window: window)
        window.delegate = self
    }

    var cancelExitCode: Int32 {
        exitState.exitCode(for: invocation)
    }

    /// jj blocks on our exit status, so closing the tool window ends the session even while other windows are open.
    func windowWillClose(_ notification: Notification) {
        NSApp.terminate(nil)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func present() {
        guard let window else { return }
        if let visibleFrame = NSScreen.main?.visibleFrame {
            window.setFrame(
                WindowContentSizer.fittedFrame(window.frame, within: visibleFrame),
                display: false
            )
        }
        showWindow(nil)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}
