diff-type-to-find-ellipsis = Type to find…
diff-edit-file = Edit File
diff-edit-this-working-copy-file = Edit this working-copy file
diff-edit-diff = Edit Diff
diff-open-dedicated-diff-edit-mode = Open dedicated diff edit mode
diff-switch-to-side-by-side = Switch to side-by-side
diff-switch-to-unified = Switch to unified
diff-show-rendered-svg-preview = Show rendered SVG preview
diff-show-rendered-markdown = Show rendered Markdown
diff-open-working-copy-html-in-default = Open working-copy HTML in default app
diff-unified = Unified
diff-side-by-side = Side-by-side
diff-select-a-file = Select a file
diff-choose-a-file-from-the-list = Choose a file from the list to see its diff.
diff-loading-annotations-ellipsis = Loading annotations…
diff-no-annotations-available = No annotations available
diff-annotations-unavailable = Annotations unavailable
diff-git-submodule = Git submodule
diff-this-submodule-has-working-copy-changes = This submodule has working-copy changes, but JayJay does not render an inline text diff for submodule contents. Open or commit the submodule in its own repository.
diff-git-lfs-backed-file = Git LFS-backed file
diff-this-file-is-tracked-through-git = This file is tracked through Git LFS. JayJay does not render an inline text diff between the committed pointer and the local binary object.
diff-this-file-was-copied-its-contents = This file was copied; its contents match the source.
diff-this-file-was-renamed-its-contents = This file was renamed; its contents are identical.
diff-no-content-changes = No content changes
diff-loading-diff-ellipsis = Loading diff…
diff-no-textual-diff-binary-identical-or = No textual diff (binary, identical, or empty)
diff-show-in-unified = Show in Unified
diff-close-file-filter = Close file filter
diff-conflicted = Conflicted
diff-loading-files-ellipsis = Loading files…
diff-added = Added
diff-removed = Removed
diff-renamed = Renamed
diff-copied = Copied
diff-before = Before
diff-after = After
diff-drag-to-resize-images-double-click = Drag to resize images. Double-click to restore equal widths.
diff-no-post-change-markdown-content = No post-change Markdown content.
diff-image = Image
diff-select-change-group = Select Change Group
diff-select-file = Select File
diff-image-comparison-divider = Image comparison divider
diff-restore-equal-widths = Restore equal widths
diff-large-markdown-preview-truncated-to-keep = Large Markdown preview truncated to keep JayJay responsive.
diff-no-differences = No differences
diff-binary-property-list-on-disk-previewed = Binary property list on disk, previewed as XML
diff-show-table-preview = Show table preview
diff-show-notebook-preview = Show notebook preview
diff-show-property-list-preview = Show property list preview
diff-show-sarif-report-preview = Show SARIF report preview
diff-show-rich-preview = Show rich preview
diff-whitespace-only-changes-hidden-by-the = Whitespace-only changes hidden by the 'Ignore whitespace' setting.
diff-copy-path = Copy path
diff-show-rendered-svg = Show rendered SVG
diff-show-rendered-html-scripts-wont-run = Show rendered HTML (scripts won't run in this preview)
diff-show-rendered-html = Show rendered HTML
diff-show-source-diff = Show source diff
diff-new-change-description = New change description
diff-nothing-selected = Nothing Selected
diff-couldnt-load-all-files = Couldn't Load All Files
diff-diff-changed = Diff Changed
diff-show-more-lines = Show { NUMBER($count) }
diff-show-all = Show all
diff-unmodified-lines-action = { $action } unmodified lines
diff-jump-to-moved-copy = Jump to moved copy at { $location }
diff-edit-load-failed = Failed to load { $path }. Done was not applied — try again.
diff-review-notes-on-file =
    { $count ->
        [one] { $count } review note on this file
       *[other] { $count } review notes on this file
    }
diff-projection-preview = { $plugin } preview
diff-projection-preview-unavailable = { $plugin } preview unavailable
diff-image-label = { $side } image
diff-shortcut-to-save = { $shortcut } to save
diff-line-stats-label = { NUMBER($added) } added, { NUMBER($removed) } removed
diff-before-percent = Before { NUMBER($percent) } percent
diff-reviewed-of-total = { NUMBER($reviewed) }/{ NUMBER($total) } reviewed
diff-files-reviewed-with-agent = { NUMBER($reviewed) } of { NUMBER($total) } files reviewed, { NUMBER($agent) } include agent marks
diff-unmodified-lines =
    { $count ->
        [one] { $count } unmodified line
       *[other] { $count } unmodified lines
    }
diff-edit-hint = Select files, hunks, or line ranges to edit
diff-edit-selection =
    { $files ->
        [one] { $files } file
       *[other] { $files } files
    }, { $lines ->
        [one] { $lines } line
       *[other] { $lines } lines
    } selected
diff-plugin-notebook = Notebook
diff-plugin-delimited-table = Delimited table
diff-plugin-property-list = Property list
diff-plugin-sarif-report = SARIF report
diff-review-note-count =
    { $count ->
        [one] Review Note
       *[other] { $count } Review Notes
    }
diff-moved-from-to = { $kind } from { $old } to { $new }
diff-file-unavailable = (file unavailable)
diff-preview-unavailable = (preview unavailable)
diff-markdown-metadata = { NUMBER($count) } blocks, { $size }
diff-couldnt-open-file-editor = Couldn’t Open File Editor
diff-empty-markdown = (empty Markdown document)
diff-edit-selection-stale = { $path } changed after its selection was rendered. Done was not applied — refresh and try again.
diff-includes-agent-marks = { $label }, includes agent marks
diff-mode-changed = mode
