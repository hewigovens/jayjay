import AppKit
import JayJayCore

class AppDelegate: NSObject, NSApplicationDelegate {
    var openRepositoryPicker: (() -> Void)?
    var openHandler: ((String) -> Void)?
    var showRepoSelector: (() -> Void)?
    var recentReposProvider: (() -> [String])?
    var terminateAfterLastWindowClosed = false
    var externalToolInvocation: ExternalToolInvocation?
    private var dockMenuActions: [DockMenuAction] = []
    private var externalToolWindowController: ExternalToolWindowController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        DockIcon.install()
        Task.detached { CLIInstaller.refreshLinkIfInstalled() }
        if let invocation = externalToolInvocation {
            terminateAfterLastWindowClosed = true
            let controller = ExternalToolWindowController(invocation: invocation)
            externalToolWindowController = controller
            controller.present()
        }
    }

    func applicationDidUpdate(_ notification: Notification) {
        guard let toolWindow = externalToolWindowController?.window else { return }
        for window in NSApp.windows where window !== toolWindow {
            window.close()
        }
    }

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        dockMenuActions.removeAll(keepingCapacity: true)
        let menu = NSMenu()
        let openItem = dockMenuItem(title: "Open Repository...") { [weak self] in
            self?.openRepositoryPicker?()
        }
        menu.addItem(openItem)
        let listItem = dockMenuItem(title: "Repository List...") { [weak self] in
            self?.showRepoSelector?()
        }
        menu.addItem(listItem)

        let repos = recentReposProvider?() ?? []
        guard !repos.isEmpty else { return menu }

        menu.addItem(.separator())
        let submenu = NSMenu(title: "Recent Repositories")
        for path in repos {
            let name = URL(fileURLWithPath: path).repositoryDisplayName
            let item = dockMenuItem(title: name) { [weak self] in
                self?.openHandler?(path)
            }
            submenu.addItem(item)
        }
        let recentItem = NSMenuItem(title: "Recent Repositories", action: nil, keyEquivalent: "")
        recentItem.submenu = submenu
        menu.addItem(recentItem)
        return menu
    }

    private func dockMenuItem(title: String, action: @escaping () -> Void) -> NSMenuItem {
        let target = DockMenuAction(action: action)
        dockMenuActions.append(target)
        let item = NSMenuItem(title: title, action: #selector(DockMenuAction.invoke), keyEquivalent: "")
        item.target = target
        return item
    }

    func application(_ application: NSApplication, open urls: [URL]) {
        for url in urls {
            guard url.scheme == URLScheme.scheme, url.host == URLScheme.hostOpen,
                  let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                  let path = components.queryItems?.first(where: { $0.name == URLScheme.paramPath })?.value
            else { continue }
            openHandler?(path)
        }
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            showRepoSelector?()
        }
        return false
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        terminateAfterLastWindowClosed
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        // In a tool session jj interprets our exit status; any exit that is not an explicit save must report the cancel code.
        if let controller = externalToolWindowController {
            Darwin.exit(controller.cancelExitCode)
        }
        if let invocation = externalToolInvocation {
            Darwin.exit(externalToolCancelExitCode(invocation: invocation))
        }
        return .terminateNow
    }
}

private final class DockMenuAction: NSObject {
    private let action: () -> Void

    init(action: @escaping () -> Void) {
        self.action = action
    }

    @objc func invoke() {
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.main.async(execute: action)
    }
}
