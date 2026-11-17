import AppKit

@objc(ShowFileScriptCommand)
final class ShowFileScriptCommand: RepositoryScriptCommand {
    override func performDefaultImplementation() -> Any? {
        MainActor.assumeIsolated {
            perform({ try await $0.showFile($1) }, expecting: "Expected a file path relative to the workspace root.")
        }
        return nil
    }
}
