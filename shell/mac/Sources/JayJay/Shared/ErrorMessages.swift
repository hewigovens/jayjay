import Foundation
import JayJayBindings

extension Error {
    var friendlyDescription: String {
        if let jjError = self as? JayJayError {
            switch jjError {
            case let .RepoNotFound(path):
                return "No Jujutsu repository found at \(URL(fileURLWithPath: path).lastPathComponent)"
            case let .RevNotFound(rev):
                return "Revision not found: \(rev)"
            case let .Internal(message):
                return message
            }
        }
        return localizedDescription
    }
}
