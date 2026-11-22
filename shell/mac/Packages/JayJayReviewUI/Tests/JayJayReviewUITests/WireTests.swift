import Foundation
import JayJayCore
@testable import JayJayReviewUI
import Testing

struct WireTests {
    @Test func aResultDecodesWithOlderBackendsGapsFilled() throws {
        let json = """
        {"session":3,"file":0,"overview":{"title":"t","description":"","author":"a","files":[
          {"path":"src/a.rs","kind":"renamed","oldPath":"src/old.rs","added":1,"removed":0,"review":"changed_since_review","noteCount":1},
          {"path":"b.bin","added":0,"removed":0,"review":"reviewed","notice":"Binary"}],
         "notes":[{"note":{"id":"n1","change_id":"c","path":"src/a.rs","identity":"i","side":"new","line":4,"anchor_excerpt":"let x = 1;","ignore_whitespace":false,"body":"why?","created_at_ms":1,"updated_at_ms":1,"resolved":false},"status":"orphaned"}]},
         "diff":{"rows":[{"fullWidth":false,"cells":[{"oldLine":"ours","newLine":null,"style":"removed","spans":[{"text":"x","style":"removed","token":"Keyword"}],"noEofNewline":true}]}]},
         "groupStates":[],"options":{"split":false,"ignoreWhitespace":true,"showAll":false},"elapsed":1}
        """
        let result = try JSONDecoder().decode(ReviewResult.self, from: Data(json.utf8))
        let renamed = result.overview.files[0]
        #expect(renamed.kind == .renamed && renamed.sourcePath == "src/old.rs" && renamed.review == .changedSinceReview)
        let binary = result.overview.files[1]
        #expect(binary.kind == .modified && binary.noteCount == 0 && binary.notice == "Binary")
        let note = try #require(result.overview.notes.first)
        #expect(note.note.excerpt == "let x = 1;" && note.note.side == .new && note.stale)
        #expect(result.diff?.rows[0].cells[0].oldLine == .label("ours"))
        #expect(result.options.ignoreWhitespace)

        let bare = #"{"title":"","description":"","author":"","files":[]}"#
        #expect(try JSONDecoder().decode(ReviewOverview.self, from: Data(bare.utf8)).notes.isEmpty)
    }

    @Test func everyTokenTheReviewSendsKeepsItsKind() throws {
        let names = ["Plain", "Keyword", "StringLit", "Comment", "Number", "Type", "Function", "Variable", "Operator", "Punctuation", "Attribute", "Later"]
        let json = "[" + names.map { #"{"text":"","token":"\#($0)"}"# }.joined(separator: ",") + "]"
        let tokens = try JSONDecoder().decode([RenderedDiff.Span].self, from: Data(json.utf8)).map(\.token)
        #expect(tokens == [.plain, .keyword, .stringLit, .comment, .number, .type, .function, .variable, .operator, .punctuation, .attribute, nil])
    }

    @Test func actionsEncodeAsReviewWebSpellsThem() throws {
        func object(_ action: ReviewAction) throws -> NSDictionary {
            try #require(JSONSerialization.jsonObject(with: JSONEncoder().encode(action)) as? NSDictionary)
        }
        #expect(try object(.setFiles([1, 2], reviewed: true)) == ["kind": "setFiles", "files": [1, 2], "reviewed": true])
        #expect(
            try object(.addNote(file: 0, side: .old, line: 7, body: "b", ignoreWhitespace: false))
                == ["kind": "addNote", "file": 0, "side": "old", "line": 7, "body": "b", "ignoreWhitespace": false]
        )
        #expect(try object(.resolveNote(id: "n")) == ["kind": "resolveNote", "id": "n"])
    }
}
