extension RepoContentView {
    func showSidebar() {
        guard settings.sidebarHidden else { return }
        settings.sidebarHidden = false
        keyboardFocus.isSidebarHidden = false
        keyboardFocus.activePane = .dag
    }

    func handleSidebarVisibilityChange(hidden: Bool, animated: Bool) {
        if hidden, keyboardFocus.control?.isInSidebar == true {
            windowManager.repoWindow(at: viewModel.repoPath)?.makeFirstResponder(nil)
        }
        keyboardFocus.isSidebarHidden = hidden
        toolbarRoom.isSidebarHidden = hidden
        sidebarReveal.animate(toShown: !hidden, in: windowManager.repoWindow(at: viewModel.repoPath), animated: animated && !reduceMotion)
    }
}
