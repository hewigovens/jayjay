/// review-web's `ReviewResult` (`web/review/src/model/review.ts`), without what the native reader doesn't draw.
public struct ReviewResult: Decodable, Sendable {
    public var session: Int
    public var file: Int
    public var overview: ReviewOverview
    public var diff: RenderedDiff?
    public var options: DiffOptions

    public init(session: Int, file: Int, overview: ReviewOverview, diff: RenderedDiff?, options: DiffOptions) {
        self.session = session
        self.file = file
        self.overview = overview
        self.diff = diff
        self.options = options
    }
}
