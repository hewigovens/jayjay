import SwiftUI

enum SettingsPage: String, CaseIterable, Identifiable {
    case appearance
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
            case .appearance: "Appearance"
            case .diff: "Diff & Files"
            case .workflow: "Workflow"
            case .integrations: "Integrations"
            case .jujutsu: "Jujutsu"
            case .dataPrivacy: "Data & Privacy"
            case .about: "About"
        }
    }

    var symbol: String {
        switch self {
            case .appearance: "paintbrush"
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
            case .diff: .blue
            case .workflow: .orange
            case .integrations: .teal
            case .jujutsu: .green
            case .dataPrivacy: .indigo
            case .about: .blue
        }
    }
}
