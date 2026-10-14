import JayJayCore
import JayJayDiffUI
import SwiftUI

/// The one settings preview: fonts and the selected diff palette on the code-font sample. `scheme` pins the diff to the light or dark side being edited.
struct SettingsPreview: View {
    var scheme: ColorScheme?
    @Environment(AppSettings.self) private var settings
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography
    @State private var diffHeight: CGFloat?

    private static let specimen = fontSpecimen()

    var body: some View {
        let specimen = Self.specimen
        let directory = (specimen.path as NSString).deletingLastPathComponent + "/"
        let fileName = (specimen.path as NSString).lastPathComponent
        let diffScheme = scheme ?? colorScheme
        let palettes = settings.diffPalettes
        VStack(alignment: .leading, spacing: 12) {
            Text(specimen.summary)
                .jayjayFont(13, weight: .semibold)
            HStack(spacing: 8) {
                IdentifierText(value: specimen.changeId, prefixLength: 4, prefixStyle: .changeId)
                Text("\(Text(directory).foregroundStyle(.secondary))\(Text(fileName).fontWeight(.medium))")
                    .jayjayFont(12)
            }
            NativeDiffView(diff: specimen.diff, onContentHeightChanged: { diffHeight = $0 })
                .environment(\.diffTypography, typography.diffTypography)
                .environment(\.diffPalettes, palettes)
                .environment(\.colorScheme, diffScheme)
                .frame(height: diffHeight ?? specimen.diff.estimatedCardHeight(typography: typography))
                .padding(.vertical, 6)
                .background(
                    AppColors.readingBackground(diffScheme, palettes: palettes),
                    in: RoundedRectangle(cornerRadius: 8, style: .continuous)
                )
        }
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Settings.preview)
    }
}
