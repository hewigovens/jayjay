import Foundation

/// Resolve an author email to an avatar image URL, per forge.
extension AvatarStore {
    /// The local part of an email, with a GitHub noreply `<id>+<user>` prefix stripped.
    static func username(from email: String) -> String {
        let local = email.components(separatedBy: "@").first ?? ""
        return local.contains("+") ? (local.components(separatedBy: "+").last ?? local) : local
    }

    /// Numeric GitHub user id for a bot noreply address `<id>+<name>[bot]@...`, else nil.
    static func botUserID(_ email: String) -> String? {
        guard email.hasSuffix("@users.noreply.github.com") else { return nil }
        let local = email.components(separatedBy: "@").first ?? ""
        guard let plus = local.firstIndex(of: "+") else { return nil }
        let id = String(local[local.startIndex ..< plus])
        let user = String(local[local.index(after: plus)...])
        guard !id.isEmpty, id.allSatisfy(\.isNumber), user.hasSuffix("[bot]") else { return nil }
        return id
    }

    private struct BotUser: Decodable {
        let avatarURL: String
        enum CodingKeys: String, CodingKey { case avatarURL = "avatar_url" }
    }

    /// A bot's avatar is at `in/<app-id>`, reachable only via the API (`u/<id>` is just an identicon).
    static func botAvatarURL(id: String, pixelSize: Int) async -> URL? {
        guard let api = URL(string: "https://api.github.com/user/\(id)") else { return nil }
        var request = URLRequest(url: api)
        request.setValue("jayjay", forHTTPHeaderField: "User-Agent")
        guard let (data, response) = try? await URLSession.shared.data(for: request),
              (response as? HTTPURLResponse)?.statusCode == 200,
              let decoded = try? JSONDecoder().decode(BotUser.self, from: data),
              var components = URLComponents(string: decoded.avatarURL)
        else { return nil }
        components.queryItems = (components.queryItems ?? []) + [URLQueryItem(name: "size", value: String(pixelSize))]
        return components.url
    }

    /// gitlab.com username when `email` is a GitLab noreply commit address, else nil.
    static func gitlabUsername(_ email: String) -> String? {
        guard email.hasSuffix("@users.noreply.gitlab.com") else { return nil }
        let local = email.components(separatedBy: "@").first ?? ""
        // `<numeric-id>-<username>` (privacy on) or legacy `<username>`.
        var username = local
        if let dash = local.firstIndex(of: "-") {
            let id = String(local[local.startIndex ..< dash])
            if !id.isEmpty, id.allSatisfy(\.isNumber) {
                username = String(local[local.index(after: dash)...])
            }
        }
        // Validate the charset so a repo-controlled email can't inject query params.
        let valid = !username.isEmpty && username.allSatisfy { $0.isLetter || $0.isNumber || "._-".contains($0) }
        return valid ? username : nil
    }

    private struct GitLabUser: Decodable {
        let avatarURL: String?
        enum CodingKeys: String, CodingKey { case avatarURL = "avatar_url" }
    }

    /// gitlab.com serves a user's avatar via `users?username=`; a noreply email only carries the username.
    static func gitlabAvatarURL(username: String, pixelSize: Int) async -> URL? {
        guard let api = URL(string: "https://gitlab.com/api/v4/users?username=\(username)") else { return nil }
        var request = URLRequest(url: api)
        request.setValue("jayjay", forHTTPHeaderField: "User-Agent")
        guard let (data, response) = try? await URLSession.shared.data(for: request),
              (response as? HTTPURLResponse)?.statusCode == 200,
              let users = try? JSONDecoder().decode([GitLabUser].self, from: data),
              let first = users.first,
              let avatar = first.avatarURL,
              var components = URLComponents(string: avatar)
        else { return nil }
        components.queryItems = (components.queryItems ?? []) + [URLQueryItem(name: "width", value: String(pixelSize))]
        return components.url
    }

    /// GitHub avatar by id (new noreply), else profile png (old form), else Gravatar. Bots use `botAvatarURL`.
    static func remoteURL(email: String, pixelSize: Int) -> URL? {
        if email.hasSuffix("@users.noreply.github.com") {
            let local = email.components(separatedBy: "@").first ?? ""
            if let plus = local.firstIndex(of: "+") {
                let id = String(local[local.startIndex ..< plus])
                if !id.isEmpty, id.allSatisfy(\.isNumber) {
                    return URL(string: "https://avatars.githubusercontent.com/u/\(id)?size=\(pixelSize)")
                }
            }
            let username = username(from: email)
            if !username.isEmpty {
                return URL(string: "https://github.com/\(username).png?size=\(pixelSize)")
            }
        }
        return URL(string: "https://gravatar.com/avatar/\(key(email))?s=\(pixelSize)&d=retro")
    }
}
