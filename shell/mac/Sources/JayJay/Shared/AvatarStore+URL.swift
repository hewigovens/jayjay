import Foundation
import JayJayCore

/// Avatar URLs that only a forge API can resolve; `avatarSource` maps an email to its source.
extension AvatarStore {
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
}
