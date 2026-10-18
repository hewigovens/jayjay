(() => {
  const style = document.createElement('style');
  style.textContent = `
.mermaid-diagram {
  margin: 0 0 1em;
  --pele-bg: Canvas;
  --pele-fg: CanvasText;
  --pele-muted: color-mix(in srgb, CanvasText 62%, Canvas);
  --pele-line: color-mix(in srgb, CanvasText 72%, Canvas);
  --pele-surface: color-mix(in srgb, CanvasText 7%, Canvas);
  --pele-surface-alt: color-mix(in srgb, CanvasText 4%, Canvas);
  --pele-border: color-mix(in srgb, CanvasText 30%, Canvas);
  --pele-accent: AccentColor;
  --pele-font-mono: ui-monospace, "SF Mono", Menlo, monospace;
}`;
  document.head.append(style);

  for (const code of document.querySelectorAll('pre > code.language-mermaid')) {
    const diagram = document.createElement('div');
    diagram.className = 'mermaid-diagram';
    code.parentElement.replaceWith(diagram);
    pele.mount(diagram, code.textContent, { images: false });
  }
})();
