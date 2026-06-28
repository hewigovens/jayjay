import Foundation
import JayJayCore

protocol ChangeActions: AnyObject {
    func select(changeId: String?)
    func describeChange(rev: String, message: String)
    func restoreFiles(rev: String, paths: [String])
    func deleteFiles(paths: [String])
    func ignoreAndUntrack(paths: [String])
    func split(rev: String, paths: [String], message: String, parallel: Bool)
    func moveToWorkingCopy(rev: String, paths: [String])
    func applyDiffSelection(
        rev: String,
        destination: DiffEditDestination,
        selections: [DiffEditFileSelection],
        message: String,
        ignoreWhitespace: Bool
    )
    func resolveUseOurs(rev: String, path: String)
    func resolveUseTheirs(rev: String, path: String)
    func resolveInEditor(rev: String, path: String, tool: String)
}
