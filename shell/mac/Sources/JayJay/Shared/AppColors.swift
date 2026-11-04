import JayJayCore
import JayJayDiffUI
import SwiftUI

struct AppColors: Equatable {
    private let theme: ColorTheme
    private let isDefault: Bool

    init(theme: ColorTheme, isDefault: Bool) {
        self.theme = theme
        self.isDefault = isDefault
    }

    var readingBackground: Color {
        Color(rgb: theme.colors.contextBg)
    }

    var navigationBackground: Color {
        theme.repo.navigationBg.map(Color.init(rgb:)) ?? Color(nsColor: .controlBackgroundColor)
    }

    var graphLine: Color {
        Color(rgb: theme.repo.graphLine)
    }

    var workspace: Color {
        Color(rgb: theme.repo.workspace)
    }

    var bookmark: Color {
        Color(rgb: theme.repo.bookmark)
    }

    var changeIdPrefix: Color {
        Color(rgb: theme.repo.changeIdPrefix)
    }

    var commitIdPrefix: Color {
        Color(rgb: theme.repo.commitIdPrefix)
    }

    var fileAdded: Color {
        isDefault ? .green : Color(rgb: theme.repo.fileAdded)
    }

    var fileRemoved: Color {
        isDefault ? .red : Color(rgb: theme.repo.fileRemoved)
    }

    var fileModified: Color {
        isDefault ? .blue : Color(rgb: theme.repo.fileModified)
    }

    var fileRenamed: Color {
        isDefault ? .blue : Color(rgb: theme.repo.fileRenamed)
    }

    var fileLfs: Color {
        isDefault ? .purple : Color(rgb: theme.repo.fileLfs)
    }

    var noteAccent: Color {
        isDefault ? .orange : Color(rgb: theme.repo.noteAccent)
    }

    func fileColor(for hunkType: HunkType) -> Color {
        switch hunkType {
            case .added, .copied: fileAdded
            case .removed: fileRemoved
            case .modified: fileModified
            case .renamed: fileRenamed
        }
    }

    func fileBadgeColor(for hunkType: HunkType) -> Color {
        guard isDefault, !theme.isDark else {
            return fileColor(for: hunkType)
        }
        switch hunkType {
            case .added, .copied: return Color(red: 28 / 255, green: 124 / 255, blue: 58 / 255)
            case .removed: return Color(red: 180 / 255, green: 35 / 255, blue: 24 / 255)
            case .modified, .renamed: return Color(red: 23 / 255, green: 92 / 255, blue: 211 / 255)
        }
    }
}

extension Color {
    /// From a packed `0xRRGGBB` value (core's shared design tokens are `u32`).
    init(rgb: UInt32) {
        self.init(
            red: Double((rgb >> 16) & 0xFF) / 255.0,
            green: Double((rgb >> 8) & 0xFF) / 255.0,
            blue: Double(rgb & 0xFF) / 255.0
        )
    }
}
