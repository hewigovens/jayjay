import Darwin
import Foundation

enum CommandLineInterface {
    static func runAndExitIfNeeded(arguments: [String]) {
        guard let status = runIfNeeded(arguments: Array(arguments.dropFirst())) else { return }
        Darwin.exit(Int32(status))
    }

    private static func runIfNeeded(arguments: [String]) -> Int? {
        guard let first = arguments.first else { return nil }
        if first == "--version" || first == "-v" {
            writeOutput("jayjay \(AppMetadata.shortVersion)\n")
            return 0
        }
        guard first == "review" else { return nil }

        do {
            let output = try ReviewCommand(arguments: Array(arguments.dropFirst())).run()
            writeOutput(output)
            return 0
        } catch {
            writeError("error: \(errorMessage(error))\n")
            return 1
        }
    }

    private static func writeOutput(_ text: String) {
        FileHandle.standardOutput.write(Data(text.utf8))
    }

    private static func writeError(_ text: String) {
        FileHandle.standardError.write(Data(text.utf8))
    }

    private static func errorMessage(_ error: Error) -> String {
        let description = error.localizedDescription
        if let message = associatedString("message", in: description) {
            return message
        }
        if let path = associatedString("path", in: description) {
            return "repository not found: \(path)"
        }
        if let rev = associatedString("rev", in: description) {
            return "revision not found: \(rev)"
        }
        return description
    }

    private static func associatedString(_ label: String, in description: String) -> String? {
        let marker = "\(label): "
        guard let markerRange = description.range(of: marker) else { return nil }
        var end = markerRange.upperBound
        guard end < description.endIndex, description[end] == "\"" else { return nil }
        var escaped = false
        repeat {
            end = description.index(after: end)
            guard end < description.endIndex else { return nil }
            if escaped {
                escaped = false
            } else if description[end] == "\\" {
                escaped = true
            } else if description[end] == "\"" {
                break
            }
        } while true

        let literal = String(description[markerRange.upperBound ... end])
        return try? JSONDecoder().decode(String.self, from: Data(literal.utf8))
    }
}
