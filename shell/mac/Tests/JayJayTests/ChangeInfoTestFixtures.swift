import JayJayCore

func mockChangeInfo(
    changeId: String = "c-1",
    commitId: String = "abc123",
    description: String = "change",
    parents: [String] = [],
    bookmarks: [String] = [],
    tags: [String] = [],
    workspaces: [String] = [],
    isWorkingCopy: Bool = false,
    hasConflict: Bool = false,
    isEmpty: Bool = false,
    isImmutable: Bool = false,
    isDivergent: Bool = false
) -> ChangeInfo {
    ChangeInfo(
        changeId: ShortId(id: changeId, shortLen: 1),
        commitId: ShortId(id: commitId, shortLen: 1),
        description: description,
        author: .tester,
        parents: parents,
        bookmarks: bookmarks,
        tags: tags,
        workspaces: workspaces,
        isWorkingCopy: isWorkingCopy,
        hasConflict: hasConflict,
        isEmpty: isEmpty,
        isImmutable: isImmutable,
        isDivergent: isDivergent,
        newChange: NewChangeEligibility(onTop: true, before: true, after: true)
    )
}
