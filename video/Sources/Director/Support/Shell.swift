import Foundation

enum Shell {
    static func run(_ executable: String, _ arguments: [String]) throws {
        let process = Process()
        let pipe = Pipe()
        process.executableURL = URL(fileURLWithPath: executable)
        process.arguments = arguments
        process.standardOutput = pipe
        process.standardError = pipe
        try process.run()
        let output = String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
        process.waitUntilExit()
        guard process.terminationStatus == 0 else {
            let tail = output.split(separator: "\n").suffix(15).joined(separator: "\n")
            throw Failure("\(([executable] + arguments).joined(separator: " ")) exited with \(process.terminationStatus)\n\(tail)")
        }
    }
}
