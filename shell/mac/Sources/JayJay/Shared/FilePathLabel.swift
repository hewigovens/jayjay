import SwiftUI

struct FilePathLabel: View {
    let path: String
    var oldPath: String?
    var size: CGFloat = 12

    var body: some View {
        HStack(spacing: 4) {
            if let oldPath {
                text(for: oldPath)
                    .foregroundStyle(.secondary)
                Text("→")
                    .foregroundStyle(.secondary)
                    .fixedSize()
            }
            text(for: path)
        }
        .jayjayFont(size)
        .lineLimit(1)
        .truncationMode(.head)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(oldPath.map { "Renamed from \($0) to \(path)" } ?? path)
    }

    private func text(for path: String) -> Text {
        guard let slash = path.lastIndex(of: "/") else {
            return Text(path).fontWeight(.medium)
        }
        let filenameStart = path.index(after: slash)
        let directory = Text(String(path[..<filenameStart])).foregroundStyle(.secondary)
        let filename = Text(String(path[filenameStart...])).fontWeight(.medium)
        return Text("\(directory)\(filename)")
    }
}
