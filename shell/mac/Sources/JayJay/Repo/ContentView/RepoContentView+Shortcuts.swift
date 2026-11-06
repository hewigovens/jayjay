import AppKit
import JayJayCore

extension RepoContentView {
    private func canPerformShortcut(_ shortcut: AppShortcut) -> Bool {
        guard NSApp.keyWindow?.representedURL?.repositoryLocationPath == viewModel.repoPath || commandPanel.isKeyWindow,
              modal == nil, !viewModel.isLoading else { return false }
        let change = viewModel.selectedChangeId.flatMap { viewModel.change(for: $0) }
        switch shortcut {
            case .fetch: return !viewModel.isPullingInFlight
            case .push: return !viewModel.isPushingInFlight
            case .newChange, .newBookmark, .showEvolution: return change != nil
            case .editChange: return change?.isImmutable == false
            case .describeChange: return change?.isImmutable == false && change?.isWorkingCopy == false
            case .squashChange:
                guard let change, !change.isImmutable, let parent = change.parents.first else { return false }
                return viewModel.change(for: parent)?.isImmutable != true
            default: return true
        }
    }

    func performShortcut(_ shortcut: AppShortcut) {
        guard canPerformShortcut(shortcut) else { return }
        switch shortcut {
            case .refresh: viewModel.refresh()
            case .fetch: viewModel.gitFetch()
            case .push: viewModel.gitPush(bookmark: "")
            case .rebaseOntoTrunk: viewModel.rebase(rev: "@", dest: "trunk()", mode: .branch)
            case .bisect: modal = .bisect(good: "trunk()", bad: "")
            case .cleanUpStaleBookmarks: viewModel.forgetStaleBookmarks()
            case .toggleSideBySide, .toggleIgnoreWhitespace, .toggleTreeFileList:
                performDisplayShortcut(shortcut)
            default: performSelectionShortcut(shortcut)
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

    private func performSelectionShortcut(_ shortcut: AppShortcut) {
        guard let selection = viewModel.selectedChangeId, let change = viewModel.change(for: selection) else { return }
        switch shortcut {
            case .newChange: viewModel.newChange(parent: selection)
            case .editChange: viewModel.edit(rev: selection)
            case .describeChange: modal = .editDescription(rev: selection, description: change.description)
            case .squashChange: viewModel.squash(rev: selection)
            case .newBookmark: presentBookmarkCreate(rev: selection)
            case .showEvolution: viewModel.showEvolog(rev: selection)
            default: break
        }
    }
}
