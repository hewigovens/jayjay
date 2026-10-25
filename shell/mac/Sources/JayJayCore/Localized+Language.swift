import Foundation
import Observation
import Synchronization

final class CurrentLanguage: Observable, Sendable {
    static let shared = CurrentLanguage()
    private let registrar = ObservationRegistrar()
    private let storedLocale = Mutex(Locale.autoupdatingCurrent)

    var locale: Locale {
        registrar.access(self, keyPath: \.locale)
        return storedLocale.withLock { $0 }
    }

    func set(locale: Locale) {
        registrar.withMutation(of: self, keyPath: \.locale) {
            storedLocale.withLock { $0 = locale }
        }
    }
}

public extension Localized {
    /// Formatters use this instead of `Locale.current`, which keeps the launch language.
    static var locale: Locale {
        CurrentLanguage.shared.locale
    }

    /// Views that read `Localized` in their body re-render, as they do for any observed state.
    static func setLanguage(_ tag: String?) {
        // `Locale.current` and `Locale.preferredLanguages` keep any language the app launched with, so System reads the global list.
        let languages = tag.map { [$0] }
            ?? UserDefaults.standard.persistentDomain(forName: UserDefaults.globalDomain)?["AppleLanguages"] as? [String]
            ?? Locale.preferredLanguages
        JayJayCore.setPreferredLanguages(languages: languages)
        var components = Locale.Components(identifier: languages.first ?? "en")
        components.region = Locale.current.region
        CurrentLanguage.shared.set(locale: Locale(components: components))
    }

    internal static func message(id: String, args: [String: LocalizedArgument]) -> String {
        _ = CurrentLanguage.shared.locale
        return localizedMessage(id: id, args: args)
    }
}
