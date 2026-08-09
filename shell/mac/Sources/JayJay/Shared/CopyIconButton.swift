import SwiftUI

struct CopyIconButton: View {
    let value: String
    let help: String
    var label: String?
    @State private var copied = false

    var body: some View {
        Button {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(value, forType: .string)
            copied = true
            Task {
                try? await Task.sleep(for: .seconds(1.5))
                copied = false
            }
        } label: {
            HStack(spacing: 4) {
                Image(systemName: copied ? "checkmark" : "doc.on.doc")
                    .jayjayFont(9)
                if let label {
                    Text(copied ? "Copied" : label)
                        .font(.system(size: 11))
                }
            }
            .foregroundStyle(copied ? Color.green : Color.secondary.opacity(label == nil ? 0.5 : 1))
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
