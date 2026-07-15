import CryptoKit
import Foundation

/// Sends anonymous activity without transmitting a stable installation identifier.
enum AppTelemetry {
    private static let endpoint = URL(string: "https://jayjay.hewigovens.workers.dev/ping")!
    private static let installSecretKey = "jayjay.telemetryInstallSecret"
    private static let lastSentDayKey = "jayjay.telemetryLastSentDay"

    static func maybePing(
        enabled: Bool,
        defaults: UserDefaults = .standard,
        now: Date = Date(),
        session: URLSession = .shared
    ) {
        guard enabled, releaseTelemetryEnabled else { return }
        let periods = periods(at: now)
        guard defaults.string(forKey: lastSentDayKey) != periods.day else { return }

        let secret = installSecret(defaults: defaults)
        var components = URLComponents(url: endpoint, resolvingAgainstBaseURL: false)
        components?.queryItems = [
            URLQueryItem(name: "platform", value: "macos"),
            URLQueryItem(name: "app", value: "jayjay"),
            URLQueryItem(name: "version", value: AppMetadata.shortVersion),
            URLQueryItem(name: "build", value: AppMetadata.buildNumber),
            URLQueryItem(name: "os", value: "macos"),
            URLQueryItem(name: "osver", value: osVersion),
            URLQueryItem(name: "arch", value: architecture),
            URLQueryItem(name: "daily_id", value: periodID(secret: secret, scope: "day", period: periods.day)),
            URLQueryItem(name: "monthly_id", value: periodID(secret: secret, scope: "month", period: periods.month))
        ]
        guard let url = components?.url else { return }

        session.dataTask(with: url) { _, response, _ in
            guard let response = response as? HTTPURLResponse, 200 ..< 300 ~= response.statusCode else { return }
            defaults.set(periods.day, forKey: lastSentDayKey)
        }.resume()
    }

    static func periods(at date: Date) -> (day: String, month: String) {
        (period(date, format: "yyyy-MM-dd"), period(date, format: "yyyy-MM"))
    }

    static func periodID(secret: String, scope: String, period: String) -> String {
        let digest = SHA256.hash(data: Data("\(secret)\0\(scope)\0\(period)".utf8))
        return digest.map { String(format: "%02x", $0) }.joined()
    }

    private static var releaseTelemetryEnabled: Bool {
        #if DEBUG
            false
        #else
            true
        #endif
    }

    private static func installSecret(defaults: UserDefaults) -> String {
        if let secret = defaults.string(forKey: installSecretKey), !secret.isEmpty {
            return secret
        }
        let secret = UUID().uuidString.lowercased()
        defaults.set(secret, forKey: installSecretKey)
        return secret
    }

    private static func period(_ date: Date, format: String) -> String {
        let formatter = DateFormatter()
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.dateFormat = format
        return formatter.string(from: date)
    }

    private static var osVersion: String {
        let version = ProcessInfo.processInfo.operatingSystemVersion
        return "\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)"
    }

    private static var architecture: String {
        #if arch(arm64)
            "arm64"
        #elseif arch(x86_64)
            "x86_64"
        #else
            "unknown"
        #endif
    }
}
