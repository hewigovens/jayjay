import AppKit
import Foundation

enum FeedbackEmail {
    private static let recipient = "hi@hewig.dev"

    static let composeURL: URL = {
        var components = URLComponents()
        components.scheme = "mailto"
        components.path = recipient
        components.queryItems = [URLQueryItem(name: "subject", value: "JayJay Feedback")]
        return components.url!
    }()

    @MainActor
    @discardableResult
    static func open(
        using openURL: (URL) -> Bool = { NSWorkspace.shared.open($0) },
        onFailure: (() -> Void)? = nil
    ) -> Bool {
        guard openURL(composeURL) else {
            if let onFailure {
                onFailure()
            } else {
                showFailureAlert()
            }
            return false
        }
        return true
    }

    @MainActor
    private static func showFailureAlert() {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = "Unable to Open Mail"
        alert.informativeText = "No email app could be opened. You can send feedback to \(recipient)."
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
