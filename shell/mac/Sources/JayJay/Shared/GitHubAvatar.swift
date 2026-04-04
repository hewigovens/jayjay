import CryptoKit
import SwiftUI

/// Displays an avatar for a commit author.
/// Tries GitHub noreply username first, then falls back to Gravatar (works with any email).
struct CommitAvatar: View {
    let email: String
    var size: CGFloat = 20

    private var avatarURL: URL? {
        guard !email.isEmpty else { return nil }

        // GitHub noreply: extract username → github.com/{username}.png
        if email.hasSuffix("@users.noreply.github.com") {
            let local = email.components(separatedBy: "@").first ?? ""
            let username = local.contains("+")
                ? local.components(separatedBy: "+").last ?? local : local
            if !username.isEmpty {
                return URL(string: "https://github.com/\(username).png?size=\(Int(size * 2))")
            }
        }

        // Gravatar: md5(lowercase email) — universal fallback
        let hash = Insecure.MD5
            .hash(data: Data(email.lowercased().trimmingCharacters(in: .whitespaces).utf8))
            .map { String(format: "%02x", $0) }
            .joined()
        return URL(string: "https://gravatar.com/avatar/\(hash)?s=\(Int(size * 2))&d=retro")
    }

    var body: some View {
        AsyncImage(url: avatarURL) { phase in
            switch phase {
                case let .success(image):
                    image.resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: size, height: size)
                        .clipShape(Circle())
                default:
                    fallbackIcon
            }
        }
        .frame(width: size, height: size)
    }

    private var fallbackIcon: some View {
        Image(systemName: "person.circle.fill")
            .resizable()
            .frame(width: size, height: size)
            .foregroundStyle(.secondary)
    }
}
