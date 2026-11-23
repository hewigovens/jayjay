import SwiftUI

struct DiffStackMessageRow<Action: View>: View {
    /// The file whose header sits above: the icon centres under its file icon, which also starts the title where the path starts.
    let path: String
    let systemImage: String
    let title: String
    var description: String?
    @ViewBuilder var action: () -> Action

    @Environment(\.jayjayTypography) private var typography

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Image(systemName: systemImage)
                .jayjayFont(10)
                .foregroundStyle(.secondary)
                .frame(width: FileIcon.inkWidth(path: path, interfaceScale: typography.interfaceScale))
            VStack(alignment: .leading, spacing: 3) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(title)
                        .fontWeight(.medium)
                    action()
                        .buttonStyle(InlineTextButtonStyle())
                        .foregroundStyle(Color.accentColor)
                }
                .jayjayFont(12)
                if let description {
                    Text(description)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        }
        .padding(.horizontal, PaneLayout.detailInset)
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
    }
}

extension DiffStackMessageRow where Action == EmptyView {
    init(path: String, systemImage: String, title: String, description: String? = nil) {
        self.init(path: path, systemImage: systemImage, title: title, description: description) { EmptyView() }
    }
}
