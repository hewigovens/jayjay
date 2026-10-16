import Foundation

struct FileHistoryRequest: Codable, Hashable {
    let repoPath: String
    let path: String

    /// SwiftUI window values travel through the window manager as strings.
    var windowValue: String {
        (try? JSONEncoder().encode(self)).flatMap { String(data: $0, encoding: .utf8) } ?? ""
    }

    init(repoPath: String, path: String) {
        self.repoPath = repoPath
        self.path = path
    }

    init?(windowValue: String) {
        guard let request = try? JSONDecoder().decode(Self.self, from: Data(windowValue.utf8)) else { return nil }
        self = request
    }
}
