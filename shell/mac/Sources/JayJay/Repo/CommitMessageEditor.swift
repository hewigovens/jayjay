import AppKit
import SwiftUI

struct CommitMessageEditor: View {
    @Binding var summary: String
    @Binding var details: String
    var bodyHeight: CGFloat?
    var focusSummaryAtEnd = false
    @State private var summarySelection: TextSelection?
    @State private var didSetInitialSelection = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Summary", text: $summary, selection: $summarySelection)
                .textFieldStyle(.plain)
                .jayjayFont(13, design: .monospaced)
                .padding(.horizontal, 11)
                .padding(.vertical, 6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .accessibilityIdentifier(AID.CommitBox.summary)

            // TextEditor has no native placeholder; overlay one while empty.
            TextEditor(text: $details)
                .jayjayFont(13, design: .monospaced)
                .scrollContentBackground(.hidden)
                .accessibilityIdentifier(AID.CommitBox.draft)
                .overlay(alignment: .topLeading) {
                    if details.isEmpty {
                        Text("Description (optional)")
                            .jayjayFont(13, design: .monospaced)
                            .foregroundStyle(.tertiary)
                            // Match the TextEditor's text origin so the placeholder aligns with the cursor.
                            .padding(.leading, 5)
                            .padding(.top, 0)
                            .allowsHitTesting(false)
                            .accessibilityHidden(true)
                    }
                }
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .frame(minHeight: bodyHeight ?? 50, maxHeight: bodyHeight ?? 100)
        }
        .onChange(of: summarySelection) { _, selection in
            guard focusSummaryAtEnd, !didSetInitialSelection, selection != nil else { return }
            didSetInitialSelection = true
            summarySelection = TextSelection(insertionPoint: summary.endIndex)
            // SwiftUI sets the insertion point without revealing it in a long single-line field.
            DispatchQueue.main.async {
                guard let editor = NSApp.keyWindow?.firstResponder as? NSTextView,
                      editor.isFieldEditor, editor.string == summary
                else { return }
                editor.scrollRangeToVisible(NSRange(location: (summary as NSString).length, length: 0))
            }
        }
    }
}
