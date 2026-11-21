@testable import JayJayReviewUI
import Testing

struct Failure: Error {}

private actor FixtureBackend: ReviewBackend {
    var files = [ReviewFileSummary(path: "first", added: 1, removed: 0), ReviewFileSummary(path: "second", added: 1, removed: 0)]
    var notes: [ReviewNote] = []
    var failing = false
    var lazy = false
    var split = false
    var session = 7
    var failingRenders = false
    private var pauseNext = false
    private var holdNext = false
    private var gate: CheckedContinuation<Void, Never>?
    private var waiter: CheckedContinuation<Void, Never>?

    func pauseNextRender() {
        pauseNext = true
    }

    func awaitPaused() async {
        if gate == nil {
            await withCheckedContinuation { waiter = $0 }
        }
    }

    func holdNextSave() {
        holdNext = true
    }

    func resume() {
        gate?.resume()
        gate = nil
    }

    func failSaves() {
        failing = true
    }

    func openLazily() {
        lazy = true
    }

    func openSplit() {
        split = true
    }

    func moveToARevisionThatCannotRender() {
        session += 1
        lazy = true
        failingRenders = true
    }

    func markReviewed(_ index: Int) {
        files[index].review = .reviewed
    }

    func open() -> ReviewResult {
        var opened = result(session: session, file: 0, options: DiffOptions(split: split))
        if lazy {
            opened.diff = nil
        }
        return opened
    }

    func render(_ selection: ReviewSelection) async throws -> ReviewResult {
        if failingRenders {
            throw Failure()
        }
        let reply = result(session: selection.session, file: selection.file, options: selection.options)
        if pauseNext {
            pauseNext = false
            await pause()
        }
        return reply
    }

    private func pause() async {
        await withCheckedContinuation { gate = $0
            waiter?.resume()
            waiter = nil
        }
    }

    func mutate(_ selection: ReviewSelection, _ action: ReviewAction) async throws -> ReviewResult {
        if failing {
            throw Failure()
        }
        switch action {
            case let .setFiles(indices, reviewed):
                for i in indices {
                    files[i].review = reviewed ? .reviewed : .unreviewed
                }
            case let .addNote(file, side, line, body, _):
                notes.append(ReviewNote(note: .init(id: "n" + String(notes.count), path: files[file].path, side: side, line: line, body: body), status: .current))
            case let .resolveNote(id):
                if let i = notes.firstIndex(where: { $0.id == id }) {
                    notes[i].note.resolved = true
                    notes[i].status = .resolved
                }
        }
        for i in files.indices {
            files[i].noteCount = notes.filter { $0.note.path == files[i].path && !$0.note.resolved }.count
        }
        let reply = result(session: selection.session, file: selection.file, options: selection.options)
        if holdNext {
            holdNext = false
            await pause()
        }
        return reply
    }

    private func result(session: Int, file: Int, options: DiffOptions) -> ReviewResult {
        let cell = RenderedDiff.Cell(newLine: 1, style: .added, spans: [.init(text: files[file].path)])
        let cells = options.split ? [RenderedDiff.Cell(style: .other, spans: []), cell] : [cell]
        return ReviewResult(
            session: session, file: file, overview: ReviewOverview(title: "", description: "", author: "", files: files, notes: notes),
            diff: RenderedDiff(rows: [.init(cells: cells)]), options: options
        )
    }
}

@MainActor struct ReviewModelTests {
    @Test func aLateRenderCannotReplaceANewerSelection() async throws {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        let files = try #require(model.overview?.files)
        await backend.pauseNextRender()
        let slow = Task { await model.select(files[1], backend: backend) }
        await backend.awaitPaused()
        await model.select(files[0], backend: backend)
        await backend.resume()
        await slow.value
        #expect(model.path == "first")
        #expect(model.lines.first?.cell.text == "first")
    }

    @Test func markingAnotherFileKeepsTheReaderAndAFailedSaveKeepsTheMark() async throws {
        let backend = FixtureBackend(), model = ReviewModel()
        var failures = 0
        model.failed = { _ in failures += 1 }
        await model.open(backend)
        try await model.select(#require(model.overview?.files.last), backend: backend)
        await model.render(backend, expand: true)
        await model.toggle("first", backend: backend)
        #expect(model.overview?.files.first?.review == .reviewed)
        #expect(model.path == "second" && model.all)
        await model.open(backend)
        #expect(model.path == "second" && model.all)

        await backend.failSaves()
        await model.toggle("first", backend: backend)
        #expect(model.overview?.files.first?.review == .reviewed)
        #expect(failures == 1 && model.pending.isEmpty)
    }

    @Test func aNoteShowsOnItsFileAndResolvingKeepsItAsARecord() async throws {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        let added = await model.note(.add(side: .new, line: 1, body: "why?"), backend: backend)
        #expect(added)
        #expect(model.notes.map(\.note.body) == ["why?"])
        #expect(model.overview?.files.first?.noteCount == 1)
        let id = try #require(model.notes.first?.id)
        await model.note(.resolve(id: id), backend: backend)
        #expect(model.notes.first?.note.resolved == true)
        #expect(model.overview?.files.first?.noteCount == 0)
        try await model.select(#require(model.overview?.files.last), backend: backend)
        #expect(model.notes.isEmpty)
    }

    @Test func aRefreshWaitsForANoteBeingSaved() async {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        await backend.holdNextSave()
        let save = Task { await model.note(.add(side: .new, line: 1, body: "why?"), backend: backend) }
        await backend.awaitPaused()
        await model.open(backend)
        await backend.resume()
        #expect(await save.value)
        #expect(model.notes.map(\.note.body) == ["why?"])
    }

    @Test func aSlowNoteReplyCannotDropANewerNote() async throws {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        await backend.holdNextSave()
        let first = Task { await model.note(.add(side: .new, line: 1, body: "first"), backend: backend) }
        await backend.awaitPaused()
        let second = Task { await model.note(.add(side: .new, line: 1, body: "second"), backend: backend) }
        try await Task.sleep(for: .milliseconds(50))
        await backend.resume()
        #expect(await first.value)
        #expect(await second.value)
        #expect(model.notes.map(\.note.body) == ["first", "second"])
    }

    @Test func aQueuedNoteStaysOnTheFileItWasWrittenFor() async throws {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        await backend.holdNextSave()
        let first = Task { await model.note(.add(side: .new, line: 1, body: "first"), backend: backend) }
        await backend.awaitPaused()
        let second = Task { await model.note(.add(side: .new, line: 1, body: "second"), backend: backend) }
        try await Task.sleep(for: .milliseconds(50))
        try await model.select(#require(model.overview?.files.last), backend: backend)
        await backend.resume()
        #expect(await first.value)
        #expect(await second.value)
        #expect(model.overview?.notes.map(\.note.path) == ["first", "first"])
    }

    @Test func aRenderFromBeforeARefreshCannotUndoIt() async {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        await backend.pauseNextRender()
        let slow = Task { await model.render(backend, expand: false) }
        await backend.awaitPaused()
        await backend.markReviewed(0)
        await model.open(backend)
        await backend.resume()
        await slow.value
        #expect(model.overview?.files.first?.review == .reviewed)
        #expect(!model.rendering)
    }

    @Test func aFileOpenedWithoutItsDiffIsRendered() async {
        let backend = FixtureBackend(), model = ReviewModel()
        await backend.openLazily()
        await model.open(backend)
        #expect(model.lines.first?.cell.text == "first")
    }

    @Test func aSplitDiffIsRenderedAgainAsUnified() async {
        let backend = FixtureBackend(), model = ReviewModel()
        await backend.openSplit()
        await model.open(backend)
        #expect(model.lines.map(\.cell.text) == ["first"])
    }

    @Test func anotherReviewOpensWhileTheLastOnesNoteIsStillSaving() async {
        let backend = FixtureBackend(), model = ReviewModel()
        await model.open(backend)
        await backend.holdNextSave()
        let save = Task { await model.note(.add(side: .new, line: 1, body: "why?"), backend: backend) }
        await backend.awaitPaused()
        model.reset()
        let other = FixtureBackend()
        await model.open(other)
        #expect(await model.note(.add(side: .new, line: 1, body: "here"), backend: other))
        await backend.resume()
        _ = await save.value
        await model.open(FixtureBackend())
        #expect(model.notes.isEmpty && !model.loading)
    }

    @Test func aRevisionThatFailsToRenderDoesNotKeepTheLastOnesLines() async {
        let backend = FixtureBackend(), model = ReviewModel()
        model.failed = { _ in }
        await model.open(backend)
        await backend.moveToARevisionThatCannotRender()
        await model.open(backend)
        #expect(model.session == 8 && model.lines.isEmpty)
    }
}
