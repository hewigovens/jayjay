import JayJayCore

extension ReviewMode {
    static let symbolName = "text.badge.checkmark"

    var showsMarks: Bool {
        self == .workingCopy || self == .on
    }

    var isSwitchable: Bool {
        self == .on || self == .off
    }
}
