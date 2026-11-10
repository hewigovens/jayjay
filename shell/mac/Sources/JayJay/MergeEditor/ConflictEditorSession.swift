import Foundation
import JayJayCore
import Observation

@MainActor
@Observable
final class ConflictEditorSession: Identifiable, MergeEditingSession {
    let id = UUID()
    let target: ConflictEditorTarget

    var data: ConflictEditorData? {
        didSet { updateUnresolvedCount() }
    }

    var result = "" {
        didSet { updateUnresolvedCount() }
    }

    var resultMode = MergeResultMode.hunks
    var selectedSource: MergeHunkSource? {
        didSet { updateUnresolvedCount() }
    }

    /// Stored because the editor reads it several times per render and a marker scan walks the whole result.
    private(set) var unresolvedCount = 0
    var picks: [UInt32: MergeHunkSource] = [:]
    var highlights: MergeEditorHighlights?
    var isLoading = true
    var isSaving = false
    private(set) var loadError: String?
    var actionError: String?

    init(target: ConflictEditorTarget) {
        self.target = target
    }

    var path: String {
        target.path
    }

    private func updateUnresolvedCount() {
        guard let data, selectedSource == nil else {
            unresolvedCount = 0
            return
        }
        unresolvedCount = Int(externalConflictMarkerCount(content: result, markerLength: data.markerLength))
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
            loadError = error.friendlyDescription
        }
        isLoading = false
    }

    func sideLabel(_ source: MergeHunkSource) -> String? {
        switch source {
            case .left: data?.leftLabel
            case .base: data?.baseLabel
            case .right: data?.rightLabel
        }
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
