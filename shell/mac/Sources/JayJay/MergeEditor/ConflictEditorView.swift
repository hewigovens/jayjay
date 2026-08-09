import JayJayCore
import SwiftUI

struct ConflictEditorView: View {
    typealias Save = (ConflictEditorData, String, @escaping @MainActor (Bool) -> Void) -> Void

    @State private var session: ConflictEditorSession
    let onSave: Save
    let onDone: () -> Void

    init(session: ConflictEditorSession, onSave: @escaping Save, onDone: @escaping () -> Void) {
        _session = State(initialValue: session)
        self.onSave = onSave
        self.onDone = onDone
    }

    var body: some View {
        MergeEditorView(
            session: session,
            headerAccessibilityIdentifier: AID.Conflict.editorModal,
            onCancel: onDone,
            onSave: save
        )
    }

    private func save() {
        guard let data = session.data, data.isText, !session.isSaving else { return }
        session.isSaving = true
        onSave(data, session.result) { success in
            session.isSaving = false
            if success {
                onDone()
            }
        }
    }
}
