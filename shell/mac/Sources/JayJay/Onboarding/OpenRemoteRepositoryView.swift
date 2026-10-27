import JayJayCore
import SwiftUI

struct OpenRemoteRepositoryView: View {
    let onOpen: (String) -> Void
    @State private var model = OpenRemoteRepositoryModel()
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            LabeledContent(Localized.Ssh.machine) {
                HStack {
                    TextField(Localized.Ssh.hostPlaceholder, text: $model.machine)
                        .disabled(model.isBusy)
                        .accessibilityIdentifier(AID.OpenRemoteRepository.machine)
                        .onSubmit { Task { await model.browse() } }
                    Menu {
                        ForEach(model.machines, id: \.self) { machine in
                            Button(machine) { model.machine = machine }
                        }
                    } label: { Image(systemName: "server.rack") }
                        .fixedSize()
                        .help(Localized.Ssh.selectMachine)
                        .disabled(model.isBusy || model.machines.isEmpty)
                    if model.directory == nil {
                        Button(Localized.Ssh.connect) { Task { await model.browse() } }
                            .disabled(model.isBusy || model.host.isEmpty)
                            .accessibilityIdentifier(AID.OpenRemoteRepository.connect)
                    }
                }
            }
            if let directory = model.directory {
                LabeledContent(Localized.Ssh.folder) {
                    HStack {
                        TextField("~/workspace/project", text: $model.path)
                            .disabled(model.isBusy)
                            .accessibilityIdentifier(AID.OpenRemoteRepository.path)
                            .onChange(of: model.path) { model.selectedFolder = nil }
                            .onSubmit { Task { await model.browse() } }
                        Button(Localized.Ssh.parent, systemImage: "arrow.up") {
                            if let parent = directory.parent {
                                Task { await model.browse(parent) }
                            }
                        }
                        .labelStyle(.iconOnly)
                        .disabled(model.isBusy || directory.parent == nil)
                    }
                }
                List(directory.folders, id: \.path, selection: $model.selectedFolder) { folder in
                    Label(folder.name, systemImage: "folder").tag(folder.path)
                }
                .contextMenu(forSelectionType: String.self, menu: { _ in }, primaryAction: { paths in
                    if let path = paths.first {
                        Task { await model.browse(path) }
                    }
                })
                .frame(height: 210)
                .accessibilityIdentifier(AID.OpenRemoteRepository.folders)
            }
            if let error = model.error {
                Text(error).jayjayFont(12).foregroundStyle(.red).textSelection(.enabled)
            }
            HStack {
                if model.isBusy {
                    ProgressView().controlSize(.small)
                    Text(Localized.Ssh.connecting).jayjayFont(12).foregroundStyle(.secondary)
                }
                Spacer()
                Button(Localized.Common.cancel) { dismiss() }.keyboardShortcut(.cancelAction)
                Button(Localized.Common.open) {
                    Task {
                        if let path = await model.location() {
                            onOpen(path)
                            dismiss()
                        }
                    }
                }
                .keyboardShortcut(.defaultAction)
                .disabled(!model.canOpen)
                .accessibilityIdentifier(AID.OpenRemoteRepository.open)
            }
        }
        .textFieldStyle(.roundedBorder)
        .jayjayFont(12)
        .padding(22)
        .frame(width: 540)
        .task { await model.loadMachines() }
        .onDisappear { model.invalidate() }
    }
}
