import Foundation

protocol ChangeActions: AnyObject {
    func describeChange(rev: String, message: String)
    func restoreFiles(rev: String, paths: [String])
    func deleteFiles(paths: [String])
    func ignoreAndUntrack(paths: [String])
    func split(rev: String, paths: [String], message: String)
}
