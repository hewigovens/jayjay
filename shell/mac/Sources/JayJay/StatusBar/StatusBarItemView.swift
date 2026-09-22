import AppKit
import SwiftUI

struct StatusBarItemView: View {
    let item: StatusBarItem

    var body: some View {
        switch item {
            case let .text(_, icon, text, tooltip, _):
                HStack(spacing: 3) {
                    if let icon {
                        Image(systemName: icon).jayjayFont(10)
                    }
                    Text(text)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                .help(tooltip ?? "")

            case let .link(_, icon, text, url, tooltip):
                Button {
                    NSWorkspace.shared.open(url)
                } label: {
                    HStack(spacing: 3) {
                        Text(text)
                        Image(systemName: icon).jayjayFont(10)
                    }
                }
                .buttonStyle(StatusBarButtonStyle())
                .help(tooltip ?? url.absoluteString)

            case let .action(_, icon, text, tooltip, _, perform):
                Button(action: perform) {
                    HStack(spacing: 3) {
                        Image(systemName: icon).jayjayFont(10)
                        Text(text)
                            .lineLimit(1)
                            .truncationMode(.tail)
                    }
                }
                .buttonStyle(StatusBarButtonStyle())
                .help(tooltip ?? text)
        }
    }
}

private struct StatusBarButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        Label(configuration: configuration)
    }

    private struct Label: View {
        let configuration: Configuration
        @State private var hovered = false

        var body: some View {
            configuration.label
                .foregroundStyle(hovered || configuration.isPressed ? AnyShapeStyle(.primary) : AnyShapeStyle(.secondary))
                .contentShape(Rectangle())
                .onHover { inside in
                    hovered = inside
                    if inside {
                        NSCursor.pointingHand.push()
                    } else {
                        NSCursor.pop()
                    }
                }
        }
    }
}
