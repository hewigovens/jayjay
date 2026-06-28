import AppKit
import JayJayCore
import SwiftUI

/// Preview-then-submit panel for a stack of GitHub PRs built from a linear jj
/// change stack (`trunk()..tip`): one PR per change with dependent bases. Submit
/// is idempotent — re-running updates the existing PRs.
struct StackedPrPanel: View {
    let viewModel: RepoViewModel
    let tipRev: String
    let onDismiss: () -> Void

    @State private var stack: Stack?
    @State private var results: StackedPrResult?
    @State private var isWorking = false
    @State private var aiNaming = false
    @State private var errorMessage: String?
    /// Edited / AI-generated branch names, keyed by change-id. Falls back to the
    /// name core proposed when absent.
    @State private var editedNames: [String: String] = [:]
    /// The change-id whose branch name is currently being edited inline.
    @State private var editingChangeId: String?
    @FocusState private var focusedChangeId: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(spacing: 8) {
                Label(
                    results == nil ? "Stacked Pull Requests" : "Stacked PRs Submitted",
                    systemImage: "square.stack.3d.up.fill"
                )
                .jayjayFont(15, weight: .semibold)
                if isWorking {
                    ProgressView().controlSize(.small)
                }
            }

            if let results {
                resultsBody(results)
            } else {
                previewBody
            }
        }
        .padding(20)
        .frame(width: 480)
        .task { await loadStack() }
    }

    // MARK: - Preview

    @ViewBuilder
    private var previewBody: some View {
        if let stack {
            if stack.layers.isEmpty {
                message("No changes above trunk() to stack.")
                closeRow
            } else {
                let count = stack.layers.count
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    message(
                        "\(count) change\(count == 1 ? "" : "s") — one PR each, bottom targets \(stack.baseBookmark)."
                    )
                    Spacer()
                    if StackedPrNamer.isAvailable {
                        Button { Task { await generateNames() } } label: {
                            HStack(spacing: 4) {
                                if aiNaming {
                                    ProgressView().controlSize(.mini)
                                } else {
                                    Image(systemName: "sparkles")
                                }
                                Text(aiNaming ? "Generating…" : "Generate bookmarks").jayjayFont(11)
                            }
                        }
                        .controlSize(.small)
                        .disabled(aiNaming || isWorking)
                        .help("Suggest branch names with Apple Intelligence")
                    }
                }
                // While submitting, freeze the stack — no name edits or regenerate.
                layerList(stack).disabled(isWorking)
                if let errorMessage { errorBanner(errorMessage) }
                actionRow(confirm: "Submit", disabled: !allNamesValid(stack)) { submit() }
            }
        } else if let errorMessage {
            message(errorMessage)
            closeRow
        } else {
            HStack { Spacer()
                ProgressView()
                Spacer()
            }.frame(height: 72)
        }
    }

    private func layerList(_ stack: Stack) -> some View {
        ScrollView {
            // Show top of the stack first (layers are bottom→top).
            VStack(spacing: 6) {
                ForEach(Array(stack.layers.enumerated()).reversed(), id: \.element.changeId) { index, layer in
                    layerRow(stack, index, layer)
                }
            }
        }
        .frame(maxHeight: 280)
    }

    private func layerRow(_ stack: Stack, _ index: Int, _ layer: StackLayer) -> some View {
        rowCard {
            Image(systemName: "arrow.triangle.branch").jayjayFont(11).foregroundStyle(.tertiary)
            VStack(alignment: .leading, spacing: 4) {
                if !layer.title.isEmpty {
                    Text(layer.title).jayjayFont(12, weight: .semibold).lineLimit(1)
                }
                bookmarkField(layer)
                HStack(spacing: 5) {
                    Image(systemName: "arrow.right").jayjayFont(8).foregroundStyle(.tertiary)
                    Text(displayedBase(stack, index)).jayjayFont(10, design: .monospaced).foregroundStyle(.tertiary)
                    if !layer.bookmarkExisted { newBadge }
                }
            }
            Spacer()
        }
    }

    /// The branch name: read-only text with an inline pencil, swapping to a focused
    /// text field while editing.
    @ViewBuilder
    private func bookmarkField(_ layer: StackLayer) -> some View {
        if editingChangeId == layer.changeId {
            HStack(spacing: 5) {
                TextField("branch name", text: Binding(
                    get: { editedNames[layer.changeId] ?? layer.bookmark },
                    set: { editedNames[layer.changeId] = $0 }
                ))
                .textFieldStyle(.roundedBorder)
                .jayjayFont(11, design: .monospaced)
                .lineLimit(1)
                .focused($focusedChangeId, equals: layer.changeId)
                .onSubmit { editingChangeId = nil }
                validityWarning(layer)
                Button { editingChangeId = nil } label: {
                    Image(systemName: "checkmark.circle.fill").jayjayFont(12)
                }
                .buttonStyle(.plain).foregroundStyle(.green)
                .help("Done")
            }
        } else {
            HStack(spacing: 5) {
                Text(name(for: layer)).jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary).lineLimit(1)
                validityWarning(layer)
                Button {
                    editingChangeId = layer.changeId
                    focusedChangeId = layer.changeId
                } label: {
                    Image(systemName: "pencil").jayjayFont(10)
                }
                .buttonStyle(.plain).foregroundStyle(.tertiary)
                .help("Edit branch name")
            }
        }
    }

    private func name(for layer: StackLayer) -> String {
        editedNames[layer.changeId] ?? layer.bookmark
    }

    private func trimmedName(for layer: StackLayer) -> String {
        name(for: layer).trimmingCharacters(in: .whitespaces)
    }

    private func nameInvalid(_ layer: StackLayer) -> Bool {
        !isValidBookmarkName(name: trimmedName(for: layer))
    }

    private func allNamesValid(_ stack: Stack) -> Bool {
        stack.layers.allSatisfy { isValidBookmarkName(name: trimmedName(for: $0)) }
    }

    @ViewBuilder
    private func validityWarning(_ layer: StackLayer) -> some View {
        if nameInvalid(layer) {
            Image(systemName: "exclamationmark.triangle.fill").jayjayFont(9)
                .foregroundStyle(.orange).help("Not a valid branch name")
        }
    }

    /// The base shown for a layer, recomputed live from the (possibly edited)
    /// name of the layer below so the preview stays accurate as you type.
    private func displayedBase(_ stack: Stack, _ index: Int) -> String {
        index == 0 ? stack.baseBookmark : name(for: stack.layers[index - 1])
    }

    // MARK: - Results

    private func resultsBody(_ result: StackedPrResult) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            ScrollView {
                VStack(spacing: 6) {
                    ForEach(result.layers.reversed(), id: \.bookmark) { layer in
                        resultRow(layer)
                    }
                }
            }
            .frame(maxHeight: 260)
            HStack {
                Spacer()
                Button("Done") {
                    openSubmittedPrs(result)
                    onDismiss()
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                Spacer()
            }
        }
    }

    /// Open every created/updated PR's web page in the browser.
    private func openSubmittedPrs(_ result: StackedPrResult) {
        for layer in result.layers where !layer.prUrl.isEmpty {
            if let url = URL(string: layer.prUrl) {
                NSWorkspace.shared.open(url)
            }
        }
    }

    private func resultRow(_ layer: SubmittedLayer) -> some View {
        rowCard {
            Image(systemName: outcomeIcon(layer.outcome)).foregroundStyle(outcomeColor(layer.outcome))
            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 6) {
                    Text(layer.title.isEmpty ? layer.bookmark : layer.title)
                        .jayjayFont(12, weight: .medium).lineLimit(1)
                    if layer.prNumber > 0, let url = URL(string: layer.prUrl) {
                        Link("#\(layer.prNumber)", destination: url).jayjayFont(11, weight: .semibold)
                    }
                }
                Text(layer.detail).jayjayFont(10, design: .monospaced)
                    .foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
        }
    }

    // MARK: - Shared pieces

    private func rowCard(@ViewBuilder _ content: () -> some View) -> some View {
        HStack(spacing: 10) { content() }
            .padding(.horizontal, 10).padding(.vertical, 7)
            .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    private var newBadge: some View {
        Text("new").jayjayFont(9, weight: .semibold)
            .padding(.horizontal, 4).padding(.vertical, 1)
            .background(.green.opacity(0.18), in: Capsule())
            .foregroundStyle(.green)
    }

    private func message(_ text: String) -> some View {
        Text(text).jayjayFont(12).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
    }

    private func errorBanner(_ text: String) -> some View {
        Text(text).jayjayFont(11).foregroundStyle(.red)
            .padding(8).background(.red.opacity(0.1), in: RoundedRectangle(cornerRadius: 6))
    }

    private var closeRow: some View {
        HStack { Spacer()
            Button("Close") { onDismiss() }.keyboardShortcut(.cancelAction)
            Spacer()
        }
    }

    private func actionRow(confirm: String, disabled: Bool = false, action: @escaping () -> Void) -> some View {
        HStack(spacing: 12) {
            Spacer()
            Button("Cancel") { onDismiss() }.keyboardShortcut(.cancelAction).disabled(isWorking)
            Button(confirm, action: action)
                .keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent)
                .disabled(isWorking || disabled)
            Spacer()
        }
    }

    private func outcomeIcon(_ outcome: StackLayerOutcome) -> String {
        switch outcome {
            case .created: "plus.circle.fill"
            case .updated: "arrow.triangle.2.circlepath.circle.fill"
            case .failed: "xmark.octagon.fill"
        }
    }

    private func outcomeColor(_ outcome: StackLayerOutcome) -> Color {
        switch outcome {
            case .created: .green
            case .updated: .blue
            case .failed: .red
        }
    }

    // MARK: - Actions

    private func loadStack() async {
        guard stack == nil, results == nil else { return }
        viewModel.load {
            try $0.detectStack(baseRev: "trunk()", tipRev: tipRev)
        } onSuccess: { _, detected in
            stack = detected
            errorMessage = nil
        } onFailure: { _, error in
            errorMessage = error.friendlyDescription
        }
    }

    /// Replace each auto-named layer's branch name with an on-device suggestion.
    /// User-initiated (the "Generate names" button) so the preview never shifts on
    /// its own. Existing bookmarks are left untouched.
    @MainActor
    private func generateNames() async {
        guard let stack else { return }
        aiNaming = true
        defer { aiNaming = false }
        for layer in stack.layers where !layer.bookmarkExisted {
            let description = [layer.title, layer.body].filter { !$0.isEmpty }.joined(separator: "\n")
            guard let ai = await StackedPrNamer.branchName(from: description) else { continue }
            editedNames[layer.changeId] = "\(ai)-\(layer.changeIdShort)"
        }
    }

    private func submit() {
        guard let stack else { return }
        editingChangeId = nil // collapse any open inline editor before freezing
        isWorking = true
        errorMessage = nil
        // Capture Sendable primitives; rebuild the records inside the operation.
        let payload = stack.layers.map { ($0.changeId, trimmedName(for: $0), $0.title, $0.body) }
        viewModel.load { repo in
            let layers = payload.map {
                SubmitStackLayer(changeId: $0.0, bookmark: $0.1, title: $0.2, body: $0.3)
            }
            return try repo.submitStack(layers: layers)
        } onSuccess: { vm, result in
            results = result
            isWorking = false
            vm.refresh(selecting: nil)
        } onFailure: { _, error in errorMessage = error.friendlyDescription
            isWorking = false
        }
    }
}
