import SwiftUI

struct CopyableRow: View {
    let label: String
    let value: String
    let copyValue: String

    init(_ label: String, value: String, copyValue: String? = nil) {
        self.label = label
        self.value = value
        self.copyValue = copyValue ?? value
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label).jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
            Text(value).jayjayFont(11, design: .monospaced).textSelection(.enabled)
            CopyIconButton(value: copyValue, help: "Copy \(label.lowercased())")
        }
    }
}

struct CopyIconButton: View {
    let value: String
    let help: String
    @State private var copied = false

    var body: some View {
        Button {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(value, forType: .string)
            copied = true
            Task { try? await Task.sleep(for: .seconds(1.5))
                copied = false
            }
        } label: {
            Image(systemName: copied ? "checkmark" : "doc.on.doc")
                .jayjayFont(9)
                .foregroundStyle(copied ? Color.green : Color.secondary.opacity(0.5))
        }
        .buttonStyle(.plain)
        .help(help)
    }
}

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
