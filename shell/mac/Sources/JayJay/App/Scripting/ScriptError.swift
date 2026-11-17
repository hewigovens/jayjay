import Foundation

struct ScriptError: Error {
    let number: Int
    let message: String

    static let repositoryClosed = ScriptError(number: -1728, message: "The repository is no longer open.")
    static let severalRepositories = ScriptError(number: -1700, message: "Address one repository at a time.")

    func report(to command: NSScriptCommand?) {
        command?.scriptErrorNumber = number
        command?.scriptErrorString = message
    }
}
