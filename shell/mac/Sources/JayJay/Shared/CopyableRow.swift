import SwiftUI

struct CopyableRow: View {
    let label: String
    let value: String
    let copyValue: String
    var bold: Bool = false
    /// When set, the first N characters render bold (the shortest unique change-id
    /// prefix); the remainder stays normal weight.
    var emphasizedPrefix: Int?

    init(
        _ label: String,
        value: String,
        copyValue: String? = nil,
        bold: Bool = false,
        emphasizedPrefix: Int? = nil
    ) {
        self.label = label
        self.value = value
        self.copyValue = copyValue ?? value
        self.bold = bold
        self.emphasizedPrefix = emphasizedPrefix
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label).jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
            valueContent
            CopyIconButton(value: copyValue, help: "Copy \(label.lowercased())")
        }
    }

    @ViewBuilder
    private var valueContent: some View {
        if emphasizedPrefix != nil || bold {
            // Bake explicit fonts so per-segment weight isn't reset by jayjayFont
            // (which applies a single uniform weight across the whole Text).
            emphasizedValueText.textSelection(.enabled)
        } else {
            Text(value).jayjayFont(11, design: .monospaced).textSelection(.enabled)
        }
    }

    private var emphasizedValueText: Text {
        let regular = Font.system(size: 11, design: .monospaced)
        let boldFont = Font.system(size: 11, design: .monospaced).weight(.bold)
        if let prefixLen = emphasizedPrefix, prefixLen > 0 {
            // Clamp to the displayed length (a 12-char id may need a longer prefix
            // in very large repos), matching the DAG/GPUI split behavior.
            let n = min(prefixLen, value.count)
            let split = value.index(value.startIndex, offsetBy: n)
            var attr = AttributedString(value[..<split])
            attr.font = boldFont
            var rest = AttributedString(value[split...])
            rest.font = regular
            attr.append(rest)
            return Text(attr)
        }
        return Text(value).font(bold ? boldFont : regular)
    }
}
