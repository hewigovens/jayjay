import AppKit
import SwiftUI

struct StatusBarItemView: View {
    let item: StatusBarItem

    var body: some View {
        switch item {
            case let .text(_, icon, text, tooltip):
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
                .buttonStyle(.plain)
                .help(tooltip ?? url.absoluteString)
                .onHover { inside in
                    if inside {
                        NSCursor.pointingHand.push()
                    } else {
                        NSCursor.pop()
                    }
                }

            case let .action(_, icon, text, perform):
                Button(action: perform) {
                    HStack(spacing: 3) {
                        Image(systemName: icon).jayjayFont(10)
                        Text(text)
                    }
                }
                .buttonStyle(.plain)
        }
    }
}
