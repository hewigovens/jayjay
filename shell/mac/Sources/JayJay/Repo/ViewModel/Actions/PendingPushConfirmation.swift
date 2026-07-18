enum PendingPushConfirmation {
    static func remainingBookmark(
        afterConfirming bookmark: String?,
        startPush: (String) -> Bool
    ) -> String? {
        guard let bookmark else { return nil }
        return startPush(bookmark) ? nil : bookmark
    }
}
