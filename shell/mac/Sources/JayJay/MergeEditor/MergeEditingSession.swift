import Foundation
import JayJayCore
import Observation

@MainActor
protocol MergeEditingSession: AnyObject, Observable {
    var path: String { get }
    var left: String { get }
    var base: String { get }
    var right: String { get }
    var result: String { get set }
    var resultMode: MergeResultMode { get set }
    var isText: Bool { get }
    var canSave: Bool { get }
    var canUseSources: Bool { get }
    var showsSources: Bool { get }
    var isLoading: Bool { get }
    var isSaving: Bool { get }
    var selectedSource: MergeHunkSource? { get set }
    var picks: [UInt32: MergeHunkSource] { get set }
    var loadError: String? { get }
    var actionError: String? { get set }
    var unresolvedCount: Int { get }
    var highlights: MergeEditorHighlights? { get }

    func useSource(_ source: MergeHunkSource)
    func sideLabel(_ source: MergeHunkSource) -> String?
}

struct MergeEditState: Equatable {
    var result: String
    var picks: [UInt32: MergeHunkSource]
    var selectedSource: MergeHunkSource?
}

extension MergeEditingSession {
    var hasEdits: Bool {
        highlights.map { result != $0.resultText } ?? false
    }

    var editState: MergeEditState {
        get { MergeEditState(result: result, picks: picks, selectedSource: selectedSource) }
        set {
            result = newValue.result
            picks = newValue.picks
            selectedSource = newValue.selectedSource
        }
    }

    func resultChanged() {
        selectedSource = nil
    }

    func recordingUndo(_ undoManager: UndoManager?, _ change: () -> Void) {
        let before = editState
        change()
        if editState != before {
            registerUndo(restoring: before, undoManager)
        }
    }

    private func registerUndo(restoring state: MergeEditState, _ undoManager: UndoManager?) {
        undoManager?.registerUndo(withTarget: self) { session in
            MainActor.assumeIsolated {
                let current = session.editState
                session.editState = state
                session.registerUndo(restoring: current, undoManager)
            }
        }
    }

    func useWholeSource(_ source: MergeHunkSource, undoManager: UndoManager?) {
        recordingUndo(undoManager) {
            useSource(source)
            picks = [:]
        }
    }

    func useHunkSource(_ hunk: MergeEditorHunk, _ source: MergeHunkSource, undoManager: UndoManager?) -> Bool {
        guard let highlights else { return false }
        do {
            let updated = try mergeResultPickHunk(
                original: highlights.resultText,
                hunks: highlights.hunks.map(\.hunk),
                picks: picks.map { MergeHunkPick(index: $0.key, source: $0.value) },
                result: result,
                pick: MergeHunkPick(index: hunk.index, source: source)
            )
            recordingUndo(undoManager) {
                result = updated
                picks[hunk.index] = source
                selectedSource = nil
            }
            actionError = nil
            return true
        } catch {
            actionError = error.friendlyDescription
            return false
        }
    }

    func sideHelp(_ source: MergeHunkSource) -> String {
        guard let label = sideLabel(source) else { return source.useHelp }
        return "\(source.useHelp)\n\(label)"
    }
}
