use gpui::actions;

actions!(
    jayjay,
    [
        OpenSettings,
        // cmd-w: close the focused window (after dismissing any open overlay).
        CloseWindow,
        // escape: dismiss an open overlay without closing the window.
        Dismiss,
        Refresh,
        OpenCommandPalette,
        OpenFind,
        CopyDiffSelection
    ]
);
