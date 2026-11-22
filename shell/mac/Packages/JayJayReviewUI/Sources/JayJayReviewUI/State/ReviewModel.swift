import Observation

/// A reply is applied only while it is the latest for what it answers, so a slow render never replaces a newer selection and an older save never undoes a newer one.
@MainActor @Observable public final class ReviewModel {
    public private(set) var overview: ReviewOverview?
    public private(set) var session: Int?
    public private(set) var path: String?
    public var anchor: ReaderLine.ID?
    public private(set) var all = false
    public private(set) var lines: [ReaderLine] = []
    public private(set) var placeholder: ReaderPlaceholder?
    public private(set) var loading = false
    public private(set) var rendering = false
    public private(set) var pending: Set<String> = []
    public var failed: (any Error) -> Void = { _ in }

    private var options = DiffOptions()
    private var generation = 0
    private var renderGeneration = 0
    private var markGeneration = 0
    private var noteSaves = 0
    @ObservationIgnored private var lastNoteSave: Task<Bool, Never>?
    @ObservationIgnored private var positions: [ReaderLine.ID: Int] = [:]

    public init() {}

    public var file: ReviewFileSummary? {
        overview?.files.first { $0.path == path }
    }

    public var notes: [ReviewNote] {
        (overview?.notes ?? []).filter { $0.note.path == path }
    }

    public func reset() {
        generation += 1
        renderGeneration += 1
        overview = nil
        session = nil
        path = nil
        clearReader()
        loading = false
        rendering = false
        pending = []
        noteSaves = 0
        lastNoteSave = nil
    }

    public func open(_ backend: some ReviewBackend) async {
        // A reply to a save still in flight would be dropped, and this open may have read the store before it.
        guard pending.isEmpty, noteSaves == 0 else { return }
        generation += 1
        let ticket = generation
        loading = true
        defer {
            if generation == ticket {
                loading = false
            }
        }
        do {
            let opened = try await backend.open()
            guard generation == ticket, !Task.isCancelled else { return }
            overview = opened.overview
            let revised = opened.session != session
            session = opened.session
            options = opened.options
            // The reader draws one column, so a backend's split default is asked again as unified.
            options.split = false
            let selected = opened.overview.files.firstIndex { $0.path == path } ?? 0
            let next = opened.overview.files.indices.contains(selected) ? opened.overview.files[selected].path : nil
            if next != path {
                clearReader()
            } else if revised {
                // Another revision's lines must not take notes while this one's are still to come.
                lines = []
                positions = [:]
                placeholder = nil
            }
            path = next
            if selected == opened.file, !all, !opened.options.split, opened.diff != nil || next == nil {
                // A render from before this open carries an older overview.
                renderGeneration += 1
                rendering = false
                install(opened)
            } else {
                await render(backend, expand: all)
            }
        } catch {
            if generation == ticket {
                report(error)
            }
        }
    }

    public func select(_ file: ReviewFileSummary, backend: some ReviewBackend) async {
        guard file.path != path else { return }
        path = file.path
        clearReader()
        await render(backend, expand: false)
    }

    public func render(_ backend: some ReviewBackend, expand: Bool) async {
        guard let session, let index = overview?.files.firstIndex(where: { $0.path == path }) else { return }
        renderGeneration += 1
        let ticket = renderGeneration
        let marksAtStart = markGeneration
        rendering = true
        defer {
            if renderGeneration == ticket {
                rendering = false
            }
        }
        var wanted = options
        wanted.showAll = expand
        do {
            let result = try await backend.render(ReviewSelection(session: session, file: index, options: wanted))
            guard renderGeneration == ticket, self.session == session, !Task.isCancelled else { return }
            // A render reads the store too; a save of ours that overlapped it has the newer reply.
            if pending.isEmpty, marksAtStart == markGeneration {
                overview = result.overview
            }
            install(result)
        } catch {
            if renderGeneration == ticket, self.session == session {
                report(error)
            }
        }
    }

    public func toggle(_ path: String, backend: some ReviewBackend) async {
        guard !loading, let session, let files = overview?.files, let index = files.firstIndex(where: { $0.path == path }),
              pending.insert(path).inserted
        else { return }
        let owner = generation
        markGeneration += 1
        defer {
            markGeneration += 1
            if generation == owner {
                pending.remove(path)
            }
        }
        let selection = ReviewSelection(session: session, file: index, options: renderedOptions)
        do {
            let result = try await backend.mutate(selection, .setFiles([index], reviewed: files[index].review != .reviewed))
            guard self.session == session, generation == owner, result.overview.files.indices.contains(index) else { return }
            // Only this file: a concurrent toggle's older overview may arrive last.
            overview?.files[index].review = result.overview.files[index].review
        } catch {
            if generation == owner {
                report(error)
            }
        }
    }

    public enum NoteEdit: Sendable {
        case add(side: NoteSide, line: Int, body: String)
        case resolve(id: String)
    }

    /// Returns whether the backend took it.
    @discardableResult
    public func note(_ edit: NoteEdit, backend: some ReviewBackend) async -> Bool {
        guard !loading, let session, let index = overview?.files.firstIndex(where: { $0.path == path }) else { return false }
        let selection = ReviewSelection(session: session, file: index, options: renderedOptions)
        let action: ReviewAction = switch edit {
            case let .add(side, line, body):
                .addNote(file: index, side: side, line: line, body: body, ignoreWhitespace: options.ignoreWhitespace)
            case let .resolve(id): .resolveNote(id: id)
        }
        let owner = generation
        // One at a time: every reply carries all the notes, so an older one arriving last would drop the newer edit.
        let previous = lastNoteSave
        let save = Task {
            _ = await previous?.value
            guard self.generation == owner else { return false }
            return await self.save(action, at: selection, backend: backend)
        }
        lastNoteSave = save
        noteSaves += 1
        defer {
            if generation == owner {
                noteSaves -= 1
            }
        }
        return await save.value
    }

    private func save(_ action: ReviewAction, at selection: ReviewSelection, backend: some ReviewBackend) async -> Bool {
        let owner = generation
        markGeneration += 1
        defer { markGeneration += 1 }
        do {
            let result = try await backend.mutate(selection, action)
            guard generation == owner else { return true }
            // Notes and their counts only: a concurrent mark's own reply owns the marks.
            overview?.notes = result.overview.notes
            for (i, file) in result.overview.files.enumerated() where overview?.files.indices.contains(i) == true {
                overview?.files[i].noteCount = file.noteCount
            }
            return true
        } catch {
            if generation == owner {
                report(error)
            }
            return false
        }
    }

    func topmost(of ids: [ReaderLine.ID]) -> ReaderLine.ID? {
        ids.min { positions[$0, default: .max] < positions[$1, default: .max] }
    }

    private var renderedOptions: DiffOptions {
        var current = options
        current.showAll = all
        return current
    }

    private func install(_ result: ReviewResult) {
        options = result.options
        all = result.options.showAll
        let display = ReaderLine.display(file: file, diff: result.diff)
        lines = display.lines
        positions = Dictionary(lines.enumerated().map { ($1.id, $0) }) { first, _ in first }
        placeholder = display.placeholder
        anchor = ReaderLine.readingAnchor(saved: anchor, in: lines)
    }

    private func clearReader() {
        anchor = nil
        all = false
        lines = []
        positions = [:]
        placeholder = nil
    }

    private func report(_ error: any Error) {
        guard !(error is CancellationError), !Task.isCancelled else { return }
        failed(error)
    }
}
