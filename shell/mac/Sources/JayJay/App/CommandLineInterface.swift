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
        let cliArguments = Array(arguments.dropFirst())
        guard !cliArguments.isEmpty else { return nil }
        return runAppCliCommand(arguments: cliArguments, version: AppMetadata.shortVersion)
    }
}
