import SwiftUI

struct AppCommands: Commands {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Commands {
        CommandGroup(replacing: .windowArrangement) {}
        CommandGroup(replacing: .singleWindowList) {}

        CommandGroup(after: .sidebar) {
            Button { settings.sidebarHidden.toggle() } label: {
                Label(settings.sidebarHidden ? "Show Sidebar" : "Hide Sidebar", systemImage: "sidebar.leading")
            }
            .keyboardShortcut(AppShortcut.toggleSidebar)
            .disabled(ActiveRepoTracker.shared.handler == nil)
        }

        CommandGroup(after: .pasteboard) {
            Button {
                if let window = NSApp.keyWindow,
                   let tv = findDiffTextView(in: window.contentView)
                {
                    window.makeFirstResponder(tv)
                    let item = NSMenuItem()
                    item.tag = Int(NSFindPanelAction.showFindPanel.rawValue)
                    tv.performFindPanelAction(item)
                }
            } label: {
                Label("Find...", systemImage: "magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.findInDiff)
        }

        CommandGroup(after: .textFormatting) {
            Button { settings.zoomIn() } label: {
                Label("Zoom In", systemImage: "plus.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.zoomIn)

            Button { settings.zoomOut() } label: {
                Label("Zoom Out", systemImage: "minus.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.zoomOut)

            Button { settings.resetZoom() } label: {
                Label("Reset Zoom", systemImage: "1.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.resetZoom)
        }

        CommandGroup(replacing: .newItem) {
            Button {
                windowManager.openRepositoryPicker()
            } label: {
                Label("Open Repository...", systemImage: "folder")
            }
            .keyboardShortcut(AppShortcut.openRepository)

            Menu {
                if settings.recentRepos.isEmpty {
                    Text("No Recent Repositories")
                } else {
                    ForEach(settings.recentRepos, id: \.self) { path in
                        Button {
                            windowManager.openRepo(path)
                        } label: {
                            Label(
                                URL(fileURLWithPath: path).repositoryDisplayName,
                                systemImage: "arrow.triangle.branch"
                            )
                        }
                    }

                    Divider()

                    Button {
                        settings.recentRepos = []
                        settings.lastOpenedRepo = nil
                    } label: {
                        Label("Clear", systemImage: "trash")
                    }
                }
            } label: {
                Label("Open Recent", systemImage: "clock")
            }
        }
    }

    private static let diffTextViewID = NSUserInterfaceItemIdentifier("diffTextView")

    private func findDiffTextView(in view: NSView?) -> NSTextView? {
        guard let view else { return nil }
        if let tv = view as? NSTextView, tv.identifier == Self.diffTextViewID {
            return tv
        }
        for sub in view.subviews {
            if let found = findDiffTextView(in: sub) {
                return found
            }
        }
        return nil
    }
}
