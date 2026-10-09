import SwiftUI

extension View {
    func appEnvironment(_ settings: AppSettings) -> some View {
        environment(settings)
            .environment(\.jayjayTypography, settings.typography)
            .preferredColorScheme(settings.appearanceMode.colorScheme)
    }
}
