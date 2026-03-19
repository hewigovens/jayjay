(function () {
  let monacoRef = null;
  let diffEditor = null;
  let currentModels = [];
  let pendingPayload = null;
  let ready = false;

  const loadingEl = document.getElementById("loading");
  const editorEl = document.getElementById("editor");

  function notifyReady() {
    if (
      window.webkit &&
      window.webkit.messageHandlers &&
      window.webkit.messageHandlers.monacoReady
    ) {
      window.webkit.messageHandlers.monacoReady.postMessage("ready");
    }
  }

  function languageForPath(path) {
    const ext = (path.split(".").pop() || "").toLowerCase();
    switch (ext) {
      case "rs":
        return "rust";
      case "swift":
        return "swift";
      case "js":
      case "cjs":
      case "mjs":
        return "javascript";
      case "ts":
      case "tsx":
        return "typescript";
      case "jsx":
        return "javascript";
      case "json":
        return "json";
      case "md":
        return "markdown";
      case "toml":
        return "ini";
      case "yaml":
      case "yml":
        return "yaml";
      case "sh":
      case "zsh":
      case "bash":
        return "shell";
      case "html":
        return "html";
      case "css":
        return "css";
      case "py":
        return "python";
      case "go":
        return "go";
      case "java":
        return "java";
      case "kt":
        return "kotlin";
      case "sql":
        return "sql";
      default:
        return "plaintext";
    }
  }

  function disposeModels() {
    currentModels.forEach(function (model) {
      if (model) {
        model.dispose();
      }
    });
    currentModels = [];
  }

  function applyPayload(payload) {
    if (!ready || !monacoRef || !diffEditor) {
      pendingPayload = payload;
      return;
    }

    pendingPayload = payload;
    var isDark = payload.theme === "vs-dark" || payload.theme === "github-dark";
    document.body.classList.toggle("dark", isDark);
    monacoRef.editor.setTheme(payload.theme);

    disposeModels();

    const language = languageForPath(payload.path || "");
    const originalModel = monacoRef.editor.createModel(
      payload.original || "",
      language
    );
    const modifiedModel = monacoRef.editor.createModel(
      payload.modified || "",
      language
    );
    currentModels = [originalModel, modifiedModel];

    diffEditor.updateOptions({
      renderSideBySide: payload.renderSideBySide !== false,
      wordWrap: payload.wordWrap || "on",
      fontSize: payload.fontSize || 12,
    });
    diffEditor.setModel({
      original: originalModel,
      modified: modifiedModel,
    });

    loadingEl.style.display = "none";
    editorEl.classList.add("ready");
  }

  window.renderDiff = function (payload) {
    applyPayload(payload);
  };

  function registerCustomThemes(monaco) {
    monaco.editor.defineTheme("github-light", {
      base: "vs",
      inherit: true,
      rules: [
        { token: "comment", foreground: "6a737d", fontStyle: "italic" },
        { token: "keyword", foreground: "d73a49" },
        { token: "string", foreground: "032f62" },
        { token: "number", foreground: "005cc5" },
        { token: "type", foreground: "6f42c1" },
        { token: "class", foreground: "6f42c1" },
        { token: "function", foreground: "6f42c1" },
        { token: "variable", foreground: "e36209" },
        { token: "operator", foreground: "d73a49" },
        { token: "constant", foreground: "005cc5" },
        { token: "tag", foreground: "22863a" },
        { token: "attribute.name", foreground: "6f42c1" },
        { token: "attribute.value", foreground: "032f62" },
      ],
      colors: {
        "editor.background": "#ffffff",
        "editor.foreground": "#24292e",
        "editor.lineHighlightBackground": "#f6f8fa",
        "editorLineNumber.foreground": "#babbbd",
        "editorLineNumber.activeForeground": "#24292e",
        "editor.selectionBackground": "#0366d625",
        "diffEditor.insertedTextBackground": "#28a74530",
        "diffEditor.removedTextBackground": "#d73a4930",
        "diffEditor.insertedLineBackground": "#dafbe1",
        "diffEditor.removedLineBackground": "#ffeef0",
      },
    });

    monaco.editor.defineTheme("github-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [
        { token: "comment", foreground: "8b949e", fontStyle: "italic" },
        { token: "keyword", foreground: "ff7b72" },
        { token: "string", foreground: "a5d6ff" },
        { token: "number", foreground: "79c0ff" },
        { token: "type", foreground: "d2a8ff" },
        { token: "class", foreground: "d2a8ff" },
        { token: "function", foreground: "d2a8ff" },
        { token: "variable", foreground: "ffa657" },
        { token: "operator", foreground: "ff7b72" },
        { token: "constant", foreground: "79c0ff" },
        { token: "tag", foreground: "7ee787" },
        { token: "attribute.name", foreground: "d2a8ff" },
        { token: "attribute.value", foreground: "a5d6ff" },
      ],
      colors: {
        "editor.background": "#0d1117",
        "editor.foreground": "#c9d1d9",
        "editor.lineHighlightBackground": "#161b22",
        "editorLineNumber.foreground": "#484f58",
        "editorLineNumber.activeForeground": "#c9d1d9",
        "editor.selectionBackground": "#388bfd26",
        "diffEditor.insertedTextBackground": "#23863630",
        "diffEditor.removedTextBackground": "#da363430",
        "diffEditor.insertedLineBackground": "#12261e",
        "diffEditor.removedLineBackground": "#2d1315",
      },
    });
  }

  require(["vs/editor/editor.main"], function (monaco) {
    monacoRef = monaco;
    registerCustomThemes(monaco);
    diffEditor = monaco.editor.createDiffEditor(editorEl, {
      automaticLayout: true,
      readOnly: true,
      originalEditable: false,
      renderSideBySide: true,
      useInlineViewWhenSpaceIsLimited: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      renderMarginRevertIcon: false,
      renderOverviewRuler: true,
      renderIndicators: true,
      diffWordWrap: "on",
      wordWrap: "on",
      hideUnchangedRegions: {
        enabled: true,
        contextLineCount: 4,
        minimumLineCount: 3,
        revealLineCount: 12,
      },
    });

    ready = true;
    notifyReady();

    if (pendingPayload) {
      applyPayload(pendingPayload);
    }
  });
})();
