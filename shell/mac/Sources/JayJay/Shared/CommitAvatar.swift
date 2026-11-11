import AppKit
import JayJayCore
import SwiftUI

/// A circular author avatar: the cached image when available, else a stable
/// colored monogram. Reads the in-memory cache synchronously so a known avatar
/// never flashes a placeholder while scrolling.
struct CommitAvatar: View {
    let email: String
    var size: CGFloat = 20
    private let cacheKey: String
    private let monogramColor: Color
    private let monogramInitial: String
    @State private var image: NSImage?

    init(author: CommitAuthor, size: CGFloat = 20) {
        email = author.email
        self.size = size
        cacheKey = avatarCacheKey(email: email)
        monogramColor = Color(rgb: avatarMonogramColor(email: email))
        monogramInitial = avatarMonogramInitial(author: author)
        _image = State(initialValue: Self.cachedImage(email: email, key: cacheKey))
    }

    var body: some View {
        content
            .frame(width: size, height: size)
            // Re-seed when a recycled row swaps to a different author.
            .onChange(of: email) { image = Self.cachedImage(email: email, key: cacheKey) }
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
                .scaledToFill()
                .frame(width: size, height: size)
                .clipShape(Circle())
        } else {
            monogram
        }
    }

    private var monogram: some View {
        Circle()
            .fill(monogramColor)
            .overlay(
                Text(monogramInitial)
                    .font(.system(size: size * 0.5, weight: .semibold))
                    .foregroundStyle(.white)
            )
            .frame(width: size, height: size)
    }

    private static func cachedImage(email: String, key: String) -> NSImage? {
        email.isEmpty ? nil : AvatarStore.cachedImage(key: key)
    }
}
