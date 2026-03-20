import Foundation
import JayJayBindings

/// Checks for jj installation via Rust core.
enum JJEnvironment {
    struct Status {
        let isInstalled: Bool
        let version: String?
        let path: String?
    }

    static func check() -> Status {
        let result = checkJjEnvironment()
        return Status(
            isInstalled: result.isInstalled,
            version: result.version.isEmpty ? nil : result.version,
            path: result.path.isEmpty ? nil : result.path
        )
    }
}
