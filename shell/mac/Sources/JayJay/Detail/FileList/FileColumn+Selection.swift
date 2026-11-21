import AppKit
import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func handleFileColumnKey(_ event: NSEvent) -> Bool {
        switch event.horizontalNavigationKeyCode {
            case KeyCode.space: return toggleReviewOnSelection()
            case KeyCode.leftArrow:
                keyboardFocus?.focusPane(.dag)
                return true
            case KeyCode.rightArrow: return true
            case KeyCode.pageUp, KeyCode.pageDown, KeyCode.home, KeyCode.end: return scrollDiff(event)
            case KeyCode.escape where appSettings.showAllFileDiffs && fileSelection.primary != nil:
                fileSelection = FileSelection()
                return true
            default: break
        }
        if event.opensSelectionContextMenu, let selectedPath = fileSelection.primary, keyboardFocus?.isSuspended != true,
           filteredDiff.contains(where: { $0.path == selectedPath })
        {
            collapsedDirs = collapsedDirs.filter { !selectedPath.hasPrefix("\($0)/") }
            contextMenuRequest = SelectionContextMenuRequest(target: selectedPath)
            return true
        }
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if modifiers == .command, event.charactersIgnoringModifiers == "a" {
            return selectAllFiles()
        }
        let isCtrl = modifiers == .control
        guard let delta = DAGViewModel.selectionDelta(
            keyCode: event.keyCode,
            charactersIgnoringModifiers: event.charactersIgnoringModifiers,
            controlPressed: isCtrl
        ) else { return false }
        return moveFileSelection(by: delta)
    }

    private func scrollDiff(_ event: NSEvent) -> Bool {
        guard event.modifierFlags.isDisjoint(with: [.command, .control, .option, .shift]) else { return false }
        if isStacked {
            switch event.keyCode {
                case KeyCode.pageUp: stackScroll.scrollView?.pageUp(nil)
                case KeyCode.pageDown: stackScroll.scrollView?.pageDown(nil)
                default: stackScrollEdge = event.keyCode == KeyCode.end ? .bottom : .top
            }
            return true
        }
        guard let diff = NSApp.keyWindow?.contentView?.diffTextView else { return false }
        switch event.keyCode {
            case KeyCode.pageUp: diff.scrollPageUp(nil)
            case KeyCode.pageDown: diff.scrollPageDown(nil)
            case KeyCode.home: diff.scrollToBeginningOfDocument(nil)
            default: diff.scrollToEndOfDocument(nil)
        }
        return true
    }

    private func toggleReviewOnSelection() -> Bool {
        reconcileFileSelection()
        guard showsReviewMarks else { return false }
        let selected = fileSelection.paths
        let paths = reviewableDiff.compactMap { selected.contains($0.path) ? $0.path : nil }
        guard !paths.isEmpty else { return false }
        toggleReview(paths)
        return true
    }

    /// Marking the selected file reviewed moves on to the next unreviewed file after it, so files the reviewer skipped stay behind.
    func toggleReview(_ paths: [String]) {
        let order = displayOrderedPaths
        let reviewed = !paths.allSatisfy { fileRollups[$0] == .reviewed }
        applyReviewMarks(paths: paths, reviewed: reviewed)
        guard reviewed, let selectedPath = fileSelection.primary, paths.contains(selectedPath),
              let position = order.firstIndex(of: selectedPath)
        else { return }
        let following = order[(position + 1)...] + order[..<position]
        guard let next = following.first(where: { !reviewedPaths.contains($0) }) else { return }
        fileSelection.reveal(next, expanding: &collapsedDirs)
    }

    func reconcileFileSelection() {
        let paths = filteredDiff.map(\.path)
        let available = Set(paths)
        var selection = fileSelection
        selection.paths.formIntersection(available)
        if appSettings.showAllFileDiffs, selection.paths.isEmpty {
            fileSelection = FileSelection()
            return
        }
        if selection.primary.map({ available.contains($0) }) != true {
            let order = visibleSelectablePaths
            selection.primary = order.first(where: selection.paths.contains) ?? order.first ?? paths.first
        }
        if let primary = selection.primary {
            selection.paths.insert(primary)
        }
        if selection.anchor.map({ available.contains($0) }) != true {
            selection.anchor = selection.primary
        }
        fileSelection = selection
    }

    @discardableResult
    func moveFileSelection(by delta: Int) -> Bool {
        let paths = visibleSelectablePaths
        guard !paths.isEmpty else { return false }
        let currentIdx: Int = if let cur = fileSelection.primary, let idx = paths.firstIndex(of: cur) {
            idx
        } else {
            delta > 0 ? -1 : paths.count
        }
        let newIdx = max(0, min(paths.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return false }
        selectSingleFile(paths[newIdx])
        return true
    }

    /// Tree mode includes files under collapsed folders.
    var displayOrderedPaths: [String] {
        let paths = filteredDiff.map(\.path)
        guard appSettings.treeFileList else { return paths }
        return fileTreeCache.entries(for: paths).compactMap { $0.hunkIndex == nil ? nil : $0.path }
    }

    private func isInCollapsedDir(_ path: String) -> Bool {
        appSettings.treeFileList && collapsedDirs.contains { path.hasPrefix("\($0)/") }
    }

    var visibleSelectablePaths: [String] {
        displayOrderedPaths.filter { !isInCollapsedDir($0) }
    }

    func contextSelectionPaths(for clickedPath: String) -> [String] {
        let activeSelection: Set<String> =
            if fileSelection.paths.contains(clickedPath), fileSelection.paths.count > 1 {
                fileSelection.paths
            } else {
                [clickedPath]
            }
        return visibleSelectablePaths.filter(activeSelection.contains)
    }

    func handleFileSelection(_ path: String) {
        fileSelection = FileSelection(applySelectionClick(
            selection: fileSelection.ordered,
            click: SelectionClick(modifiers: NSEvent.modifierFlags),
            id: path,
            order: visibleSelectablePaths
        ))
    }

    private func selectAllFiles() -> Bool {
        let paths = visibleSelectablePaths
        guard let primary = fileSelection.primary.flatMap({ paths.contains($0) ? $0 : nil }) ?? paths.first else { return false }
        fileSelection = FileSelection(primary: primary, paths: Set(paths), anchor: primary)
        return true
    }

    func selectSingleFile(_ path: String) {
        fileSelection = .single(path)
    }
}
