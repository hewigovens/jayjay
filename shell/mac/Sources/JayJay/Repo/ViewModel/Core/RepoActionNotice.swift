import Foundation

/// Success feedback for a history rewrite; `undoOperationId` is the operation it produced, when it produced one.
struct RepoActionNotice: Equatable {
    let id = UUID()
    let message: String
    let undoOperationId: String?
}
