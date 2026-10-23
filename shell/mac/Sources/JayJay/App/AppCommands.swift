import JayJayCore
import SwiftUI

struct AppCommands: Commands {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Commands {
        CommandGroup(replacing: .windowArrangement) {}
        CommandGroup(replacing: .singleWindowList) {}

        CommandGroup(after: .sidebar) {
            Button { settings.sidebarHidden.toggle() } label: {
                Label(settings.sidebarHidden ? Localized.Menu.showSidebar : Localized.Menu.hideSidebar, systemImage: "sidebar.leading")
            }
            .keyboardShortcut(AppShortcut.toggleSidebar)
            .disabled(ActiveRepoTracker.shared.handler == nil)

            Divider()

            Button { settings.zoomIn() } label: {
                Label(Localized.Menu.zoomIn, systemImage: "plus.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.zoomIn)

            Button { settings.zoomOut() } label: {
                Label(Localized.Menu.zoomOut, systemImage: "minus.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.zoomOut)

            Button { settings.resetZoom() } label: {
                Label(Localized.Menu.resetZoom, systemImage: "1.magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.resetZoom)
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
                Label(Localized.Menu.findEllipsis, systemImage: "magnifyingglass")
            }
            .keyboardShortcut(AppShortcut.findInDiff)
        }

        CommandGroup(replacing: .newItem) {
            Button {
                windowManager.openRepositoryPicker()
            } label: {
                Label(Localized.Menu.openRepositoryEllipsis, systemImage: "folder")
            }
            .keyboardShortcut(AppShortcut.openRepository)

            Button {
                windowManager.showCloneRepository()
            } label: {
                Label(Localized.Menu.cloneRepositoryEllipsis, systemImage: "square.and.arrow.down")
            }

            Menu {
                if settings.recentRepos.isEmpty {
                    Text(Localized.Menu.noRecentRepositories)
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
                        Label(Localized.Common.clear, systemImage: "trash")
                    }
                }
            } label: {
                Label(Localized.Menu.openRecent, systemImage: "clock")
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
