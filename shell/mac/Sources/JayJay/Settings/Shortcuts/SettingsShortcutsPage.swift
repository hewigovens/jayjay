import JayJayCore
import SwiftUI

struct SettingsShortcutsPage: View {
    let store: ShortcutStore
    @State private var query = ""
    @State private var modifiedOnly = false
    @State private var recorder: ShortcutRecorder

    init(store: ShortcutStore) {
        self.store = store
        _recorder = State(initialValue: ShortcutRecorder(store: store))
    }

    private var groups: [ShortcutSection] {
        ShortcutGuide.sections(bindings: store.bindings.filter {
            (!modifiedOnly || $0.modified) && (query.isEmpty || $0.title.localizedCaseInsensitiveContains(query)
                || $0.chords.flatMap { $0 }.joined().localizedCaseInsensitiveContains(query))
        })
    }

    var body: some View {
        VStack(spacing: 10) {
            HStack(spacing: 10) {
                HStack {
                    Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                    TextField(Localized.Shortcuts.searchActions, text: $query)
                        .textFieldStyle(.plain)
                        .accessibilityIdentifier(AID.Shortcuts.search)
                }
                .padding(7)
                .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 6))
                Picker("", selection: $modifiedOnly) {
                    Text(Localized.Shortcuts.all).tag(false)
                    Text(Localized.Shortcuts.modified).tag(true)
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(width: 180)
            }
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(groups) { group in
                        Text(group.title.uppercased())
                            .jayjayFont(11, weight: .bold)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(8)
                            .background(Color.primary.opacity(0.05))
                            .padding(.top, 8)
                        ForEach(group.entries) { binding in
                            ShortcutSettingsRow(binding: binding, recorder: recorder)
                        }
                    }
                }
                .padding(.trailing, 12)
            }
            if query.isEmpty {
                HStack {
                    Button(Localized.Shortcuts.resetAll) { recorder.apply(.resetAll, id: "") }
                        .accessibilityIdentifier(AID.Shortcuts.resetAll)
                    Spacer()
                }
            }
        }
        .padding(16)
        .onDisappear { recorder.cancel() }
    }
}
