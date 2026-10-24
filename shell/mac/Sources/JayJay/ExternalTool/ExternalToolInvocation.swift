import Darwin
import Foundation
import JayJayCore

extension ExternalToolInvocation {
    static func parse(arguments: [String]) -> Self? {
        do {
            let toolArguments = CommandLineInterface.appArguments(arguments)
            guard let first = toolArguments.first,
                  first == "tool" || (toolArguments.count == 2 && !first.hasPrefix("-"))
            else { return nil }
            return try parseExternalToolInvocation(arguments: toolArguments)
        } catch {
            FileHandle.standardError.write(Data("error: \(error.friendlyDescription)\n".utf8))
            Darwin.exit(1)
        }
    }

    var windowTitle: String {
        switch self {
            case let .diff(_, _, editable): editable ? Localized.Merge.editDiffJayjay : Localized.Merge.compareJayjay
            case let .merge(_, _, _, _, path, _): Localized.Merge.resolveWindowTitle(path: path)
        }
    }
}
