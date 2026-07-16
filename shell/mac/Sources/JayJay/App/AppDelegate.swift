import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var openRepositoryPicker: (() -> Void)?
    var openHandler: ((String) -> Void)?
    var showRepoSelector: (() -> Void)?
    var recentReposProvider: (() -> [String])?
    private var dockMenuActions: [DockMenuAction] = []

    func applicationDidFinishLaunching(_ notification: Notification) {
        DockIcon.install()
        Task.detached { CLIInstaller.refreshLinkIfInstalled() }
    }

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        dockMenuActions.removeAll(keepingCapacity: true)
        let menu = NSMenu()
        let openItem = dockMenuItem(title: "Open Repository...") { [weak self] in
            self?.openRepositoryPicker?()
        }
        menu.addItem(openItem)

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
