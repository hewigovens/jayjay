import AppKit

class RepositoryScriptCommand: NSScriptCommand {
    @MainActor
    func targetRepository() throws -> ScriptRepository {
        if let argument = evaluatedArguments?["repository"] {
            guard let repository = argument as? ScriptRepository else { throw ScriptError.repositoryClosed }
            return repository
        }
        guard let front = ScriptingHost.windowManager?.frontScriptingRepository else {
            throw ScriptError(number: -1728, message: "No repository is open.")
        }
        return front
    }

    @MainActor
    func perform(_ work: @escaping @MainActor (ScriptRepository, String) async throws -> Void, expecting message: String) {
        do {
            guard let text = directParameter as? String,
                  !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            else { throw ScriptError(number: -1700, message: message) }
            let repository = try targetRepository()
            complete {
                try await work(repository, text)
                return nil
            }
        } catch {
            (error as? ScriptError)?.report(to: self)
        }
    }
}
