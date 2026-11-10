import Foundation
import JayJayCore

/// Checks for jj installation via Rust core.
enum JJEnvironment {
    struct Status: Sendable {
        let isInstalled: Bool
        let version: String?
        let path: String?
    }

    /// Off the main actor: the check spawns `jj version` and may first resolve the login shell's PATH.
    static func check() async -> Status {
        await Task.detached(priority: .userInitiated) {
            let result = checkJjEnvironment()
            return Status(
                isInstalled: result.isInstalled,
                version: result.version.isEmpty ? nil : result.version,
                path: result.path.isEmpty ? nil : result.path
            )
        }.value
    }
}
