import SwiftUI

struct LabeledRow: View {
    let label: String
    let value: String
    init(_ label: String, value: String) {
        self.label = label
        self.value = value
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label).jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
            Text(value).jayjayFont(11, design: .monospaced).textSelection(.enabled)
        }
    }
}
