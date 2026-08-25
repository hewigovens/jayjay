import SwiftUI

struct AppCommands: Commands {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Commands {
        CommandGroup(replacing: .windowArrangement) {}
        CommandGroup(replacing: .singleWindowList) {}

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
            .keyboardShortcut("f")
        }

        CommandGroup(after: .textFormatting) {
            Button { settings.fontSize = min(24, settings.fontSize + 1) } label: {
                Label("Zoom In", systemImage: "plus.magnifyingglass")
            }
            .keyboardShortcut("+", modifiers: .command)

            Button { settings.fontSize = max(9, settings.fontSize - 1) } label: {
                Label("Zoom Out", systemImage: "minus.magnifyingglass")
            }
            .keyboardShortcut("-", modifiers: .command)

            Button { settings.fontSize = 12 } label: {
                Label("Reset Zoom", systemImage: "1.magnifyingglass")
            }
            .keyboardShortcut("0", modifiers: .command)
        }

        CommandGroup(replacing: .newItem) {
            Button {
                windowManager.openRepositoryPicker()
            } label: {
                Label("Open Repository...", systemImage: "folder")
            }
            .keyboardShortcut("o")

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
