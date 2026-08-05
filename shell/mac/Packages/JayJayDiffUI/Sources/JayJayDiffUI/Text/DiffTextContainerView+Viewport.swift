import AppKit

extension DiffTextContainerView {
    func captureViewportAnchor() -> DiffViewportAnchor? {
        guard !viewportLineLocations.isEmpty,
              let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer
        else { return nil }

        let clipView = scrollView.contentView
        let containerPoint = NSPoint(
            x: textView.textContainerInset.width,
            y: max(0, clipView.bounds.minY - textView.textContainerInset.height + 0.5)
        )
        let glyphIndex = layoutManager.glyphIndex(for: containerPoint, in: textContainer)
        let characterIndex = layoutManager.characterIndexForGlyph(at: glyphIndex)
        guard let location = viewportLocation(atOrAfter: characterIndex) else { return nil }

        let rect = textRect(for: location, layoutManager: layoutManager, textContainer: textContainer)
        return DiffViewportAnchor(
            identity: location.identity,
            offsetFromVisibleTop: rect.minY - clipView.bounds.minY
        )
    }

    func setViewportLineLocations(
        _ locations: [DiffViewportLineLocation],
        restoring anchor: DiffViewportAnchor?
    ) {
        viewportLineLocations = locations
        pendingViewportAnchor = anchor
        needsLayout = true
    }

    func applySelectionResetGeneration(_ generation: UInt64) {
        defer { lastSelectionResetGeneration = generation }
        guard let previous = lastSelectionResetGeneration, generation != previous else { return }
        textView.setSelectedRange(NSRange(location: 0, length: 0))
        gutterTextView.resetLineSelection()
    }

    func scheduleRevealFeedback(
        _ feedback: DiffContextRevealFeedback?,
        reduceMotion: Bool
    ) {
        guard let feedback else { return }
        guard lastRevealFeedbackGeneration != feedback.generation else { return }
        lastRevealFeedbackGeneration = feedback.generation
        guard DiffContextRevealFeedbackPolicy.shouldAnimate(
            feedback: feedback,
            reduceMotion: reduceMotion
        ) else { return }
        pendingRevealFeedback = PendingDiffRevealFeedback(
            feedback: feedback,
            reduceMotion: reduceMotion
        )
        needsLayout = true
    }

    func applyPendingViewportUpdates() {
        if let anchor = pendingViewportAnchor {
            pendingViewportAnchor = nil
            restoreViewportAnchor(anchor)
        }
        if let pending = pendingRevealFeedback {
            pendingRevealFeedback = nil
            showRevealFeedback(pending)
        }
    }

    private func restoreViewportAnchor(_ anchor: DiffViewportAnchor) {
        guard let location = viewportLineLocations.first(where: {
            $0.identity == anchor.identity
        }) ?? anchor.identity.fallback.flatMap({ fallback in
            viewportLineLocations.first(where: { $0.identity == fallback })
        }),
            let layoutManager = textView.layoutManager,
            let textContainer = textView.textContainer
        else { return }

        let rect = textRect(for: location, layoutManager: layoutManager, textContainer: textContainer)
        let documentHeight = layoutManager.usedRect(for: textContainer).height
            + textView.textContainerInset.height * 2
        let maximumY = max(0, documentHeight - scrollView.contentView.bounds.height)
        let targetY = min(max(0, rect.minY - anchor.offsetFromVisibleTop), maximumY)
        scroll(toY: targetY)
    }

    private func showRevealFeedback(_ pending: PendingDiffRevealFeedback) {
        guard !pending.reduceMotion,
              let newLineRange = pending.feedback.newLineRange,
              let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer
        else { return }

        let matching = viewportLineLocations.filter {
            $0.identity.revealNewLine.map(newLineRange.contains) == true
        }
        guard let first = matching.first else { return }

        var revealedRect = textRect(
            for: first,
            layoutManager: layoutManager,
            textContainer: textContainer
        )
        for location in matching.dropFirst() {
            revealedRect = revealedRect.union(textRect(
                for: location,
                layoutManager: layoutManager,
                textContainer: textContainer
            ))
        }

        let rectInContainer = textView.convert(revealedRect, to: self)
            .intersection(scrollView.frame)
        guard !rectInContainer.isNull, rectInContainer.height > 0 else { return }

        let overlay = DiffRevealFeedbackView(frame: NSRect(
            x: scrollView.frame.minX,
            y: rectInContainer.minY,
            width: scrollView.frame.width,
            height: rectInContainer.height
        ))
        overlay.wantsLayer = true
        overlay.layer?.backgroundColor = NSColor.controlAccentColor
            .withAlphaComponent(0.12)
            .cgColor
        addSubview(overlay, positioned: .above, relativeTo: scrollView)
        NSAnimationContext.runAnimationGroup { context in
            context.duration = 0.14
            context.timingFunction = CAMediaTimingFunction(name: .easeOut)
            overlay.animator().alphaValue = 0
        } completionHandler: {
            overlay.removeFromSuperview()
        }
    }

    private func viewportLocation(atOrAfter characterIndex: Int) -> DiffViewportLineLocation? {
        var low = 0
        var high = viewportLineLocations.count - 1
        var candidate: Int?
        while low <= high {
            let mid = (low + high) / 2
            let range = viewportLineLocations[mid].characterRange
            if characterIndex < range.location {
                candidate = mid
                high = mid - 1
            } else if characterIndex >= NSMaxRange(range) {
                low = mid + 1
            } else {
                return viewportLineLocations[mid]
            }
        }
        if let candidate {
            return viewportLineLocations[candidate]
        }
        return viewportLineLocations.last
    }

    private func textRect(
        for location: DiffViewportLineLocation,
        layoutManager: NSLayoutManager,
        textContainer: NSTextContainer
    ) -> NSRect {
        layoutManager.ensureLayout(forCharacterRange: location.characterRange)
        let glyphRange = layoutManager.glyphRange(
            forCharacterRange: location.characterRange,
            actualCharacterRange: nil
        )
        var rect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
        rect.origin.x += textView.textContainerInset.width
        rect.origin.y += textView.textContainerInset.height
        return rect
    }

    private func scroll(toY y: CGFloat) {
        var contentOrigin = scrollView.contentView.bounds.origin
        contentOrigin.y = y
        scrollView.contentView.scroll(to: contentOrigin)
        scrollView.reflectScrolledClipView(scrollView.contentView)

        var gutterOrigin = gutterScrollView.contentView.bounds.origin
        gutterOrigin.y = y
        gutterScrollView.contentView.scroll(to: gutterOrigin)
        gutterScrollView.reflectScrolledClipView(gutterScrollView.contentView)
    }
}
