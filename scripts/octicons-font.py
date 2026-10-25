#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10,<3.14"
# dependencies = ["fonttools==4.60.2", "skia-pathops==0.8.0.post2"]
# ///
"""Build shell/gpui/assets/fonts/Octicons.ttf: one glyph per `glyph::` constant in icons.rs, from Octicons or a drawing below.

Map a new constant in OCTICON or DRAWN, then run: uv run scripts/octicons-font.py
"""
import io
import re
import tarfile
import urllib.request
from pathlib import Path

import pathops
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.svgLib.path import parse_path

OCTICONS_VERSION = "19.15.0"
ROOT = Path(__file__).resolve().parent.parent
ICONS_RS = ROOT / "shell/gpui/src/ui/icons.rs"
OUTPUT = ROOT / "shell/gpui/assets/fonts/Octicons.ttf"

UPM = 1024
GRID = 16
STROKE = 1.5

OCTICON = {
    "APP_WINDOW": "browser",
    "ARROW_DOWN": "arrow-down",
    "ARROW_RIGHT": "arrow-right",
    "ARROW_UP": "arrow-up",
    "ARROW_UTURN_BACK": "undo",
    "ARROWS_LEFT_RIGHT": "arrow-switch",
    "BOOKMARK": "bookmark",
    "CARET_DOWN": "chevron-down",
    "CARET_RIGHT": "chevron-right",
    "CASE_SENSITIVE": "typography",
    "CHART_BAR": "graph",
    "CHECK": "check",
    "CLOUD": "cloud",
    "CLOUD_OFF": "cloud-offline",
    "COLUMNS": "columns",
    "COPY": "copy",
    "DATABASE": "database",
    "DIFF": "diff",
    "DOT": "dot-fill",
    "EDIT_CODE": "code-square",
    "EXTERNAL_LINK": "link-external",
    "EYE": "eye",
    "EYE_OFF": "eye-closed",
    "FILE_CODE": "file-code",
    "FILTER": "filter",
    "FOLDER": "file-directory",
    "GEAR": "gear",
    "GIT_BRANCH": "git-branch",
    "GIT_MERGE": "git-merge",
    "GLOBE": "globe",
    "HISTORY": "history",
    "INFO": "info",
    "LIST": "list-unordered",
    "LIST_CHECKS": "checklist",
    "MARK_GITHUB": "mark-github",
    "MOON": "moon",
    "PACKAGE": "package",
    "PALETTE": "paintbrush",
    "PANEL_LEFT": "sidebar-collapse",
    "PENCIL": "pencil",
    "PIN": "pin",
    "PIN_OFF": "pin-slash",
    "PLUS": "plus",
    "ROWS": "rows",
    "SEARCH": "search",
    "SHIELD": "shield",
    "SLIDERS_HORIZONTAL": "sliders",
    "SPARKLE": "sparkle-fill",
    "SUBMODULE": "file-submodule",
    "SUN": "sun",
    "SYNC": "sync",
    "TAG": "tag",
    "TERMINAL": "terminal",
    "TRASH": "trash",
    "UNFOLD_VERTICAL": "unfold",
    "WARNING": "alert",
    "WRENCH": "tools",
    "X": "x",
    "X_CIRCLE": "x-circle",
    "ZOOM_IN": "zoom-in",
    "ZOOM_OUT": "zoom-out",
}

RING = "M8 1.25a6.75 6.75 0 1 1 0 13.5a6.75 6.75 0 1 1 0-13.5Z"
LEFT_BRACE = "M5.75 2.25H5a1.75 1.75 0 0 0-1.75 1.75v2.25A1.75 1.75 0 0 1 1.5 8a1.75 1.75 0 0 1 1.75 1.75V12A1.75 1.75 0 0 0 5 13.75h.75"


def drawn(d, transform=None):
    path = pathops.Path()
    pen = path.getPen()
    parse_path(d, TransformPen(pen, transform) if transform else pen)
    return path


def stroked(d, transform=None):
    path = drawn(d, transform)
    path.stroke(STROKE, pathops.LineCap.ROUND_CAP, pathops.LineJoin.ROUND_JOIN, 4)
    path.convertConicsToQuads(0.01)
    return path


def union(*paths):
    out = pathops.Path()
    for path in paths:
        out = pathops.op(out, path, pathops.PathOp.UNION)
    return out


MIRROR_X = (-1, 0, 0, 1, GRID, 0)
QUARTER_TURN = (0, 1, -1, 0, GRID, 0)

DRAWN = {
    "ARROW_CIRCLE_RIGHT": lambda svgs: union(stroked(RING), stroked("M4.75 8h6M8.5 5.25 11.25 8 8.5 10.75")),
    "ARROW_UP_DOWN": lambda svgs: union(*(drawn(d, QUARTER_TURN) for d in svgs["arrow-switch"])),
    "A_LARGE_SMALL": lambda svgs: union(
        stroked("M1.5 13.25 5.25 2.75 9 13.25M2.85 9.5h4.8"),
        stroked("M9.25 13.25 11.75 7l2.5 6.25M10.15 11.25h3.2"),
    ),
    "BRACES": lambda svgs: union(stroked(LEFT_BRACE), stroked(LEFT_BRACE, MIRROR_X)),
    "CONTRAST": lambda svgs: union(stroked(RING), drawn("M8 1.25a6.75 6.75 0 0 0 0 13.5Z")),
    "FOLDER_MINUS": lambda svgs: union(*(drawn(d) for d in svgs["file-directory"]), stroked("M5.75 9h4.5")),
    "LIGATURE": lambda svgs: union(
        stroked("M4.75 14V5.25a3 3 0 0 1 3-3h1.5a2.5 2.5 0 0 1 2.5 2.5"),
        stroked("M2.75 7.25h9v6.75"),
    ),
    "LIST_TREE": lambda svgs: union(
        drawn("M2.75 1.75a1 1 0 1 1 0 2a1 1 0 1 1 0-2Z"),
        stroked("M5.75 2.75h8.25M7.75 8h6.25M7.75 13.25h6.25"),
        stroked("M2.75 5.5v6a1.75 1.75 0 0 0 1.75 1.75h.75M2.75 8h2.5"),
    ),
    "WHITESPACE": lambda svgs: union(
        stroked("M8.25 2.75V14M11.5 2.75V14M6.5 2.75h7.25"),
        drawn("M9 2V9.5H6.5a3.75 3.75 0 0 1 0-7.5Z"),
    ),
}


def octicon_paths():
    url = f"https://registry.npmjs.org/@primer/octicons/-/octicons-{OCTICONS_VERSION}.tgz"
    with urllib.request.urlopen(url) as response:
        archive = tarfile.open(fileobj=io.BytesIO(response.read()))
    paths = {}
    for member in archive.getmembers():
        match = re.fullmatch(r"package/build/svg/(.+)-16\.svg", member.name)
        if match:
            svg = archive.extractfile(member).read().decode()
            paths[match[1]] = re.findall(r'<path[^>]* d="([^"]+)"', svg)
    return paths


def outline(name, svgs):
    path = DRAWN[name](svgs) if name in DRAWN else union(*(drawn(d) for d in svgs[OCTICON[name]]))
    scale = UPM / GRID
    font_path = pathops.Path()
    path.draw(TransformPen(font_path.getPen(), (scale, 0, 0, -scale, 0, UPM)))
    return pathops.simplify(font_path, fix_winding=True, clockwise=True)


def main():
    codepoints = dict(re.findall(r'pub const ([A-Z_]+): &str = "\\u\{([0-9a-f]+)\}"', ICONS_RS.read_text()))
    unmapped = sorted(set(codepoints) - OCTICON.keys() - DRAWN.keys())
    if unmapped:
        raise SystemExit(f"map these glyph constants in OCTICON or DRAWN: {', '.join(unmapped)}")
    svgs = octicon_paths()

    blank = TTGlyphPen(None).glyph()
    # GPUI measures the em with "m", so the font needs a glyph for it.
    glyphs = {".notdef": blank, "space": blank, "m": blank}
    metrics = {name: (UPM, 0) for name in glyphs}
    cmap = {ord(" "): "space", ord("m"): "m"}
    for name, codepoint in sorted(codepoints.items()):
        path = outline(name, svgs)
        pen = TTGlyphPen(None)
        path.draw(Cu2QuPen(pen, max_err=0.5))
        bounds = BoundsPen(None)
        path.draw(bounds)
        glyph_name = name.lower()
        glyphs[glyph_name] = pen.glyph()
        metrics[glyph_name] = (UPM, round(bounds.bounds[0]))
        cmap[int(codepoint, 16)] = glyph_name

    builder = FontBuilder(UPM, isTTF=True)
    builder.setupGlyphOrder(list(glyphs))
    builder.setupCharacterMap(cmap)
    builder.setupGlyf(glyphs)
    builder.setupHorizontalMetrics(metrics)
    builder.setupHorizontalHeader(ascent=UPM, descent=0, lineGap=0)
    builder.setupNameTable({"familyName": "Octicons", "styleName": "Regular", "uniqueFontIdentifier": "Octicons", "fullName": "Octicons", "psName": "Octicons"})
    builder.setupOS2(sTypoAscender=UPM, sTypoDescender=0, sTypoLineGap=92, usWinAscent=1116, usWinDescent=14)
    builder.setupPost()
    builder.save(OUTPUT)
    print(f"{len(codepoints)} glyphs from Octicons {OCTICONS_VERSION} -> {OUTPUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
