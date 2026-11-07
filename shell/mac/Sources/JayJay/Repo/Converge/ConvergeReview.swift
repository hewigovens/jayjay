import Foundation
import JayJayCore

@Observable
final class ConvergeReview {
    var plan: ConvergePlan?
    var selections: [ConvergeItem: String] = [:]
    var customDescription: String?
    var focusedItem: ConvergeItem?
    var isLoading = false
    var isApplying = false
    var isCurrent = false
    var wasStale = false
    var error: String?
    var requestId = UUID()

    var versions: [ConvergeVersion] {
        plan?.versions ?? []
    }

    func letter(of commitId: String) -> String {
        let index = versions.firstIndex { $0.commitId == commitId } ?? 0
        return String(UnicodeScalar(UInt8(65 + index % 26)))
    }

    func isSolved(_ item: ConvergeItem) -> Bool {
        guard let plan else { return false }
        switch item {
            case .description: if case .unsolved = plan.description {
                    return false
                }
            case .author: if case .unsolved = plan.author {
                    return false
                }
            case .parents: if case .unsolved = plan.parents {
                    return false
                }
            case .contents: break
        }
        return true
    }

    func isSame(_ item: ConvergeItem) -> Bool {
        switch item {
            case .description: Set(versions.map(\.description)).count == 1
            case .author: Set(versions.map(\.author)).count == 1
            case .parents: Set(versions.map(\.parents)).count == 1
            case .contents: false
        }
    }

    func isChosen(_ item: ConvergeItem) -> Bool {
        selections[item] != nil || (item == .description && customDescription != nil)
    }

    func isUnresolved(_ item: ConvergeItem) -> Bool {
        !isSolved(item) && !isChosen(item)
    }

    func automaticSource(_ item: ConvergeItem) -> String? {
        guard let plan else { return nil }
        let version: ConvergeVersion? = switch (item, plan.description, plan.author, plan.parents) {
            case let (.description, .solved(value), _, _): versions.first { $0.description == value }
            case let (.author, _, .solved(value), _): versions.first { $0.author == value }
            case let (.parents, _, _, .solved(value)): versions.first { $0.parents == value }
            case (.contents, _, _, _): contentsFollowParents ? nil : versions.first { $0.commitId == plan.contentsSource }
            default: nil
        }
        return version?.commitId
    }

    var contentsFollowParents: Bool {
        guard let plan, selections[.contents] == nil else { return false }
        guard case let .solved(solved) = plan.parents else { return true }
        return selections[.parents].map { id in versions.first { $0.commitId == id }?.parents != solved } ?? false
    }

    var cardItems: [ConvergeItem] {
        ConvergeItem.allCases.filter { !isSame($0) }
    }

    var sameItems: [ConvergeItem] {
        ConvergeItem.allCases.filter(isSame)
    }

    var unresolvedItems: [ConvergeItem] {
        ConvergeItem.allCases.filter(isUnresolved)
    }

    var automaticCount: Int {
        ConvergeItem.allCases.filter(isSolved).count
    }

    var expectedConflicts: [String]? {
        selections[.contents] == nil && !contentsFollowParents ? plan?.conflictedPaths : nil
    }

    var willConflict: Bool {
        !(expectedConflicts ?? []).isEmpty
    }

    var hint: String? {
        switch unresolvedItems.first {
            case .author: Localized.Converge.chooseAuthorHint
            case .description: Localized.Converge.chooseDescriptionHint
            case .parents: Localized.Converge.chooseParentsHint
            case .contents, nil: nil
        }
    }

    var canApply: Bool {
        plan != nil && isCurrent && !isLoading && !isApplying
            && plan?.effects.blockedReason == nil && unresolvedItems.isEmpty
    }

    var choices: ConvergeChoices {
        ConvergeChoices(
            author: selections[.author],
            parents: selections[.parents],
            description: customDescription.map { .custom($0) } ?? selections[.description].map { .fromVersion($0) },
            contents: selections[.contents]
        )
    }

    func use(_ commitId: String, for item: ConvergeItem) {
        selections[item] = commitId
        focusedItem = item
        if item == .description {
            customDescription = nil
        }
    }

    func keep(_ commitId: String) {
        for item in ConvergeItem.allCases {
            selections[item] = commitId
        }
        customDescription = nil
    }

    func editDescription() {
        guard customDescription == nil else { return }
        let source = selections[.description] ?? automaticSource(.description) ?? versions.first?.commitId
        customDescription = versions.first { $0.commitId == source }?.description ?? ""
        selections[.description] = nil
        focusedItem = .description
    }

    func useVersion(at index: Int) {
        guard let item = focusedItem, versions.indices.contains(index) else { return }
        use(versions[index].commitId, for: item)
    }

    func moveFocus(by delta: Int) {
        let items = cardItems
        guard !items.isEmpty else { return }
        let current = focusedItem.flatMap { items.firstIndex(of: $0) } ?? (delta > 0 ? -1 : items.count)
        focusedItem = items[(current + delta + items.count) % items.count]
    }

    func accept(_ plan: ConvergePlan) {
        self.plan = plan
        selections = [:]
        customDescription = nil
        focusedItem = unresolvedItems.first ?? cardItems.first
        isLoading = false
        isCurrent = true
    }
}
