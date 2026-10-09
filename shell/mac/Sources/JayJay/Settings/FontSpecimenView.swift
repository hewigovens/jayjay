import JayJayCore
import JayJayDiffUI
import SwiftUI

struct FontSpecimenView: View {
    @Environment(\.jayjayTypography) private var typography
    @State private var diffHeight: CGFloat?

    private static let specimen = fontSpecimen()

    var body: some View {
        let specimen = Self.specimen
        let directory = (specimen.path as NSString).deletingLastPathComponent + "/"
        let fileName = (specimen.path as NSString).lastPathComponent
        VStack(alignment: .leading, spacing: 12) {
            Text(specimen.summary)
                .jayjayFont(13, weight: .semibold)
                .accessibilityIdentifier(AID.Settings.fontPreview)
            HStack(spacing: 8) {
                IdentifierText(value: specimen.changeId, prefixLength: 4, prefixStyle: .changeId)
                (Text(directory).foregroundStyle(.secondary) + Text(fileName).fontWeight(.medium))
                    .jayjayFont(12)
            }
            Divider()
            NativeDiffView(diff: specimen.diff, onContentHeightChanged: { diffHeight = $0 })
                .environment(\.diffTypography, typography.diffTypography)
                .frame(height: diffHeight ?? specimen.diff.estimatedCardHeight(typography: typography))
            Divider()
            Text("Font choices stay the same when you change themes.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }
}
