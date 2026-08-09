import AppKit
import Darwin
import Foundation
import JayJayCore
import JayJayDiffUI
import Observation

@MainActor
@Observable
final class ExternalDiffSession {
    let left: String
    let right: String
    let editable: Bool

    var files: [ExternalDiffFileState] = []
    var isLoading = true
    var isSaving = false
    var errorMessage: String?
    private let onLoadFailure: () -> Void

    var canSave: Bool {
        editable && !isLoading && !isSaving && errorMessage == nil
    }

    init(left: String, right: String, editable: Bool, onLoadFailure: @escaping () -> Void = {}) {
        self.left = left
        self.right = right
        self.editable = editable
        self.onLoadFailure = onLoadFailure
    }

    func load() async {
        guard files.isEmpty else { return }
        isLoading = true
        do {
            let left = left
            let right = right
            let editable = editable
            let loaded = try await Task.detached {
                try loadExternalDiff(left: left, right: right, editable: editable)
            }.value
            let states = await Task.detached {
                loaded.map(ExternalDiffFileState.init)
            }.value
            let stats = states.map(\.stats)
            let collapsed = Set(diffEditAutoCollapsedPaths(stats: stats))
            for state in states {
                state.isCollapsed = collapsed.contains(state.hunk.path)
            }
            files = states
        } catch {
            errorMessage = error.localizedDescription
            onLoadFailure()
        }
        isLoading = false
    }

    func save() {
        guard canSave else { return }
        isSaving = true
        errorMessage = nil
        let selections = files.compactMap(\.selection)
        Task {
            do {
                try await Task.detached {
                    try applyExternalDiff(
                        left: self.left,
                        right: self.right,
                        selections: selections,
                        ignoreWhitespace: false
                    )
                }.value
                Darwin.exit(0)
            } catch {
                errorMessage = error.localizedDescription
                isSaving = false
            }
        }
    }

    func cancel() {
        NSApp.terminate(nil)
    }

    func toggleFile(_ selected: ExternalDiffFileState) {
        let side: ExternalDiffSide = selected.keepsAllChanges ? .old : .new
        for file in files where file === selected
            || selected.topologyGroup.map({ file.topologyGroup == $0 }) == true
        {
            file.selectSide(side)
        }
    }
}

@Observable
final class ExternalDiffFileState: Identifiable {
    enum ExecutableMode: Equatable {
        case unavailable
        case disabled
        case enabled
    }

    let hunk: DiffHunk
    let topologyGroup: String?
    let displayDiff: FileDiff
    let displayToFull: [Int: Int]
    let changedLines: Set<Int>
    let supportsEditing: Bool
    let oldExists: Bool
    let newExists: Bool
    let oldExecutable: ExecutableMode
    let newExecutable: ExecutableMode
    let stats: FileDiffStats

    var selectedLines: Set<Int>
    var selectedExists: Bool
    var selectedExecutable: ExecutableMode
    var wholeFileSide: ExternalDiffSide?
    var isCollapsed = false
    var measuredHeight: CGFloat?

    var id: String {
        hunk.path
    }

    init(file: ExternalDiffFile) {
        hunk = file.hunk
        topologyGroup = file.topologyGroup
        displayDiff = file.displayDiff
        displayToFull = Dictionary(uniqueKeysWithValues: file.displayToFull.map {
            (Int($0.displayLine), Int($0.fullLine))
        })
        changedLines = Set(file.changedLines.map(Int.init))
        selectedLines = changedLines
        supportsEditing = file.supportsEditing
        oldExists = file.oldExists
        newExists = file.newExists
        selectedExists = file.newExists
        oldExecutable = switch file.oldExecutable {
            case true?: .enabled
            case false?: .disabled
            case nil: .unavailable
        }
        newExecutable = switch file.newExecutable {
            case true?: .enabled
            case false?: .disabled
            case nil: .unavailable
        }
        selectedExecutable = newExecutable
        wholeFileSide = file.supportsEditing ? nil : .new
        stats = file.stats
    }

    var selection: ExternalDiffSelection? {
        ExternalDiffSelection(
            file: DiffEditFileSelection(
                path: hunk.path,
                oldPath: nil,
                oldContent: hunk.old.content,
                newContent: hunk.new.content,
                hunkType: hunk.hunkType,
                lineRanges: diffEditRanges(lines: selectedLines.sorted().map(UInt32.init))
            ),
            selectedExists: selectedExists,
            selectedExecutable: selectedExecutable == .unavailable ? nil : selectedExecutable == .enabled,
            wholeFileSide: wholeFileSide
        )
    }

    func selectSide(_ side: ExternalDiffSide) {
        if wholeFileSide != nil {
            wholeFileSide = side
        }
        switch side {
            case .old:
                selectedLines = []
                selectedExists = oldExists
                selectedExecutable = oldExecutable
            case .new:
                selectedLines = changedLines
                selectedExists = newExists
                selectedExecutable = newExecutable
        }
    }

    var executableChanged: Bool {
        oldExecutable != .unavailable && newExecutable != .unavailable && oldExecutable != newExecutable
    }

    var keepsAllChanges: Bool {
        if let wholeFileSide {
            return wholeFileSide == .new
        }
        return selectedLines == changedLines
            && selectedExists == newExists
            && selectedExecutable == newExecutable
    }

    var keepsAnyChanges: Bool {
        if let wholeFileSide {
            return wholeFileSide == .new
        }
        return !selectedLines.isEmpty
            || (oldExists != newExists && selectedExists == newExists)
            || (executableChanged && selectedExecutable == newExecutable)
    }

    func selectDisplayRange(_ range: ClosedRange<Int>) {
        selectedLines.formUnion(range.compactMap { displayToFull[$0] })
        syncExistsWithSelection()
    }

    func toggleDisplayLine(_ line: Int) {
        guard let fullLine = displayToFull[line] else { return }
        if selectedLines.contains(fullLine) {
            selectedLines.remove(fullLine)
        } else {
            selectedLines.insert(fullLine)
        }
        syncExistsWithSelection()
    }

    private func syncExistsWithSelection() {
        guard oldExists != newExists else { return }
        if selectedLines.isEmpty {
            selectedExists = oldExists
        } else if selectedLines == changedLines {
            selectedExists = newExists
        } else {
            selectedExists = true
        }
        if !selectedExists {
            selectedExecutable = .unavailable
        } else if newExists {
            selectedExecutable = newExecutable
        } else {
            selectedExecutable = oldExecutable
        }
    }
}
