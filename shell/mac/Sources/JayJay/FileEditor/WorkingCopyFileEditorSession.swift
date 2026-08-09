import Foundation
import JayJayCore
import Observation

@MainActor
@Observable
final class WorkingCopyFileEditorSession: Identifiable {
    let id = UUID()
    let repo: JayJayRepo
    let path: String

    var data: FileEditorData?
    var content = ""
    var highlightedLines: [[DiffSpan]] = []
    var isLoading = true
    var isSaving = false
    var errorMessage: String?

    init(repo: JayJayRepo, path: String) {
        self.repo = repo
        self.path = path
    }

    var hasChanges: Bool {
        data.map { $0.content != content } ?? false
    }

    func load() async {
        guard isLoading else { return }
        do {
            let repo = repo
            let path = path
            let prepared = try await Task.detached {
                let data = try repo.workingCopyFileEditor(path: path)
                let highlightedLines = highlightFileLines(path: data.path, content: data.content)
                return (data, highlightedLines)
            }.value
            data = prepared.0
            content = prepared.0.content
            highlightedLines = prepared.1
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
