import SwiftUI

struct SettingsSectionCard<Content: View>: View {
    let title: String?
    let subtitle: String?
    let content: Content

    @Environment(\.colorScheme) private var colorScheme

    init(title: String, subtitle: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.subtitle = subtitle
        self.content = content()
    }

    init(title: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.subtitle = nil
        self.content = content()
    }

    init(@ViewBuilder content: () -> Content) {
        self.title = nil
        self.subtitle = nil
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if let title {
                VStack(alignment: .leading, spacing: 4) {
                    Text(title).jayjayFont(16, weight: .semibold).foregroundStyle(Color.primary.opacity(0.84))
                    if let subtitle {
                        Text(subtitle).jayjayFont(12).foregroundStyle(Color.secondary)
                    }
                }
            }
            content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(RoundedRectangle(cornerRadius: 24, style: .continuous).fill(cardFill))
        .overlay(RoundedRectangle(cornerRadius: 24, style: .continuous).stroke(cardStroke, lineWidth: 1))
    }

    private var cardFill: Color { colorScheme == .dark ? Color.white.opacity(0.08) : Color.white.opacity(0.72) }
    private var cardStroke: Color { colorScheme == .dark ? Color.white.opacity(0.14) : Color.white.opacity(0.82) }
}

struct SettingsToggleRow: View {
    let title: String
    let description: String
    @Binding var isOn: Bool
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title).jayjayFont(13, weight: .semibold).foregroundStyle(Color.primary.opacity(0.8))
                Text(description).jayjayFont(11).foregroundStyle(Color.secondary).fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 16)
            Toggle("", isOn: $isOn).labelsHidden().toggleStyle(.switch).tint(Color(red: 0.18, green: 0.41, blue: 0.9))
        }
        .padding(14)
        .background(RoundedRectangle(cornerRadius: 18, style: .continuous)
            .fill(colorScheme == .dark ? Color.white.opacity(0.06) : Color.white.opacity(0.54)))
    }
}

struct LabeledRow: View {
    let label: String
    let value: String
    init(_ label: String, value: String) { self.label = label; self.value = value }
    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label).jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
            Text(value).jayjayFont(11, design: .monospaced).textSelection(.enabled)
        }
    }
}
