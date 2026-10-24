import AppKit
import Foundation
import JayJayCore

enum FeedbackEmail {
    private static let recipient = "hi@hewig.dev"
    private static let subject = "JayJay Feedback"

    static func composeURL(askingFor log: URL? = nil) -> URL {
        var components = URLComponents()
        components.scheme = "mailto"
        components.path = recipient
        components.queryItems = [URLQueryItem(name: "subject", value: subject)]
        if let log {
            components.queryItems?.append(URLQueryItem(name: "body", value: Localized.Common.attachDebugLog(path: log.path) + "\n\n"))
        }
        return components.url!
    }

    @MainActor
    @discardableResult
    static func open(
        attaching log: URL? = DebugMode.feedbackLog,
        using openURL: (URL) -> Bool = { NSWorkspace.shared.open($0) },
        onFailure: (() -> Void)? = nil
    ) -> Bool {
        if let log, defaultClientIsMail, composeInMail(attaching: log) {
            return true
        }
        guard openURL(composeURL(askingFor: log)) else {
            if let onFailure {
                onFailure()
            } else {
                showFailureAlert()
            }
            return false
        }
        if let log {
            NSWorkspace.shared.activateFileViewerSelecting([log])
        }
        return true
    }

    /// The compose-email share service always drives Mail.app, even when the user's mail client is another app.
    private static var defaultClientIsMail: Bool {
        let client = NSWorkspace.shared.urlForApplication(toOpen: composeURL())
        return client.flatMap { Bundle(url: $0)?.bundleIdentifier } == "com.apple.mail"
    }

    @MainActor
    private static func composeInMail(attaching log: URL) -> Bool {
        guard let service = NSSharingService(named: .composeEmail), service.canPerform(withItems: [log]) else {
            return false
        }
        service.recipients = [recipient]
        service.subject = subject
        service.perform(withItems: [log])
        return true
    }

    @MainActor
    private static func showFailureAlert() {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = Localized.Common.unableToOpenMail
        alert.informativeText = Localized.Common.noEmailApp(address: recipient)
        alert.addButton(withTitle: Localized.Repo.ok)
        alert.runModal()
    }
}
