import AppKit
import SwiftUI

struct StatusBarItemView: View {
    let item: StatusBarItem

    var body: some View {
        switch item {
            case .text(_, let text):
                Text(text)
                    .lineLimit(1)
                    .truncationMode(.middle)

            case .link(_, let icon, let text, let url, let tooltip):
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
                    if inside { NSCursor.pointingHand.push() } else { NSCursor.pop() }
                }

            case .action(_, let icon, let text, let perform):
                Button(action: perform) {
                    HStack(spacing: 3) {
                        Image(systemName: icon).jayjayFont(10)
                        Text(text)
                    }
                }
                .buttonStyle(.plain)

            case .picker(_, let icon, let label, let options):
                Image(systemName: icon).jayjayFont(10)
                Menu {
                    ForEach(options) { option in
                        StatusBarPickerOptionView(option: option)
                    }
                } label: {
                    Text(label)
                        .jayjayFont(11, weight: .medium, design: .monospaced)
                        .lineLimit(1)
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
        }
    }
}

private struct StatusBarPickerOptionView: View {
    let option: StatusBarPickerOption

    var body: some View {
        if let children = option.children, !children.isEmpty {
            Menu(option.label) {
                ForEach(children) { child in
                    StatusBarPickerLeafView(option: child)
                }
            }
        } else {
            StatusBarPickerLeafView(option: option)
        }
    }
}

private struct StatusBarPickerLeafView: View {
    let option: StatusBarPickerOption

    var body: some View {
        if let action = option.action {
            if option.destructive {
                Button(option.label, role: .destructive, action: action)
            } else {
                Button(action: action) {
                    if let icon = option.icon {
                        Label(option.label, systemImage: icon)
                    } else {
                        Text(option.label)
                    }
                }
                .disabled(option.disabled)
            }
        } else {
            Button {} label: {
                if let icon = option.icon {
                    Label(option.label, systemImage: icon)
                } else {
                    Text(option.label)
                }
            }
            .disabled(true)
        }
    }
}
