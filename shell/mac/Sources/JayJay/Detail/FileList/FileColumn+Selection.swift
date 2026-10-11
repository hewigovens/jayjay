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
            default: break
        }
        let isCtrl = event.modifierFlags.intersection(.deviceIndependentFlagsMask) == .control
        guard let delta = DAGViewModel.selectionDelta(
            keyCode: event.keyCode,
            charactersIgnoringModifiers: event.charactersIgnoringModifiers,
            controlPressed: isCtrl
        ) else { return false }
        return moveFileSelection(by: delta)
    }

    private func toggleReviewOnSelection() -> Bool {
        reconcileFileSelection()
        let selectedReviewablePaths = selectedPaths
            .filter { path in reviewableDiff.contains(where: { $0.path == path }) }
            .sorted()
        guard showsReviewControls, !selectedReviewablePaths.isEmpty else { return false }
        applyReviewMarks(
            paths: selectedReviewablePaths,
            reviewed: !selectedReviewablePaths.allSatisfy { fileRollups[$0] == .reviewed }
        )
        if let primaryPath = selectedPath, reviewedPaths.contains(primaryPath) {
            let unreviewed = displayOrderedPaths.filter { !reviewedPaths.contains($0) }
            if let next = unreviewed.first(where: { !isInCollapsedDir($0) }) ?? unreviewed.first {
                if isInCollapsedDir(next) {
                    collapsedDirs = collapsedDirs.filter { !next.hasPrefix("\($0)/") }
                }
                selectSingleFile(next)
            }
        }
        return true
    }

    func reconcileFileSelection() {
        let paths = filteredDiff.map(\.path)
        let available = Set(paths)
        selectedPaths.formIntersection(available)
        if selectedPath.map({ available.contains($0) }) != true {
            let order = visibleSelectablePaths
            selectedPath = order.first(where: selectedPaths.contains) ?? order.first ?? paths.first
        }
        if let selectedPath {
            selectedPaths.insert(selectedPath)
        }
        if fileSelectionAnchorPath.map({ available.contains($0) }) != true {
            fileSelectionAnchorPath = selectedPath
        }
    }

    @discardableResult
    func moveFileSelection(by delta: Int) -> Bool {
        let paths = visibleSelectablePaths
        guard !paths.isEmpty else { return false }
        let currentIdx: Int = if let cur = selectedPath, let idx = paths.firstIndex(of: cur) {
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
    private var displayOrderedPaths: [String] {
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
            if selectedPaths.contains(clickedPath), selectedPaths.count > 1 {
                selectedPaths
            } else {
                [clickedPath]
            }
        return visibleSelectablePaths.filter(activeSelection.contains)
    }

    func handleFileSelection(_ path: String) {
        let selection = applySelectionClick(
            selection: orderedSelection(
                selected: Array(selectedPaths),
                primary: selectedPath,
                anchor: fileSelectionAnchorPath
            ),
            click: SelectionClick(modifiers: NSEvent.modifierFlags),
            id: path,
            order: visibleSelectablePaths
        )
        selectedPaths = Set(selection.selected)
        selectedPath = selection.primary
        fileSelectionAnchorPath = selection.anchor
    }

    func selectSingleFile(_ path: String) {
        selectedPath = path
        selectedPaths = [path]
        fileSelectionAnchorPath = path
    }
}
