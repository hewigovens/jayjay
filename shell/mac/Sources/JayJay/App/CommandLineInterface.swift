import Darwin
import Foundation
import JayJayCore

enum CommandLineInterface {
    static func runAndExitIfNeeded(arguments: [String]) {
        guard let outcome = outcome(for: arguments) else { return }
        let output = outcome.exitCode != 0 ? FileHandle.standardError : FileHandle.standardOutput
        output.write(Data(outcome.message.utf8))
        Darwin.exit(outcome.exitCode)
    }

    static func outcome(for arguments: [String]) -> CliCommandOutcome? {
        let cliArguments = appArguments(arguments)
        guard !cliArguments.isEmpty else { return nil }
        return runAppCliCommand(arguments: cliArguments, version: AppMetadata.shortVersion)
    }

    /// The `jayjay` launcher prefixes tool sessions with `-ApplePersistenceIgnoreState YES`.
    static func appArguments(_ arguments: [String]) -> [String] {
        var appArguments = Array(arguments.dropFirst())
        if let index = appArguments.firstIndex(of: "-ApplePersistenceIgnoreState") {
            appArguments.remove(at: index)
            if index < appArguments.endIndex {
                appArguments.remove(at: index)
            }
        }
        return appArguments
    }
}
