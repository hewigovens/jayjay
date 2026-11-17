import SwiftUI

struct InlineTextButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        Label(configuration: configuration)
    }

    private struct Label: View {
        let configuration: Configuration
        @State private var hovered = false

        var body: some View {
            configuration.label
                .underline(hovered || configuration.isPressed)
                .contentShape(Rectangle())
                .onHover { hovered = $0 }
        }
    }
}
