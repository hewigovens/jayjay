import SwiftUI

struct SettingsLabel: View {
    let title: String
    let icon: String
    let iconScale: Image.Scale

    init(_ title: String, icon: String, iconScale: Image.Scale = .medium) {
        self.title = title
        self.icon = icon
        self.iconScale = iconScale
    }

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .imageScale(iconScale)
                .frame(width: 16, alignment: .center)
                .foregroundStyle(.secondary)
            Text(title)
        }
    }
}
