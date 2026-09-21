import SwiftUI

struct SettingsDiffPage: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section("Display") {
                Toggle(isOn: Binding(
                    get: { settings.sideBySideDiff },
                    set: { settings.sideBySideDiff = $0 }
                )) {
                    SettingsLabel("Side-by-side diff", icon: "rectangle.split.2x1")
                }
                Toggle(isOn: Binding(
                    get: { settings.ignoreWhitespace },
                    set: { settings.ignoreWhitespace = $0 }
                )) {
                    SettingsLabel("Ignore whitespace changes", icon: "space")
                }
                Toggle(isOn: Binding(
                    get: { settings.treeFileList },
                    set: { settings.treeFileList = $0 }
                )) {
                    SettingsLabel("Tree view for files", icon: "list.bullet.indent")
                }
                Toggle(isOn: Binding(
                    get: { settings.hideReviewedFiles },
                    set: { settings.hideReviewedFiles = $0 }
                )) {
                    SettingsLabel("Hide reviewed files", icon: "eye.slash")
                }
                Toggle(isOn: Binding(
                    get: { settings.autoExpandDescription },
                    set: { settings.autoExpandDescription = $0 }
                )) {
                    SettingsLabel("Auto-expand descriptions", icon: "arrow.up.and.line.horizontal.and.arrow.down")
                }
            }

            Section("Large Files") {
                Toggle(isOn: Binding(
                    get: { settings.hideGitLfsDiffs },
                    set: { settings.hideGitLfsDiffs = $0 }
                )) {
                    SettingsLabel("Hide Git LFS-backed files", icon: "externaldrive")
                }
            }
        }
        .formStyle(.grouped)
    }
}
