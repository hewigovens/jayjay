import SwiftUI

private struct AppEnvironmentModifier: ViewModifier {
    let settings: AppSettings

    func body(content: Content) -> some View {
        content
            .environment(settings)
            .environment(ShortcutStore.shared)
            .environment(\.jayjayTypography, settings.typography)
            .environment(\.appThemes, settings.appThemes)
            .environment(\.diffPalettes, settings.diffPalettes)
            .preferredColorScheme(settings.appearanceMode.colorScheme)
    }
}

extension View {
    func appEnvironment(_ settings: AppSettings) -> some View {
        modifier(AppEnvironmentModifier(settings: settings))
    }
}
