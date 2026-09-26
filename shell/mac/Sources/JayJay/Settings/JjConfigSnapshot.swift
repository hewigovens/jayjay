import Foundation
import JayJayCore

enum JjConfigSnapshot: Sendable {
    case missing
    case failed(path: String, message: String)
    case found(path: String, sections: [JjConfigSection])

    /// Core resolves the same user-level layers jj does and reports where the user config would live even before it exists, so an absent config is an empty state rather than a listing of environment-derived values. A config that fails to load keeps its path, so it can still be opened for repair.
    static func load() -> Self {
        do {
            let config = try jjUserConfig()
            if let error = config.error {
                return .failed(path: config.path, message: error)
            }
            guard config.exists else {
                return .missing
            }
            return .found(path: config.path, sections: config.sections)
        } catch {
            return .failed(path: "", message: error.friendlyDescription)
        }
    }
}
