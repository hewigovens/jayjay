settings-window-title = JayJay Settings
settings-page-appearance = Appearance
settings-page-fonts = Fonts
settings-page-diff = Diff & Files
settings-page-workflow = Workflow
settings-page-integrations = Integrations
settings-page-jujutsu = Jujutsu
settings-page-data-privacy = Data & Privacy
settings-page-about = About

settings-appearance-system = System
settings-appearance-light = Light
settings-appearance-dark = Dark
settings-language = Language
settings-language-system = System
settings-language-detail = Some system menus and update dialogs change after relaunch.
settings-language-relaunch = Relaunch
settings-colors = Colors
settings-light-theme = Light theme
settings-dark-theme = Dark theme
settings-wallpaper-tint = Tint window background with wallpaper color
settings-wallpaper-tint-detail = Allow macOS wallpaper tinting in dark mode. Turn off for a flat, neutral background.
settings-wallpaper-tint-unavailable = Wallpaper tinting is available with the { $theme } theme.

settings-interface-font = Interface Font
settings-code-font = Code Font
settings-code-font-detail = Used for diffs, editors, revsets, and identifiers.
settings-font-zoom-hint = Press { $zoom_in } or { $zoom_out } to change the interface and code font sizes, { $reset_zoom } to reset them.
settings-font-family = Family
settings-font-size = Size
settings-font-points = { NUMBER($size) } pt
settings-font-system = System
settings-font-system-mono = System Mono
settings-font-system-default = System default
settings-font-not-installed = { $family } is not installed; using { $fallback }.
settings-my-fonts = My Fonts
settings-coding-fonts = Coding Fonts
settings-other-monospaced = Other Monospaced
settings-line-spacing = Line spacing
settings-ligatures = Ligatures
settings-ligatures-unavailable = { $family } has no ligatures.
settings-system-font = The system font
settings-reset-fonts = Reset Fonts

settings-display = Display
settings-side-by-side = Side-by-side diff
settings-ignore-whitespace = Ignore whitespace changes
settings-tree-view = Tree view for files
settings-hide-reviewed = Hide reviewed files
settings-auto-expand = Auto-expand descriptions
settings-large-files = Large Files
settings-hide-lfs = Hide Git LFS-backed files

settings-confirmations = Confirmations
settings-confirm-abandon = Confirm before abandoning changes
settings-confirm-workspace-delete = Confirm before deleting a workspace
settings-confirm-drag-rebase = Confirm drag-to-rebase
settings-repository-behavior = Repository Behavior
settings-submodule-support = Enable Git submodule support

settings-applications = Applications
settings-editor = Editor
settings-terminal = Terminal
settings-command = Command
settings-app-name = App name
settings-example = e.g. { $example }
settings-ai-commit-message = AI Commit Message
settings-ai-providers = AI Providers
settings-ai-providers-detail = Drag to reorder. The first provider that answers wins.
settings-available = Available
settings-not-available = Not available
settings-version-control = Version Control
settings-jj-tool-config = jj tool configuration
settings-jj-tool-config-detail = diff, edit & merge
settings-copy-jj-tool-config = Copy jj tool configuration
settings-copy-config = Copy Config
settings-cli-unavailable = Unavailable: no home directory
settings-cli-link-missing = Link target is missing: { $target }
settings-cli-links-to = This entry links to { $target }
settings-cli-unknown-file = A file JayJay didn't install is already here
settings-cli-unmanaged = { $reason }; remove it manually if JayJay should manage this command.
settings-forges = Forges
settings-not-signed-in = Not signed in
settings-sign-in-title = Sign In to { $host }
settings-sign-in-detail = Enter this code in your browser to let JayJay use your account.
settings-sign-in-code = Enter { $code } at { $url }
settings-requesting-code = Requesting a sign-in code…
settings-waiting-for-approval = Waiting for approval…
settings-copy-code-and-open = Copy Code & Open Browser

settings-identity = Identity
settings-identity-not-set-up = Not set up
settings-identity-title = Set Up Identity
settings-identity-detail = jj records this name and email on the changes you create.
settings-identity-name = Name
settings-identity-name-placeholder = Your Name
settings-identity-email = Email
settings-loading-config = Loading jj config…
settings-config-not-loaded = jj config has not been loaded.
settings-config-not-found = Config not found

settings-tagline = A native GUI for Jujutsu
settings-version = Version { $version }
settings-version-build = Version { $version } • Build { $build }
settings-love-jayjay = Love JayJay?
settings-sponsor = Sponsor
settings-star-on-github = Star on GitHub
settings-check-updates = Check for updates automatically
settings-update-channel = Update channel
settings-update-stable = Stable
settings-update-beta = Beta

settings-privacy = Privacy
settings-diagnostics = Diagnostics
settings-debug-mode = Debug mode
settings-debug-mode-detail = Logs more detail. Send Feedback attaches the log in Mail, or reveals it for you to attach in other mail apps.
settings-review-data = Review Data
settings-review-data-detail = Review marks and notes are stored on this Mac. Clearing them affects every repository and cannot be undone.
settings-review-marks-and-notes = Review marks and notes
settings-review-summary =
    { $marks ->
        [one] { $marks } mark
       *[other] { $marks } marks
    }, { $notes ->
        [one] { $notes } note
       *[other] { $notes } notes
    }
settings-clear-review-title = Clear all review marks and notes?
settings-clear-review-message = Removes { $summary } for every repository. This cannot be undone.
settings-custom = Custom
settings-file-explorer = File Explorer
settings-cli-install-is-not-supported-on = CLI install is not supported on this platform yet
settings-cli-already-installed = CLI Already Installed
settings-cli-installed = CLI Installed
settings-cli-install-failed = CLI Install Failed
settings-cli-binary-not-found-in-app = CLI binary not found in app bundle
settings-cli-not-installed-by-jayjay = { $path } was not installed by JayJay; remove it manually
settings-appimage-not-found = AppImage not found at { $path }
settings-cli-not-on-path-fish = { $dir } is not on your PATH. Add to ~/.config/fish/config.fish: fish_add_path { $dir }
settings-cli-not-on-path = { $dir } is not on your PATH. Add to { $file }: export PATH="{ $dir }:$PATH"
settings-cli-not-executable = CLI binary not executable: { $path }
settings-cli-add-to-rc-fish =
    Add to { $file }:
    fish_add_path { $dir }
settings-cli-add-to-rc =
    Add to { $file }:
    export PATH="{ $dir }:$PATH"
settings-terminal-app-name = App name
settings-terminal-command = Command
settings-terminal-command-powershell = Command (accepts PowerShell arguments)
settings-cli-nothing-installed-at = Nothing is installed at { $path }
settings-cli-already-linked-at = jayjay is already linked at { $path }.
settings-cli-linked-at = jayjay is linked at { $path }.
settings-cli-linked-not-in-path =
    jayjay is linked at { $path }, but the directory is not in your PATH.
    
    { $hint }
settings-cli-was-linked-not-in-path =
    jayjay was linked at { $path }, but the directory is not in your PATH.
    
    { $hint }
