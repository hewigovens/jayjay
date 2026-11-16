import JayJayCore
import SwiftUI

struct SettingsDiffPage: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section(Localized.Settings.display) {
                Toggle(isOn: Binding(
                    get: { settings.sideBySideDiff },
                    set: { settings.sideBySideDiff = $0 }
                )) {
                    SettingsLabel(Localized.Settings.sideBySide, icon: "rectangle.split.2x1")
                }
                Toggle(isOn: Binding(
                    get: { settings.ignoreWhitespace },
                    set: { settings.ignoreWhitespace = $0 }
                )) {
                    SettingsLabel(Localized.Settings.ignoreWhitespace, icon: "space")
                }
                Toggle(isOn: Binding(
                    get: { settings.treeFileList },
                    set: { settings.treeFileList = $0 }
                )) {
                    SettingsLabel(Localized.Settings.treeView, icon: "list.bullet.indent")
                }
                Toggle(isOn: Binding(
                    get: { settings.hideReviewedFiles },
                    set: { settings.hideReviewedFiles = $0 }
                )) {
                    SettingsLabel(Localized.Settings.hideReviewed, icon: "eye.slash")
                }
                .accessibilityLabel(Localized.Settings.hideReviewed)
                .accessibilityIdentifier(AID.Settings.hideReviewedFiles)
                Toggle(isOn: Binding(
                    get: { settings.autoExpandDescription },
                    set: { settings.autoExpandDescription = $0 }
                )) {
                    SettingsLabel(Localized.Settings.autoExpand, icon: "arrow.up.and.line.horizontal.and.arrow.down")
                }
            }

            Section(Localized.Settings.largeFiles) {
                Toggle(isOn: Binding(
                    get: { settings.hideGitLfsDiffs },
                    set: { settings.hideGitLfsDiffs = $0 }
                )) {
                    SettingsLabel(Localized.Settings.hideLfs, icon: "externaldrive")
                }
            }
        }
        .formStyle(.grouped)
    }
}
