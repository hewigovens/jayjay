import SwiftUI

struct HeaderActionButtonStyle: ButtonStyle {
    var isActive = false

    func makeBody(configuration: Configuration) -> some View {
        Label(configuration: configuration, isActive: isActive)
    }

    private struct Label: View {
        let configuration: Configuration
        let isActive: Bool
        @State private var hovered = false

        var body: some View {
            configuration.label
                .jayjayFont(11)
                .foregroundStyle(foreground)
                .padding(.horizontal, 6)
                .padding(.vertical, 3)
                .background(
                    hovered || configuration.isPressed ? Color.primary.opacity(0.07) : .clear,
                    in: RoundedRectangle(cornerRadius: 5, style: .continuous)
                )
                .contentShape(Rectangle())
                .onHover { hovered = $0 }
        }

        private var foreground: AnyShapeStyle {
            if isActive {
                return AnyShapeStyle(Color.accentColor)
            }
            return hovered ? AnyShapeStyle(.primary) : AnyShapeStyle(.secondary)
        }
    }
}
