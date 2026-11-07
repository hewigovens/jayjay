import JayJayCore
import SwiftUI

struct ConvergeContentsCard: View {
    let review: ConvergeReview
    let repo: JayJayRepo
    let diffStore: DiffStore
    @State private var target: String?
    @State private var hunks: [DiffHunk]?
    @State private var error: String?
    @State private var selectedPath: String?
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.appThemes) private var appThemes

    private var from: String? {
        review.versions.first?.commitId
    }

    private var to: String? {
        target ?? review.versions.dropFirst().first?.commitId
    }

    var body: some View {
        ConvergeChoiceCard(item: .contents, review: review, note: note) {
            if review.versions.count > 2 {
                Picker(Localized.Converge.compareWith, selection: Binding(get: { to ?? "" }, set: { target = $0 })) {
                    ForEach(review.versions.dropFirst(), id: \.commitId) { version in
                        Text("\(review.letter(of: from ?? "")) → \(review.letter(of: version.commitId))").tag(version.commitId)
                    }
                }
                .labelsHidden()
                .fixedSize()
            }
        } content: {
            VStack(alignment: .leading, spacing: 0) {
                files
                conflicts
            }
        }
        .task(id: "\(from ?? "")|\(to ?? "")") { await load() }
    }

    private var note: String? {
        guard let hunks, !hunks.isEmpty else { return hunks == nil ? nil : Localized.Converge.same(count: review.versions.count) }
        let files = Localized.Converge.changedFiles(count: hunks.count)
        guard !review.isChosen(.contents), let source = review.automaticSource(.contents) else { return files }
        return "\(files) · \(Localized.Converge.resultKeeps(version: review.letter(of: source)))"
    }

    @ViewBuilder
    private var files: some View {
        if let error {
            Text(error).jayjayFont(12).foregroundStyle(.red).textSelection(.enabled).padding(10)
        } else if let hunks {
            ForEach(hunks, id: \.path) { hunk in
                fileRow(hunk)
            }
            if let hunk = hunks.first(where: { $0.path == selectedPath }), let from, let to {
                Divider()
                DiffSection(
                    hunk: hunk,
                    rev: to,
                    repo: repo,
                    actions: nil,
                    isWorkingCopy: false,
                    diffStore: diffStore,
                    reviewStore: nil,
                    noteEditor: .constant(nil),
                    compareFromRev: from,
                    fitsContent: true
                )
                .id("\(from)|\(to)|\(hunk.path)")
            }
        } else {
            Text(Localized.Converge.loading).jayjayFont(12).foregroundStyle(.secondary).padding(10)
        }
    }

    @ViewBuilder
    private var conflicts: some View {
        if let paths = review.expectedConflicts, !paths.isEmpty {
            Divider()
            VStack(alignment: .leading, spacing: 4) {
                Text(paths.joined(separator: "\n")).jayjayFont(11, design: .monospaced).textSelection(.enabled)
                Text(Localized.Converge.conflictsFollowup).jayjayFont(11).foregroundStyle(.secondary)
            }
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(.orange.opacity(0.1))
        }
    }

    private func fileRow(_ hunk: DiffHunk) -> some View {
        let selected = hunk.path == selectedPath
        return Button {
            selectedPath = selected ? nil : hunk.path
        } label: {
            HStack(spacing: 6) {
                Image(systemName: "chevron.right")
                    .jayjayFont(9, weight: .semibold)
                    .foregroundStyle(.secondary)
                    .rotationEffect(.degrees(selected ? 90 : 0))
                hunk.hunkType.icon
                    .jayjayFont(11)
                    .foregroundStyle(appThemes.colors(for: colorScheme).fileColor(for: hunk.hunkType))
                Text(hunk.path)
                    .jayjayFont(11, design: .monospaced)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(AID.Converge.changedFile(hunk.path))
    }

    private func load() async {
        guard let from, let to else { return }
        hunks = nil
        error = nil
        let repo = repo
        let result = await Task.detached { Result { try repo.interdiffSummary(fromRev: from, toRev: to).diff } }.value
        switch result {
            case let .success(diff):
                hunks = diff
                if !diff.contains(where: { $0.path == selectedPath }) {
                    selectedPath = nil
                }
            case let .failure(failure):
                error = failure.friendlyDescription
        }
    }
}
