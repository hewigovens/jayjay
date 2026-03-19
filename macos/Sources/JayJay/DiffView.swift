import SwiftUI
import JayJayBindings

struct DiffHunkView: View {
    let hunk: DiffHunk

    @State private var isExpanded = false

    var body: some View {
        DisclosureGroup(isExpanded: $isExpanded) {
            VStack(alignment: .leading, spacing: 0) {
                if let old = hunk.oldContent {
                    ForEach(old.components(separatedBy: "\n"), id: \.self) { line in
                        Text("- \(line)")
                            .jayjayFont(11, design: .monospaced)
                            .foregroundStyle(.red)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color.red.opacity(0.05))
                    }
                }
                if let new = hunk.newContent {
                    ForEach(new.components(separatedBy: "\n"), id: \.self) { line in
                        Text("+ \(line)")
                            .jayjayFont(11, design: .monospaced)
                            .foregroundStyle(.green)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color.green.opacity(0.05))
                    }
                }
                if hunk.oldContent == nil && hunk.newContent == nil {
                    Text("No textual preview available.")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .padding(.vertical, 4)
                }
            }
            .textSelection(.enabled)
        } label: {
            HStack(spacing: 6) {
                Image(systemName: hunkIcon)
                    .foregroundStyle(hunkColor)
                    .jayjayFont(11)
                Text(hunk.path)
                    .jayjayFont(13, design: .monospaced)
            }
        }
    }

    private var hunkIcon: String {
        switch hunk.hunkType {
        case .added: "plus.circle.fill"
        case .removed: "minus.circle.fill"
        case .modified: "pencil.circle.fill"
        }
    }

    private var hunkColor: Color {
        switch hunk.hunkType {
        case .added: .green
        case .removed: .red
        case .modified: .orange
        }
    }
}
