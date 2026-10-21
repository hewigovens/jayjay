import Foundation
import JayJayCore

enum DebugMode {
    static let defaultsKey = "jayjay.debugMode"

    static var isEnabled: Bool {
        UserDefaults.standard.object(forKey: defaultsKey) as? Bool ?? isPrereleaseBuild()
    }

    static var feedbackLog: URL? {
        guard isEnabled, let path = logFilePath() else { return nil }
        return URL(fileURLWithPath: path)
    }
}
