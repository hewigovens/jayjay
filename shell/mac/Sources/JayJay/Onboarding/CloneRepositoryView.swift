import AppKit
import SwiftUI

struct CloneRepositoryView: View {
    let onCloned: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var model: CloneRepositoryModel
    @FocusState private var sourceFocused: Bool

    init(recentRepositories: [String], onCloned: @escaping (String) -> Void) {
        self.onCloned = onCloned
        _model = State(initialValue: CloneRepositoryModel(recentRepositories: recentRepositories))
    }

    var body: some View {
        @Bindable var model = model
        VStack(alignment: .leading, spacing: 14) {
            Form {
                TextField("Repository URL", text: $model.source, prompt: Text("https://github.com/owner/repo.git"))
                    .focused($sourceFocused)
                    .accessibilityIdentifier(AID.CloneRepository.sourceField)
                TextField("Name", text: $model.name)
                    .accessibilityIdentifier(AID.CloneRepository.nameField)
                LabeledContent("Location") {
                    HStack(spacing: 6) {
                        TextField("Location", text: $model.location)
                            .labelsHidden()
                            .accessibilityIdentifier(AID.CloneRepository.locationField)
                        Button("Choose…", action: chooseLocation)
                    }
                }
                status
            }
            .formStyle(.columns)
            .textFieldStyle(.roundedBorder)
            .jayjayFont(12)
            .disabled(model.isCloning)

            HStack {
                Spacer()
                Button("Cancel", action: cancel)
                    .keyboardShortcut(.cancelAction)
                Button("Clone") { model.clone(onCloned: finish) }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(!model.canClone)
                    .accessibilityIdentifier(AID.CloneRepository.cloneButton)
            }
        }
        .padding(20)
        .frame(width: 480)
        .onAppear { sourceFocused = true }
        .onDisappear { model.cancel() }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.willTerminateNotification)) { _ in
            model.cancelForTermination()
        }
    }

    @ViewBuilder
    private var status: some View {
        if model.isCloning {
            HStack(spacing: 6) {
                ProgressView().controlSize(.small)
                Text("Cloning \(model.name)…")
                    .foregroundStyle(.secondary)
            }
            .jayjayFont(11)
        } else if let errorMessage = model.errorMessage {
            Label(errorMessage, systemImage: "exclamationmark.triangle")
                .jayjayFont(11)
                .foregroundStyle(.red)
                .textSelection(.enabled)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier(AID.CloneRepository.error)
        }
    }

    private func chooseLocation() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Choose"
        panel.directoryURL = URL(fileURLWithPath: (model.folder as NSString).expandingTildeInPath)
        if panel.runModal() == .OK, let url = panel.url {
            model.choose(folder: url.path)
        }
    }

    /// Escape stops a running clone first, so a second press is needed to close the window.
    private func cancel() {
        if model.isCloning {
            model.cancel()
        } else {
            dismiss()
        }
    }

    private func finish(_ path: String) {
        dismiss()
        onCloned(path)
    }
}
