import AppKit
import CryptoKit

/// Process-wide decoded-image cache (thread-safe) so scrolling never re-decodes or flashes.
private let avatarMemoryCache: NSCache<NSString, NSImage> = {
    let cache = NSCache<NSString, NSImage>()
    cache.countLimit = 512
    return cache
}()

/// Loads and caches author avatars under the app's native cache directory, shared with GPUI, with an in-memory cache and per-email fetch dedupe.
/// URL resolution lives in `AvatarStore+URL.swift`.
actor AvatarStore {
    static let shared = AvatarStore()
    private var inFlight: [String: Task<NSImage?, Never>] = [:]

    /// md5(trimmed lowercased email) — shared with GPUI and the Gravatar hash, so both shells share the disk file.
    static func key(_ email: String) -> String {
        let normalized = email.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return Insecure.MD5.hash(data: Data(normalized.utf8))
            .map { String(format: "%02x", $0) }
            .joined()
    }

    /// Synchronous in-memory hit, used to seed a view with no load flash.
    static func cachedImage(_ email: String) -> NSImage? {
        guard !email.isEmpty else { return nil }
        return avatarMemoryCache.object(forKey: key(email) as NSString)
    }

    func image(for email: String, pixelSize: Int) async -> NSImage? {
        guard !email.isEmpty else { return nil }
        let k = Self.key(email)
        if let cached = avatarMemoryCache.object(forKey: k as NSString) {
            return cached
        }
        if let existing = inFlight[k] {
            return await existing.value
        }

        let task = Task<NSImage?, Never> { await Self.fetch(email: email, key: k, pixelSize: pixelSize) }
        inFlight[k] = task
        let image = await task.value
        inFlight[k] = nil
        return image
    }

    private static func fetch(email: String, key: String, pixelSize: Int) async -> NSImage? {
        guard let fileURL = diskURL(key) else { return nil }
        if let data = try? Data(contentsOf: fileURL), let image = NSImage(data: data) {
            avatarMemoryCache.setObject(image, forKey: key as NSString)
            return image
        }
        let imageURL: URL? = if let botID = botUserID(email) {
            await botAvatarURL(id: botID, pixelSize: pixelSize)
        } else if let gitlabUser = gitlabUsername(email) {
            await gitlabAvatarURL(username: gitlabUser, pixelSize: pixelSize)
        } else {
            remoteURL(email: email, pixelSize: pixelSize)
        }
        guard let imageURL,
              let (data, response) = try? await URLSession.shared.data(from: imageURL),
              (response as? HTTPURLResponse)?.statusCode == 200,
              let image = NSImage(data: data)
        else { return nil }

        try? FileManager.default.createDirectory(
            at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true
        )
        try? data.write(to: fileURL)
        avatarMemoryCache.setObject(image, forKey: key as NSString)
        return image
    }

    static func diskURL(_ key: String, fileManager: FileManager = .default) -> URL? {
        fileManager.urls(for: .cachesDirectory, in: .userDomainMask).first?
            .appendingPathComponent("dev.hewig.jayjay", isDirectory: true)
            .appendingPathComponent("avatars", isDirectory: true)
            .appendingPathComponent("\(key).png")
    }
}
