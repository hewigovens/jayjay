import SwiftUI

extension View {
    func appEnvironment(_ settings: AppSettings) -> some View {
        environment(settings)
            .environment(\.jayjayFontSize, settings.fontSize)
            .environment(\.jayjayFontFamily, settings.fontFamily)
            .preferredColorScheme(settings.appearanceMode.colorScheme)
    }
}
