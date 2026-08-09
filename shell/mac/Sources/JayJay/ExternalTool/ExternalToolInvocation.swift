import Darwin
import Foundation
import JayJayCore

extension ExternalToolInvocation {
    static func parse(arguments: [String]) -> Self? {
        do {
            let toolArguments = sanitizedToolArguments(arguments)
            guard let first = toolArguments.first,
                  first == "tool" || (toolArguments.count == 2 && !first.hasPrefix("-"))
            else { return nil }
            return try parseExternalToolInvocation(arguments: toolArguments)
        } catch {
            FileHandle.standardError.write(Data("error: \(error.localizedDescription)\n".utf8))
            Darwin.exit(1)
        }
    }

    private static func sanitizedToolArguments(_ arguments: [String]) -> [String] {
        var toolArguments = Array(arguments.dropFirst())
        if let index = toolArguments.firstIndex(of: "-ApplePersistenceIgnoreState") {
            toolArguments.remove(at: index)
            if index < toolArguments.endIndex {
                toolArguments.remove(at: index)
            }
        }
        return toolArguments
    }

    var windowTitle: String {
        switch self {
            case let .diff(_, _, editable): editable ? "Edit Diff — JayJay" : "Compare — JayJay"
            case let .merge(_, _, _, _, path, _): "Resolve \(path) — JayJay"
        }
    }
}
