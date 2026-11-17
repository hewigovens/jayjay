import JayJayCore

extension ReviewMode {
    var showsMarks: Bool {
        self == .workingCopy || self == .on
    }

    var isSwitchable: Bool {
        self == .on || self == .off
    }
}
