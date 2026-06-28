import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var openRepositoryPicker: (() -> Void)?
    var openHandler: ((String) -> Void)?
    var showRepoSelector: (() -> Void)?
    var recentReposProvider: (() -> [String])?

    func applicationDidFinishLaunching(_ notification: Notification) {
        Task.detached { CLIInstaller.refreshLinkIfInstalled() }
    }

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        let menu = NSMenu()
        let openItem = NSMenuItem(
            title: "Open Repository...",
            action: #selector(dockMenuOpenRepositoryPicker),
            keyEquivalent: ""
        )
        openItem.target = self
        menu.addItem(openItem)

        let repos = recentReposProvider?() ?? []
        guard !repos.isEmpty else { return menu }

        menu.addItem(.separator())
        let submenu = NSMenu(title: "Recent Repositories")
        for path in repos {
            let name = URL(fileURLWithPath: path).lastPathComponent
            let item = NSMenuItem(title: name, action: #selector(dockMenuOpenRepo(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = path
            submenu.addItem(item)
        }
        let recentItem = NSMenuItem(title: "Recent Repositories", action: nil, keyEquivalent: "")
        recentItem.submenu = submenu
        menu.addItem(recentItem)
        return menu
    }

    @objc private func dockMenuOpenRepositoryPicker() {
        openRepositoryPicker?()
    }

    @objc private func dockMenuOpenRepo(_ sender: NSMenuItem) {
        guard let path = sender.representedObject as? String else { return }
        openHandler?(path)
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
        return true
    }
}
