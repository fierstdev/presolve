const { spawn } = require("node:child_process");
const { existsSync, readFileSync } = require("node:fs");
const { basename, dirname, join, relative, resolve } = require("node:path");

const SOURCE_LANGUAGES = new Set(["typescript", "typescriptreact"]);
const SOURCE_EXTENSIONS = new Set([".ts", ".tsx"]);

function activate(context) {
  const vscode = require("vscode");
  const output = vscode.window.createOutputChannel("Presolve");
  const diagnostics = vscode.languages.createDiagnosticCollection("presolve");
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  let lastResult = { errors: 0, warnings: 0 };
  let checking = false;

  const enabled = () => vscode.workspace.getConfiguration("presolve").get("enable", true);
  const configuration = (name, fallback) => vscode.workspace.getConfiguration("presolve").get(name, fallback);

  const refresh = () => {
    const workspace = inspectWorkspace(vscode);
    if (!enabled()) {
      status.hide();
      return workspace;
    }
    if (checking) {
      status.text = "$(sync~spin) Presolve";
      status.tooltip = "Presolve is checking compiler-owned application semantics.";
    } else if (!workspace.configured) {
      status.text = "$(warning) Presolve";
      status.tooltip = workspace.message;
    } else if (lastResult.errors > 0) {
      status.text = `$(error) Presolve ${lastResult.errors}`;
      status.tooltip = `${lastResult.errors} Presolve compiler error(s). Select to run a workspace check.`;
    } else {
      status.text = "$(check) Presolve";
      status.tooltip = `${workspace.message}\n${lastResult.warnings} compiler warning(s).`;
    }
    status.show();
    return workspace;
  };

  const execute = async (args, options = {}) => {
    const workspace = refresh();
    if (!workspace.configured) {
      vscode.window.showWarningMessage(workspace.message);
      return null;
    }
    checking = true;
    refresh();
    output.appendLine(`\n$ presolve ${args.join(" ")}`);
    const result = await runCompiler(workspace.root, args);
    if (result.stdout.trim()) output.appendLine(result.stdout.trimEnd());
    if (result.stderr.trim()) output.appendLine(result.stderr.trimEnd());
    checking = false;
    refresh();
    if (options.reveal || result.code !== 0) output.show(true);
    return result;
  };

  const checkDocument = async (document, reveal = false) => {
    if (!enabled() || !isPresolveDocument(document)) return null;
    const workspace = inspectWorkspace(vscode);
    if (!workspace.configured || !isInside(workspace.root, document.uri.fsPath)) return null;
    const source = relative(workspace.root, document.uri.fsPath);
    const result = await execute(["check", source, "--format", "json"], { reveal });
    if (!result) return null;
    const payload = parseJsonOutput(result.stdout);
    if (!payload) {
      if (result.code === 0) diagnostics.delete(document.uri);
      return result;
    }
    const extracted = extractDiagnostics(payload, document.uri.fsPath);
    const mapped = extracted.map((item) => toVscodeDiagnostic(vscode, document, item));
    diagnostics.set(document.uri, mapped);
    lastResult = summarizeDiagnostics(extracted);
    refresh();
    return { result, payload, extracted };
  };

  const checkWorkspace = async () => {
    const result = await execute(["check"], { reveal: true });
    if (result?.code === 0) {
      lastResult = { errors: 0, warnings: 0 };
      diagnostics.clear();
      vscode.window.showInformationMessage("Presolve workspace check passed.");
    } else if (result) {
      vscode.window.showErrorMessage("Presolve workspace check failed. See the Presolve output.");
    }
    refresh();
  };

  const explainDocument = async (uri) => {
    const document = uri ? await vscode.workspace.openTextDocument(uri) : vscode.window.activeTextEditor?.document;
    if (!document || !isPresolveDocument(document)) {
      vscode.window.showWarningMessage("Open a Presolve TypeScript or TSX source file first.");
      return;
    }
    const workspace = inspectWorkspace(vscode);
    if (!workspace.configured) {
      vscode.window.showWarningMessage(workspace.message);
      return;
    }
    const source = relative(workspace.root, document.uri.fsPath);
    const result = await execute(["explain", source, "--format", "json"]);
    const payload = result && parseJsonOutput(result.stdout);
    if (!payload) {
      vscode.window.showErrorMessage("Presolve could not produce compiler explanation data.");
      return;
    }
    const explanation = await vscode.workspace.openTextDocument({
      content: `${JSON.stringify(payload, null, 2)}\n`,
      language: "json",
    });
    await vscode.window.showTextDocument(explanation, { preview: true, viewColumn: vscode.ViewColumn.Beside });
  };

  const runProjectCommand = async (command) => {
    const result = await execute([command], { reveal: true });
    if (result?.code === 0) vscode.window.showInformationMessage(`Presolve ${command} completed.`);
  };

  const showStatus = async () => {
    const workspace = refresh();
    const selection = await vscode.window.showQuickPick([
      { label: "$(check) Check workspace", command: "presolve.checkWorkspace", description: "Validate compiler-owned application semantics" },
      { label: "$(file-code) Check active file", command: "presolve.checkActiveFile", description: "Publish exact compiler diagnostics" },
      { label: "$(inspect) Explain active file", command: "presolve.explainActiveFile", description: "Open compiler-derived source facts" },
      { label: "$(tools) Run doctor", command: "presolve.doctor", description: workspace.message },
      { label: "$(book) Open documentation", command: "presolve.openDocs", description: "Read the 0.2 beta guides" },
    ], { title: workspace.message, placeHolder: "Choose a Presolve action" });
    if (selection) await vscode.commands.executeCommand(selection.command);
  };

  const codeLensProvider = {
    async provideCodeLenses(document) {
      if (!enabled() || !configuration("codeLens", true) || !isPresolveDocument(document)) return [];
      const workspace = inspectWorkspace(vscode);
      if (!workspace.configured || !isInside(workspace.root, document.uri.fsPath)) return [];
      const source = relative(workspace.root, document.uri.fsPath);
      const result = await runCompiler(workspace.root, ["explain", source, "--format", "json"]);
      const payload = parseJsonOutput(result.stdout);
      return (payload?.componentClasses ?? []).map((component) => {
        const line = Math.max(0, Number(component.span?.line ?? 1) - 1);
        const range = new vscode.Range(line, 0, line, 0);
        return new vscode.CodeLens(range, {
          title: "$(preserve-case) Presolve component · Explain",
          command: "presolve.explainActiveFile",
          arguments: [document.uri],
          tooltip: `Open compiler-owned facts for ${component.name}.`,
        });
      });
    },
  };

  const subscriptions = [
    vscode.commands.registerCommand("presolve.status", showStatus),
    vscode.commands.registerCommand("presolve.checkWorkspace", checkWorkspace),
    vscode.commands.registerCommand("presolve.checkActiveFile", () => {
      const document = vscode.window.activeTextEditor?.document;
      return document ? checkDocument(document, true) : undefined;
    }),
    vscode.commands.registerCommand("presolve.explainActiveFile", explainDocument),
    vscode.commands.registerCommand("presolve.build", () => runProjectCommand("build")),
    vscode.commands.registerCommand("presolve.doctor", () => runProjectCommand("doctor")),
    vscode.commands.registerCommand("presolve.openDocs", () => vscode.env.openExternal(vscode.Uri.parse("https://presolve.dev/docs/"))),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (configuration("checkOnSave", true)) void checkDocument(document);
    }),
    vscode.workspace.onDidOpenTextDocument((document) => {
      if (configuration("checkOnOpen", false)) void checkDocument(document);
    }),
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      diagnostics.clear();
      refresh();
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("presolve")) refresh();
    }),
    vscode.languages.registerCodeLensProvider(
      [{ language: "typescriptreact", scheme: "file" }, { language: "typescript", scheme: "file" }],
      codeLensProvider,
    ),
    output,
    diagnostics,
    status,
  ];
  context.subscriptions.push(...subscriptions);
  status.command = "presolve.status";
  refresh();

  return Object.freeze({ enabled, checkDocument, inspectWorkspace: () => inspectWorkspace(vscode) });
}

function deactivate() {}

function inspectWorkspace(vscode) {
  const root = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  if (!root) return { configured: false, root: null, message: "Open a Presolve project folder to enable workspace integration." };
  if (!existsSync(join(root, "tsconfig.json"))) {
    return { configured: false, root, message: "This folder has no tsconfig.json. Create or open the Presolve project root." };
  }
  if (!existsSync(join(root, "app")) || !existsSync(join(root, "package.json"))) {
    return { configured: false, root, message: "This TypeScript folder does not have the canonical Presolve app/ and package.json inputs." };
  }
  const cli = resolveCompiler(root);
  if (!cli) {
    return { configured: false, root, message: "Install project dependencies so @presolve/cli is available in node_modules." };
  }
  const version = readPresolveVersion(root);
  return {
    configured: true,
    root,
    cli,
    version,
    message: `Presolve ${version ?? "workspace"} · compiler-owned diagnostics and explanations are available.`,
  };
}

function resolveCompiler(root) {
  const executable = process.platform === "win32" ? "presolve.cmd" : "presolve";
  const localBin = join(root, "node_modules", ".bin", executable);
  if (existsSync(localBin)) return localBin;
  return null;
}

function readPresolveVersion(root) {
  try {
    const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
    const value = manifest.dependencies?.presolve ?? manifest.devDependencies?.presolve;
    return typeof value === "string" ? value.replace(/^npm:@presolve\/framework@/, "") : null;
  } catch {
    return null;
  }
}

function runCompiler(root, args) {
  const command = resolveCompiler(root);
  if (!command) return Promise.resolve({ code: 127, stdout: "", stderr: "Project-local @presolve/cli is not installed." });
  return new Promise((resolveResult) => {
    const child = spawn(command, args, { cwd: root, env: process.env, windowsHide: true });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => { stdout += chunk; });
    child.stderr.on("data", (chunk) => { stderr += chunk; });
    child.on("error", (error) => resolveResult({ code: 127, stdout, stderr: `${stderr}${error.message}` }));
    child.on("close", (code) => resolveResult({ code: code ?? 1, stdout, stderr }));
  });
}

function parseJsonOutput(stdout) {
  try {
    return JSON.parse(stdout);
  } catch {
    return null;
  }
}

function extractDiagnostics(payload, activePath) {
  const groups = [
    ...(payload.parser_diagnostics ?? []),
    ...(payload.compiler_diagnostics ?? []),
    ...(payload.validation_diagnostics ?? []),
    ...(payload.production_diagnostics ?? []),
    ...(payload.diagnostics ?? []),
  ];
  return groups
    .map((entry) => {
      const provenance = entry.primary_provenance ?? entry.provenance ?? null;
      if (!provenance) return null;
      const normalizedActive = resolve(activePath).replaceAll("\\", "/");
      const normalizedProvenance = String(provenance.path ?? basename(activePath)).replaceAll("\\", "/").replace(/^\.\//, "");
      if (normalizedActive !== normalizedProvenance && !normalizedActive.endsWith(`/${normalizedProvenance}`)) return null;
      return {
        code: String(entry.code ?? "PRESOLVE"),
        message: String(entry.message ?? entry.name ?? "Presolve compiler diagnostic"),
        severity: String(entry.severity ?? "error").toLowerCase(),
        start: Number(provenance.start ?? 0),
        end: Number(provenance.end ?? provenance.start ?? 0),
        line: Number(provenance.line ?? 1),
        column: Number(provenance.column ?? 1),
      };
    })
    .filter(Boolean);
}

function toVscodeDiagnostic(vscode, document, item) {
  const text = document.getText();
  const start = item.start > 0 ? document.positionAt(byteOffsetToUtf16(text, item.start)) : new vscode.Position(Math.max(0, item.line - 1), Math.max(0, item.column - 1));
  const endOffset = Math.max(item.end, item.start + 1);
  const end = item.end > item.start ? document.positionAt(byteOffsetToUtf16(text, endOffset)) : new vscode.Position(start.line, start.character + 1);
  const severity = item.severity === "warning"
    ? vscode.DiagnosticSeverity.Warning
    : item.severity === "info"
      ? vscode.DiagnosticSeverity.Information
      : vscode.DiagnosticSeverity.Error;
  const diagnostic = new vscode.Diagnostic(new vscode.Range(start, end), item.message, severity);
  diagnostic.code = item.code;
  diagnostic.source = "Presolve";
  return diagnostic;
}

function byteOffsetToUtf16(text, byteOffset) {
  return Buffer.from(text, "utf8").subarray(0, Math.max(0, byteOffset)).toString("utf8").length;
}

function summarizeDiagnostics(items) {
  return items.reduce((summary, item) => {
    if (item.severity === "warning") summary.warnings += 1;
    else if (item.severity !== "info") summary.errors += 1;
    return summary;
  }, { errors: 0, warnings: 0 });
}

function isPresolveDocument(document) {
  return document?.uri?.scheme === "file"
    && SOURCE_LANGUAGES.has(document.languageId)
    && SOURCE_EXTENSIONS.has(document.uri.fsPath.slice(document.uri.fsPath.lastIndexOf(".")));
}

function isInside(root, path) {
  const rel = relative(root, path);
  return rel !== "" && !rel.startsWith("..") && !resolve(rel).startsWith("..");
}

module.exports = {
  activate,
  deactivate,
  byteOffsetToUtf16,
  extractDiagnostics,
  inspectWorkspace,
  parseJsonOutput,
  readPresolveVersion,
  resolveCompiler,
  summarizeDiagnostics,
};
