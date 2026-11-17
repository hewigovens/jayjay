import Foundation

extension NSScriptCommand {
    /// A suspended command can be resumed once, so a command addressed to a list of objects cannot be completed per object.
    var targetsSeveralObjects: Bool {
        ((evaluatedReceivers as? [Any])?.count ?? 1) > 1
    }

    func complete(_ work: @escaping @MainActor () async throws -> Any?) {
        suspendExecution()
        Task { @MainActor in
            do {
                try await resumeExecution(withResult: work())
            } catch let error as ScriptError {
                error.report(to: self)
                resumeExecution(withResult: nil)
            } catch {
                ScriptError(number: -10000, message: error.localizedDescription).report(to: self)
                resumeExecution(withResult: nil)
            }
        }
    }
}
