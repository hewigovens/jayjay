import AppKit

@objc(SelectChangeScriptCommand)
final class SelectChangeScriptCommand: RepositoryScriptCommand {
    override func performDefaultImplementation() -> Any? {
        MainActor.assumeIsolated {
            perform({ try await $0.selectChange($1) }, expecting: "Expected a jj revision expression.")
        }
        return nil
    }
}
