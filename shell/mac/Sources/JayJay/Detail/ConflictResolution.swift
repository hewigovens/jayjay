import JayJayCore

enum ConflictResolution: Hashable {
    case mergeEditor(String)
    case inJayJay
    case editorTool(String)
    case openInEditor

    static func options(
        mergeEditor: String?,
        supportsConflictEditor: Bool,
        editor: AppSettings.ExternalEditor
    ) -> [Self] {
        var options = mergeEditor.map { [Self.mergeEditor($0)] } ?? []
        if supportsConflictEditor {
            options.append(.inJayJay)
        }
        options.append(editor.jjMergeTool.map(Self.editorTool) ?? .openInEditor)
        return options
    }

    func accessibilityIdentifier(path: String) -> String {
        self == .inJayJay ? AID.Conflict.resolveInJayJay(path) : AID.Conflict.resolveElsewhere(path)
    }

    func title(editor: AppSettings.ExternalEditor) -> String {
        switch self {
            case let .mergeEditor(name): Localized.Menu.resolveIn(app: name)
            case .inJayJay: Localized.Repo.editInJayjay
            case .editorTool: Localized.Menu.resolveIn(app: editor.title)
            case .openInEditor: Localized.Menu.openIn(app: editor.title)
        }
    }
}
