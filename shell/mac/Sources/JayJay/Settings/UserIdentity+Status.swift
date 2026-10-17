import JayJayCore

extension UserIdentity {
    var label: String {
        "\(name) <\(email)>"
    }

    static func configured() async -> UserIdentity? {
        await Task.detached(priority: .userInitiated) { try? configuredUserIdentity() }.value
    }
}
