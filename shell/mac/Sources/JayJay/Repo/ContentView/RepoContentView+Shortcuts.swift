import AppKit
import JayJayCore

extension RepoContentView {
    /// The shortcuts that would act now; the Repository menu reads these for the window it targets.
    func availableShortcuts(among shortcuts: [AppShortcut]) -> Set<AppShortcut> {
        guard modal == nil, !viewModel.isLoading else { return [] }
        let selected = viewModel.selectedChangeId.flatMap(viewModel.change(for:))
        let selectedCommands = selected.map { availableChangeCommands(change: $0) } ?? []
        return Set(shortcuts.filter { shortcut in
            if shortcut == .rebaseOntoTrunk, viewModel.selectedChangeIds.count > 1 {
                return viewModel.selectionCapabilities.isAvailable(.rebaseOntoTrunk)
            }
            if let command = shortcut.changeCommand {
                guard selected == nil else { return selectedCommands.contains(command) }
                return shortcutTarget(for: command).map { availableChangeCommands(change: $0).contains(command) } ?? false
            }
            switch shortcut {
                case .fetch: return !viewModel.isPullingInFlight
                case .push: return !viewModel.isPushingInFlight
                default: return true
            }
        })
    }

    func performShortcut(_ shortcut: AppShortcut) {
        guard NSApp.keyWindow?.representedURL?.repositoryLocationPath == viewModel.repoPath || commandPanel.isKeyWindow,
              availableShortcuts(among: [shortcut]).contains(shortcut) else { return }
        if shortcut == .rebaseOntoTrunk, viewModel.selectedChangeIds.count > 1 {
            handleDAGRequest(.rebaseSelectionOntoTrunk(revisions: viewModel.selectedChangeIds))
            return
        }
        if let command = shortcut.changeCommand {
            if let change = shortcutTarget(for: command) {
                command.perform(on: change, actions: viewModel, onRequest: handleDAGRequest)
            }
            return
        }
        switch shortcut {
            case .refresh: viewModel.refresh()
            case .fetch: viewModel.gitFetch()
            case .push: viewModel.gitPush(bookmark: "")
            case .bisect: modal = .bisect(good: "trunk()", bad: "")
            case .cleanUpStaleBookmarks: viewModel.forgetStaleBookmarks()
            default: performDisplayShortcut(shortcut)
        }
    }

    private func performDisplayShortcut(_ shortcut: AppShortcut) {
        switch shortcut {
            case .toggleSideBySide: settings.sideBySideDiff.toggle()
            case .toggleIgnoreWhitespace: settings.ignoreWhitespace.toggle()
            case .toggleTreeFileList: settings.treeFileList.toggle()
            default: break
        }
    }

    /// Rebase onto Trunk falls back to the working copy when nothing is selected; the other change commands need a selection.
    private func shortcutTarget(for command: ChangeCommand) -> ChangeInfo? {
        viewModel.selectedChangeId.flatMap(viewModel.change(for:))
            ?? (command == .rebaseOntoTrunk ? viewModel.changes.first(where: \.isWorkingCopy) : nil)
    }
}
