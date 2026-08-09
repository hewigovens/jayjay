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
    var errorMessage: String? { get set }
    var unresolvedCount: Int { get }
    var highlights: MergeEditorHighlights? { get }

    func useSource(_ source: MergeHunkSource)
}

extension MergeEditingSession {
    func resultChanged() {
        selectedSource = nil
    }

    func useHunkSource(_ hunk: MergeEditorHunk, _ source: MergeHunkSource) {
        do {
            result = try mergeResultUseSource(result: result, hunk: hunk, source: source)
            selectedSource = nil
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
