public extension HunkType {
    /// Renames and copies carry their source path in `oldPath`.
    var hasSource: Bool {
        self == .renamed || self == .copied
    }
}
