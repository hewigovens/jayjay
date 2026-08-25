import AppKit
import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func handleFileColumnKey(_ event: NSEvent) -> Bool {
        if event.keyCode == KeyCode.space {
            return toggleReviewOnSelection()
        }
        switch event.keyCode {
            case KeyCode.downArrow: return moveFileSelection(by: 1)
            case KeyCode.upArrow: return moveFileSelection(by: -1)
            default: break
        }
        let isCtrl = event.modifierFlags.intersection(.deviceIndependentFlagsMask) == .control
        switch event.charactersIgnoringModifiers {
            case "j": return moveFileSelection(by: 1)
            case "k": return moveFileSelection(by: -1)
            case "n" where isCtrl: return moveFileSelection(by: 1)
            case "p" where isCtrl: return moveFileSelection(by: -1)
            default: return false
        }
    }

    private func toggleReviewOnSelection() -> Bool {
        let selectedReviewablePaths = selectedPaths
            .filter { path in reviewableDiff.contains(where: { $0.path == path }) }
            .sorted()
        guard showsReviewControls, !selectedReviewablePaths.isEmpty else { return false }
        applyReviewMarks(
            paths: selectedReviewablePaths,
            reviewed: !selectedReviewablePaths.allSatisfy { fileRollups[$0] == .reviewed }
        )
        if let primaryPath = selectedPath,
           reviewedPaths.contains(primaryPath),
           let next = filteredDiff.first(where: { !reviewedPaths.contains($0.path) })
        {
            selectedPath = next.path
            selectedPaths = [next.path]
            fileSelectionAnchorPath = next.path
        }
        return true
    }

    @discardableResult
    func moveFileSelection(by delta: Int) -> Bool {
        guard !filteredDiff.isEmpty else { return false }
        let currentIdx: Int = if let cur = selectedPath, let idx = filteredDiff.firstIndex(where: { $0.path == cur }) {
            idx
        } else {
            delta > 0 ? -1 : filteredDiff.count
        }
        let newIdx = max(0, min(filteredDiff.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return false }
        let nextPath = filteredDiff[newIdx].path
        selectedPath = nextPath
        selectedPaths = [nextPath]
        fileSelectionAnchorPath = nextPath
        return true
    }

    private var visibleSelectablePaths: [String] {
        if appSettings.treeFileList {
            let visibleHunks = filteredDiff
            let entries = buildFileTree(paths: visibleHunks.map(\.path))
            return entries.compactMap { entry in
                guard let hunkIndex = entry.hunkIndex, Int(hunkIndex) < visibleHunks.count else { return nil }
                return visibleHunks[Int(hunkIndex)].path
            }
        }
        return filteredDiff.map(\.path)
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
        let modifiers = NSEvent.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let orderedPaths = visibleSelectablePaths

        if modifiers.contains(.shift),
           let anchor = fileSelectionAnchorPath,
           let anchorIndex = orderedPaths.firstIndex(of: anchor),
           let currentIndex = orderedPaths.firstIndex(of: path)
        {
            let lower = min(anchorIndex, currentIndex)
            let upper = max(anchorIndex, currentIndex)
            selectedPaths = Set(orderedPaths[lower ... upper])
        } else {
            selectedPaths = [path]
            fileSelectionAnchorPath = path
        }

        selectedPath = path
    }

    func selectSingleFile(_ path: String) {
        selectedPath = path
        selectedPaths = [path]
        fileSelectionAnchorPath = path
    }
}
