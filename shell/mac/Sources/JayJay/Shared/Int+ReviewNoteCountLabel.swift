extension Int {
    /// Pluralized review-note count, e.g. "1 review note" / "3 review notes".
    var reviewNoteCountLabel: String {
        "\(self) review \(self == 1 ? "note" : "notes")"
    }
}
