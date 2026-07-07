@testable import JayJay
import JayJayCore

func testProjection(
    pluginId: String = "sarif",
    pluginLabel: String = "SARIF",
    mode: DiffProjectionMode = .raw,
    renderKind: DiffRenderKind = .markdown,
    virtualPath: String = "results.sarif.md"
) -> DiffProjection {
    DiffProjection(
        pluginId: pluginId,
        pluginLabel: pluginLabel,
        pluginVersion: 1,
        mode: mode,
        renderKind: renderKind,
        virtualPath: virtualPath,
        diagnostics: []
    )
}

func testHunk(
    path: String = "results.sarif",
    oldPath: String? = nil,
    oldContent: String? = nil,
    newContent: String? = nil,
    oldPreview: DiffPreview? = nil,
    newPreview: DiffPreview? = nil,
    hunkType: HunkType = .added,
    reviewIdentity: String = "identity",
    projection: DiffProjection? = nil
) -> DiffHunk {
    DiffHunk(
        path: path,
        oldPath: oldPath,
        oldContent: oldContent,
        newContent: newContent,
        oldPreview: oldPreview,
        newPreview: newPreview,
        hunkType: hunkType,
        reviewIdentity: reviewIdentity,
        projection: projection
    )
}
