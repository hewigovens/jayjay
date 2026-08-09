import Foundation
import JayJayCore
import Observation

@MainActor
@Observable
final class ConflictEditorSession: Identifiable, MergeEditingSession {
    let id = UUID()
    let target: ConflictEditorTarget

    var data: ConflictEditorData?
    var result = ""
    var resultMode = MergeResultMode.hunks
    var selectedSource: MergeHunkSource?
    var highlights: MergeEditorHighlights?
    var isLoading = true
    var isSaving = false
    var errorMessage: String?

    init(target: ConflictEditorTarget) {
        self.target = target
    }

    var path: String {
        target.path
    }

    var unresolvedCount: Int {
        guard let data, selectedSource == nil else { return 0 }
        return Int(externalConflictMarkerCount(content: result, markerLength: data.markerLength))
    }

    var left: String {
        data?.left ?? ""
    }

    var base: String {
        data?.base ?? ""
    }

    var right: String {
        data?.right ?? ""
    }

    var isText: Bool {
        data?.isText == true
    }

    var canSave: Bool {
        isText
    }

    var canUseSources: Bool {
        isText
    }

    var showsSources: Bool {
        data?.sideCount == 2
    }

    func load() async {
        guard isLoading else { return }
        do {
            let target = target
            let prepared = try await Task.detached {
                let data = try target.load()
                let highlights = MergeEditorHighlights(
                    path: data.path,
                    left: data.left,
                    base: data.base,
                    right: data.right,
                    result: data.result,
                    hunks: data.hunks
                )
                return (data, highlights)
            }.value
            data = prepared.0
            result = prepared.0.result
            highlights = prepared.1
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func useSource(_ source: MergeHunkSource) {
        guard let data, data.isText else { return }
        switch source {
            case .left: result = data.left
            case .base: result = data.base
            case .right: result = data.right
        }
        selectedSource = source
    }
}
