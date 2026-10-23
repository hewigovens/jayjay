import JayJayCore
import SwiftUI

enum SettingsPage: String, CaseIterable, Identifiable {
    case appearance
    case fonts
    case diff
    case workflow
    case integrations
    case jujutsu
    case dataPrivacy
    case about

    var id: Self {
        self
    }

    var title: String {
        switch self {
            case .appearance: Localized.Settings.pageAppearance
            case .fonts: Localized.Settings.pageFonts
            case .diff: Localized.Settings.pageDiff
            case .workflow: Localized.Settings.pageWorkflow
            case .integrations: Localized.Settings.pageIntegrations
            case .jujutsu: Localized.Settings.pageJujutsu
            case .dataPrivacy: Localized.Settings.pageDataPrivacy
            case .about: Localized.Settings.pageAbout
        }
    }

    var symbol: String {
        switch self {
            case .appearance: "paintbrush"
            case .fonts: "textformat"
            case .diff: "doc.text.magnifyingglass"
            case .workflow: "checklist"
            case .integrations: "wrench.and.screwdriver"
            case .jujutsu: "arrow.triangle.branch"
            case .dataPrivacy: "hand.raised"
            case .about: "info.circle"
        }
    }

    var color: Color {
        switch self {
            case .appearance: .purple
            case .fonts: .blue
            case .diff: .blue
            case .workflow: .orange
            case .integrations: .teal
            case .jujutsu: .green
            case .dataPrivacy: .indigo
            case .about: .blue
        }
    }
}
