struct DAGDragSession<State> {
    var state: State?
    var armTask: Task<Void, Never>?
    var previewTargetId: String?
    var previewTask: Task<Void, Never>?

    mutating func clearPreview() {
        previewTask?.cancel()
        previewTask = nil
        previewTargetId = nil
    }

    mutating func end() {
        armTask?.cancel()
        previewTask?.cancel()
        self = DAGDragSession()
    }
}
