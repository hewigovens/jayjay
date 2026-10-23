import JayJayCore
import SwiftUI

struct BisectSheet: View {
    let onCancel: () -> Void
    let onStart: (String, String, String) -> Void

    @State private var goodText: String
    @State private var badText: String
    @State private var rangeText = ""

    init(
        good: String,
        bad: String,
        onCancel: @escaping () -> Void,
        onStart: @escaping (String, String, String) -> Void
    ) {
        self.onCancel = onCancel
        self.onStart = onStart
        _goodText = State(initialValue: good)
        _badText = State(initialValue: bad)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(Localized.Repo.bisect)
                .font(.headline)
            labeledField(Localized.Repo.good, text: $goodText, identifier: AID.Bisect.sheetGood, prompt: Localized.Repo.goodEnd)
            labeledField(Localized.Repo.bad, text: $badText, identifier: AID.Bisect.sheetBad, prompt: Localized.Repo.badEnd)
            labeledField(Localized.Repo.range, text: $rangeText, identifier: AID.Bisect.sheetRange, prompt: Localized.Repo.revsetRange)
            Text("A range is used as-is. Otherwise the ends become (good)..(bad).")
                .font(.caption)
                .foregroundStyle(.secondary)
            HStack {
                Spacer()
                Button(Localized.Common.cancel, action: onCancel)
                    .keyboardShortcut(.cancelAction)
                Button(Localized.Repo.start) { onStart(goodText, badText, rangeText) }
                    .keyboardShortcut(.defaultAction)
                    .accessibilityIdentifier(AID.Bisect.sheetStart)
            }
        }
        .padding(20)
        .frame(width: 420)
    }

    private func labeledField(
        _ title: String,
        text: Binding<String>,
        identifier: String,
        prompt: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
            TextField(prompt, text: text)
                .textFieldStyle(.roundedBorder)
                .accessibilityIdentifier(identifier)
        }
    }
}
