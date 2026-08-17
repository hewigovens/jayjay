import Foundation

/// A directory moved aside with a same-volume atomic rename before a destructive async operation, so the eventual delete can only touch the captured directory — never whatever occupies the original path by the time the operation completes.
struct QuarantinedDirectory: Sendable {
    struct Identity: Equatable, Sendable {
        let systemNumber: UInt64
        let fileNumber: UInt64
    }

    let originalURL: URL
    let quarantineURL: URL

    static func identity(path: String) throws -> Identity {
        let url = try validatedURL(path: path)
        let attributes = try FileManager.default.attributesOfItem(atPath: url.path)
        guard attributes[.type] as? FileAttributeType == .typeDirectory,
              let systemNumber = attributes[.systemNumber] as? NSNumber,
              let fileNumber = attributes[.systemFileNumber] as? NSNumber
        else {
            throw QuarantinedDirectoryError.notDirectory(path)
        }
        return Identity(
            systemNumber: systemNumber.uint64Value,
            fileNumber: fileNumber.uint64Value
        )
    }

    /// Atomically moves the directory at `path` into a fresh temporary directory on the same volume, then verifies that the moved object is the one identified before core validation.
    static func capture(path: String, expectedIdentity: Identity) throws -> QuarantinedDirectory {
        let original = try validatedURL(path: path)
        let container = try FileManager.default.url(
            for: .itemReplacementDirectory,
            in: .userDomainMask,
            appropriateFor: original,
            create: true
        )
        let quarantine = container.appendingPathComponent(original.lastPathComponent, isDirectory: true)
        try FileManager.default.moveItem(at: original, to: quarantine)
        let captured = QuarantinedDirectory(originalURL: original, quarantineURL: quarantine)
        do {
            guard try identity(path: quarantine.path) == expectedIdentity else {
                throw QuarantinedDirectoryError.identityChanged(path)
            }
        } catch {
            do {
                try captured.restore()
            } catch {
                throw QuarantinedDirectoryError.identityChangedAndPreserved(quarantine.path)
            }
            throw error
        }
        return captured
    }

    private static func validatedURL(path: String) throws -> URL {
        guard !path.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
              NSString(string: path).isAbsolutePath
        else {
            throw QuarantinedDirectoryError.unsafePath(path)
        }
        let original = URL(fileURLWithPath: path).standardizedFileURL
        guard original.path != "/" else {
            throw QuarantinedDirectoryError.unsafePath(path)
        }
        return original
    }

    /// Deletes the captured directory and its temporary container; the original path is left untouched.
    func delete() throws {
        try FileManager.default.removeItem(at: quarantineURL.deletingLastPathComponent())
    }

    /// Moves the captured directory back to its original path, for when the operation it was captured for fails.
    func restore() throws {
        try FileManager.default.moveItem(at: quarantineURL, to: originalURL)
        try? FileManager.default.removeItem(at: quarantineURL.deletingLastPathComponent())
    }
}

private enum QuarantinedDirectoryError: LocalizedError {
    case identityChanged(String)
    case identityChangedAndPreserved(String)
    case notDirectory(String)
    case unsafePath(String)

    var errorDescription: String? {
        switch self {
            case let .identityChanged(path):
                "The workspace directory at \(path) changed after confirmation, so it was not deleted."
            case let .identityChangedAndPreserved(path):
                "The workspace directory changed after confirmation and could not be restored. It is preserved at:\n\(path)"
            case let .notDirectory(path):
                "The workspace path is not a directory: \(path)"
            case let .unsafePath(path):
                "Refusing to delete a workspace with an unsafe path: \(path.isEmpty ? "(empty)" : path)"
        }
    }
}
