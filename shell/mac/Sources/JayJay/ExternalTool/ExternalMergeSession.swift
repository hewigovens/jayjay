import AppKit
import Darwin
import Foundation
import JayJayCore
import Observation

@MainActor
@Observable
final class ExternalMergeSession: MergeEditingSession {
    let leftPath: String
    let basePath: String
    let rightPath: String
    let outputPath: String
    let path: String
    let markerLength: UInt32

    var left = ""
    var base = ""
    var right = ""
    var result = ""
    var resultMode = MergeResultMode.hunks
    var selectedSource: MergeHunkSource?
    var highlights: MergeEditorHighlights?
    private(set) var isTextMerge = false
    var isLoading = true
    var isSaving = false
    var errorMessage: String?
    private let onLoadFailure: () -> Void

    init(
        left: String,
        base: String,
        right: String,
        output: String,
        path: String,
        markerLength: UInt32,
        onLoadFailure: @escaping () -> Void = {}
    ) {
        leftPath = left
        basePath = base
        rightPath = right
        outputPath = output
        self.path = path
        self.markerLength = markerLength
        self.onLoadFailure = onLoadFailure
    }

    var unresolvedCount: Int {
        selectedSource == nil
            ? Int(externalConflictMarkerCount(content: result, markerLength: markerLength))
            : 0
    }

    var canSave: Bool {
        selectedSource != nil || isTextMerge
    }

    var isText: Bool {
        isTextMerge
    }

    var canUseSources: Bool {
        true
    }

    var showsSources: Bool {
        true
    }

    func load() async {
        guard isLoading else { return }
        let paths = (leftPath, basePath, rightPath, outputPath)
        let path = path
        let markerLength = markerLength
        do {
            let prepared = try await Task.detached {
                let merge = try loadExternalMerge(
                    left: paths.0,
                    base: paths.1,
                    right: paths.2,
                    output: paths.3,
                    markerLength: markerLength
                )
                let highlights = MergeEditorHighlights(
                    path: path,
                    left: merge.left,
                    base: merge.base,
                    right: merge.right,
                    result: merge.result,
                    hunks: merge.hunks
                )
                return (merge, highlights)
            }.value
            let merge = prepared.0
            (left, base, right) = (merge.left, merge.base, merge.right)
            result = merge.result
            isTextMerge = merge.isText
            highlights = prepared.1
        } catch {
            errorMessage = error.localizedDescription
            onLoadFailure()
        }
        isLoading = false
    }

    func useSource(_ source: MergeHunkSource) {
        let content: String = switch source {
            case .left: left
            case .base: base
            case .right: right
        }
        if isTextMerge {
            result = content
        }
        selectedSource = source
    }

    func save() {
        guard !isSaving, canSave else { return }
        isSaving = true
        errorMessage = nil
        let output = outputPath
        let content = result
        let source = selectedSourcePath
        Task {
            do {
                try await Task.detached {
                    if let source {
                        try useExternalMergeSide(source: source, output: output)
                    } else {
                        try writeExternalMerge(output: output, content: content)
                    }
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

    private var selectedSourcePath: String? {
        switch selectedSource {
            case .left: leftPath
            case .base: basePath
            case .right: rightPath
            case nil: nil
        }
    }
}
