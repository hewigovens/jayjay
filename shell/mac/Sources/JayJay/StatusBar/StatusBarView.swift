import AppKit
import SwiftUI

struct StatusBarView: View {
    let leadingItems: [StatusBarItem]
    let trailingItems: [StatusBarItem]

    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(leadingItems.enumerated()), id: \.element.id) { index, item in
                if index > 0 { separator }
                StatusBarItemView(item: item)
            }
            Spacer()
            ForEach(Array(trailingItems.enumerated()), id: \.element.id) { index, item in
                if index > 0 { separator }
                StatusBarItemView(item: item)
            }
        }
        .jayjayFont(11)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 12)
        .padding(.vertical, 5)
        .background(.bar)
    }

    private var separator: some View {
        Text("·")
            .foregroundStyle(.quaternary)
            .padding(.horizontal, 4)
    }
}

private struct StatusBarItemView: View {
    let item: StatusBarItem

    var body: some View {
        switch item {
            case .text(_, let text):
                Text(text)
                    .lineLimit(1)
                    .truncationMode(.middle)

            case .link(_, let icon, let text, let url):
                Button {
                    NSWorkspace.shared.open(url)
                } label: {
                    HStack(spacing: 3) {
                        Text(text)
                        Image(systemName: icon).jayjayFont(10)
                    }
                }
                .buttonStyle(.plain)
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
                        PickerOptionView(option: option)
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

private struct PickerOptionView: View {
    let option: StatusBarPickerOption

    var body: some View {
        if let children = option.children, !children.isEmpty {
            Menu(option.label) {
                ForEach(children) { child in
                    PickerLeafView(option: child)
                }
            }
        } else {
            PickerLeafView(option: option)
        }
    }
}

private struct PickerLeafView: View {
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
