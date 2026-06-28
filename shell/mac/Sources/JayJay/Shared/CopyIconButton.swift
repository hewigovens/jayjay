import SwiftUI

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
