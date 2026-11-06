import JayJayCore
import SwiftUI

struct ConvergeChoiceCard<Extra: View, Content: View>: View {
    let item: ConvergeItem
    let review: ConvergeReview
    var note: String?
    @ViewBuilder let extra: () -> Extra
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 8) {
                Text(item.title).jayjayFont(12, weight: .semibold)
                Text(status.text).jayjayFont(11, weight: .medium).foregroundStyle(status.color)
                Spacer()
                if let note = note ?? defaultNote {
                    Text(note).jayjayFont(11).foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            Divider()
            HStack(spacing: 14) {
                ForEach(Array(review.versions.enumerated()), id: \.element.commitId) { index, version in
                    useButton(version, shortcut: isFocused ? [AppShortcut.acceptLeftHunk, .acceptRightHunk].dropFirst(index).first : nil)
                }
                extra()
                Spacer()
            }
            .jayjayFont(11)
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            Divider()
            content()
        }
        .background(Color(nsColor: .textBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 7))
        .overlay(
            RoundedRectangle(cornerRadius: 7)
                .stroke(isFocused ? Color.accentColor : Color.primary.opacity(0.12), lineWidth: isFocused ? 1.5 : 1)
        )
        .contentShape(Rectangle())
        .onTapGesture { review.focusedItem = item }
    }

    private var isFocused: Bool {
        review.focusedItem == item
    }

    private var status: (text: String, color: Color) {
        if item == .description, review.customDescription != nil {
            return (Localized.Converge.edited, .green)
        }
        if let id = review.selections[item] {
            return (Localized.Converge.using(version: review.letter(of: id)), .green)
        }
        if review.isUnresolved(item) {
            return (Localized.Converge.unresolved, .orange)
        }
        if item == .contents {
            if let paths = review.expectedConflicts, !paths.isEmpty {
                return (Localized.Converge.willConflict(count: paths.count), .orange)
            }
            if review.contentsFollowParents {
                return (Localized.Converge.followsParents, .secondary)
            }
        }
        return (Localized.Converge.merged, .green)
    }

    private var defaultNote: String? {
        if review.isUnresolved(item) {
            return Localized.Converge.versionsDisagree
        }
        guard !review.isChosen(item), let source = review.automaticSource(item) else { return nil }
        return Localized.Converge.resultKeeps(version: review.letter(of: source))
    }

    private func useButton(_ version: ConvergeVersion, shortcut: AppShortcut?) -> some View {
        let letter = review.letter(of: version.commitId)
        let selected = review.selections[item] == version.commitId
        return Button {
            review.use(version.commitId, for: item)
        } label: {
            HStack(spacing: 4) {
                if selected {
                    Image(systemName: "checkmark").jayjayFont(9, weight: .bold)
                }
                Text(Localized.Converge.useVersion(version: letter)).fontWeight(selected ? .semibold : .medium)
                if let shortcut {
                    Text(shortcut.symbol).foregroundStyle(.tertiary)
                }
            }
            .foregroundStyle(Color.accentColor)
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(AID.Converge.use(item.rawValue, letter))
    }
}
