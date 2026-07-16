import Foundation
import JayJayCore

@MainActor
@Observable
final class RepositoryStore {
    private let storePath: String?
    private var refreshGeneration = 0

    var paths: [String] {
        _ = refreshGeneration
        return repositories(storePath: storePath)
    }

    init(storePath: String? = nil) {
        self.storePath = storePath
    }

    func reload() {
        refreshGeneration &+= 1
    }

    func setPinned(_ pinned: Bool, path: String) {
        _ = setRepositoryPinned(path: path, pinned: pinned, storePath: storePath)
        refreshGeneration &+= 1
    }
}
