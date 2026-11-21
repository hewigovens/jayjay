import JayJayCore
import SwiftUI

public struct ReviewFileLabel: View {
    var file: ReviewFileSummary

    public init(file: ReviewFileSummary) {
        self.file = file
    }

    public var body: some View {
        HStack(spacing: 8) {
            FileKindBadge(kind: file.kind)
            VStack(alignment: .leading, spacing: 2) {
                Text(file.name)
                    .lineLimit(1)
                    .opacity(file.review == .reviewed ? 0.55 : 1)
                Text(file.sourcePath.map { $0 + " → " + file.path } ?? file.folder)
                    .font(.footnote.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer(minLength: 4)
            if file.noteCount > 0 {
                Label(String(file.noteCount), systemImage: "text.bubble")
                    .font(.footnote)
                    .foregroundStyle(ReviewColors.note)
                    .accessibilityLabel(Localized.Web.notes + ", " + String(file.noteCount))
            }
            Text("+" + String(file.added) + " −" + String(file.removed))
                .font(.footnote.monospacedDigit())
                .foregroundStyle(.secondary)
                .accessibilityLabel(Localized.Diff.added + " " + String(file.added) + ", " + Localized.Diff.removed + " " + String(file.removed))
        }
        .accessibilityElement(children: .combine)
    }
}
