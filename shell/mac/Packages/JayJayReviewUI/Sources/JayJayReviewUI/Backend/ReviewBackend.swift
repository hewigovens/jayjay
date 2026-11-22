/// Each call returns the refreshed result once a write is durable, and throws to leave the viewer's previous result in place.
public protocol ReviewBackend: Sendable {
    func open() async throws -> ReviewResult
    func render(_ selection: ReviewSelection) async throws -> ReviewResult
    func mutate(_ selection: ReviewSelection, _ action: ReviewAction) async throws -> ReviewResult
}
