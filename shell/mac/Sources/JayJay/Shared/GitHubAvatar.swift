import AppKit
import CryptoKit
import SwiftUI

/// Process-wide decoded-image cache so scrolling the DAG never re-decodes or
/// flashes. NSCache is thread-safe, so it can be read synchronously when a row
/// is created — a cached avatar then renders on the first frame.
private let avatarMemoryCache: NSCache<NSString, NSImage> = {
    let cache = NSCache<NSString, NSImage>()
    cache.countLimit = 512
    return cache
}()

/// Loads and caches author avatars. Mirrors the GPUI shell: an on-disk cache at
/// `~/.cache/jayjay/avatars/<md5>.png` (shared between shells) layered under an
/// in-memory cache, with per-email fetch dedupe so N rows by one author trigger
/// a single request and scrolling never hits the network.
actor AvatarStore {
    static let shared = AvatarStore()
    private var inFlight: [String: Task<NSImage?, Never>] = [:]

    /// md5(trimmed, lowercased email) — matches the GPUI shell's cache key and
    /// the Gravatar hash, so both shells share the same disk file.
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
        if let cached = avatarMemoryCache.object(forKey: k as NSString) { return cached }
        if let existing = inFlight[k] { return await existing.value }

        let task = Task<NSImage?, Never> { await Self.fetch(email: email, key: k, pixelSize: pixelSize) }
        inFlight[k] = task
        let image = await task.value
        inFlight[k] = nil
        return image
    }

    private static func fetch(email: String, key: String, pixelSize: Int) async -> NSImage? {
        let fileURL = diskURL(key)
        if let data = try? Data(contentsOf: fileURL), let image = NSImage(data: data) {
            avatarMemoryCache.setObject(image, forKey: key as NSString)
            return image
        }
        guard let url = remoteURL(email: email, pixelSize: pixelSize),
              let (data, response) = try? await URLSession.shared.data(from: url),
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

    private static func diskURL(_ key: String) -> URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".cache/jayjay/avatars", isDirectory: true)
            .appendingPathComponent("\(key).png")
    }

    /// The local part of an email, with a GitHub noreply `<id>+<user>` prefix stripped.
    static func username(from email: String) -> String {
        let local = email.components(separatedBy: "@").first ?? ""
        return local.contains("+") ? (local.components(separatedBy: "+").last ?? local) : local
    }

    /// GitHub noreply username first (preserves the account avatar), else Gravatar.
    private static func remoteURL(email: String, pixelSize: Int) -> URL? {
        if email.hasSuffix("@users.noreply.github.com") {
            let username = username(from: email)
            if !username.isEmpty {
                return URL(string: "https://github.com/\(username).png?size=\(pixelSize)")
            }
        }
        return URL(string: "https://gravatar.com/avatar/\(key(email))?s=\(pixelSize)&d=retro")
    }
}

/// A circular author avatar: the cached image when available, else a stable
/// colored monogram. Reads the in-memory cache synchronously so a known avatar
/// never flashes a placeholder while scrolling.
struct CommitAvatar: View {
    let email: String
    var size: CGFloat = 20
    @State private var image: NSImage?

    init(email: String, size: CGFloat = 20) {
        self.email = email
        self.size = size
        _image = State(initialValue: AvatarStore.cachedImage(email))
    }

    var body: some View {
        content
            .frame(width: size, height: size)
            // Re-seed when a recycled row swaps to a different author.
            .onChange(of: email) { image = AvatarStore.cachedImage(email) }
            .task(id: email) {
                let requested = email
                guard image == nil, !requested.isEmpty else { return }
                let loaded = await AvatarStore.shared.image(for: requested, pixelSize: Int(size * 2))
                // The row may have recycled to a different author mid-fetch.
                guard !Task.isCancelled, email == requested else { return }
                image = loaded
            }
    }

    @ViewBuilder private var content: some View {
        if let image {
            Image(nsImage: image)
                .resizable()
                .aspectRatio(contentMode: .fill)
                .frame(width: size, height: size)
                .clipShape(Circle())
        } else {
            monogram
        }
    }

    private var monogram: some View {
        Circle()
            .fill(Self.monogramColor(email))
            .overlay(
                Text(Self.initial(email))
                    .font(.system(size: size * 0.5, weight: .semibold))
                    .foregroundStyle(.white)
            )
            .frame(width: size, height: size)
    }

    /// Initial from the email's username (after `+` for GitHub noreply addresses).
    private static func initial(_ email: String) -> String {
        let username = AvatarStore.username(from: email)
        let source = username.isEmpty ? email : username
        guard let ch = source.first(where: { $0.isLetter || $0.isNumber }) else { return "?" }
        return String(ch).uppercased()
    }

    /// Deterministic monogram palette, shared with the GPUI shell's `initial_color`.
    static let monogramPalette: [UInt32] = [
        0x4A5568, 0x6B46C1, 0x2563EB, 0x059669, 0xD97706, 0xDC2626, 0xDB2777, 0x0891B2
    ]

    private static func monogramColor(_ email: String) -> Color {
        let byte = UInt8(String(AvatarStore.key(email).prefix(2)), radix: 16) ?? 0
        let hex = monogramPalette[Int(byte) % monogramPalette.count]
        return Color(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }
}
